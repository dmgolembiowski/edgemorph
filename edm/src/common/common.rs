/* Disclaimer: Portions of this file were originally published
 * by contributors/developers at [EdgeDB](https://github.com/edgedb).
 * By using this software, you are subject to all of their licenses
 * including those not explicitly defined in this file.
 */
#[cfg(not(windows))]
#[macro_use] extern crate pretty_assertions;
use async_trait::async_trait;
use futures::select;
use futures::FutureExt;
use async_std::{
    io::{stdin, BufReader},
    net::{TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};
use std::sync::Mutex;
use std::convert::TryInto;
use std::io::{BufReader, BufRead};
use std::sync::mpsc::sync_channel;
use std::thread::{self, JoinHandle};
use std::process;
use std::env;
use assert_cmd::Command;
use serde::{Deserialize};
use serde_json::{Result as SerdeResult, from_str};
use once_cell::sync::Lazy;
use std::fs::{File};
use edgedb_client::{
    Builder,
    client,
    credentials,
    errors, 
    reader, 
    server_params};
use std::sync::Weak;
use std::rc::Cell;
use async_std::sync::Weak as AsyncWeak;
use double_checked_cell::DoubleCheckedCell as Duracell;
use futures::future::ready;
use std::marker::PhantomPinned;
use core::pin::Pin;
use pin_utils::pin_mut;
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;
use bytes::{Bytes, BytesMut};
use edgedb_protocol::server_message::{
    Authentication,
    Cardinality,
    ClientHandshake,
    ClientMessage,
    CommandComplete,
    CommandDataDescription,
    Data,
    DescribeAspect,
    DescribeStatement,
    ErrorResponse,
    ErrorSensitivity,
    ErrorSeverity,
    Execute,
    ExecuteScript,
    IoFormat,
    LogMessage,
    MessageSeverity,
    ParameterStatus,
    Prepare,
    PrepareComplete,
    ReadyForCommand,
    Restore,
    RestoreReady,
    ServerHandshake,
    ServerKeyData,
    ServerMessage,
    TransactionState
};


/* EdmFiber
 *
 * This generic `async_trait` is responsible for 
 * enforcing a common policy for talking to EdgeDB
 * using the high-level API from 
 * [edgedb_rust](github.com/edgedb/edgedb-rust).
 * 
 * It enforces the minimum requirements for talking
 * to EdgeDB in a deterministic way under
 * asynchronous conditions.
 * 
 * To implement this trait, your program will need:
 *   - U: Actor
 *   - C: Courier + Client
 *
 * `U` can be viewed as the "user" of edgemorph.
 * They want to run actions on the database. In my own
 * experience, I couldn't care less about the database's 
 * response to an INSERT or DELETE query.
 * As an EdgeDB user, I only want to know when an error
 * happens -- and as long as one doesn't -- I can't be
 * bothered to slow down and block everything until this
 * query finishes.
 *
 * On the other hand, `SELECT` statements are vitally important
 * for any of my applications. I probably have upstream functions
 * and procedures that depend on the outcome of a `SELECT` statement.
 * Under these conditions, waiting is fine.
 * But since I'm impatient, I still want other asynchronous work
 * to continue.
 *
 * Likewise, `C` represents features of both a database "client",
 * as well as a "corresponder" or "courier". It talks to the database,
 * and informs the `U: Actor` of any responses -- provided that the `U`
 * cares to receive them.
 */
#[async_trait]
trait EdmFiber<U: Actor, C: Courier + Client> {

    // Spawns a fiber for cooperatively multitasking work between the user
    // and the database client socket. Returns an error when the underlying
    // OS cannot spare any more resources.
    async fn spawn() -> Result<(Self)>;

    // Closes the database connection gracefully. A `timeout` should be
    // supplied as a number of seconds from an `i32`. 
    // Otherwise, if `timeout = None` this call could hang.
    async fn aclose(&mut self, timeout: Option<i32>) -> Result<()>;
    
    // Connects this nonblocking fiber to the database with an `async_std::net::TcpStream`.
    // `C: Client` are responsible for passing an `edgedb_client::Builder`
    // for connection criteria.
    async fn connect(&mut self, timeout: Option<i32>) -> Result<()>;

    // "Provide an explicit synchronization point (for the transaction state) ... Sent by the Client"
    // https://github.com/edgedb/edgedb/blob/a3ccdaa00c2dbe55675794f2fc97314c4fcdb0b7/edb/testbase/protocol/protocol.pyx
    // ```
    //   struct Sync {
    //      // Message type ('S').
    //      uint8           mtype = 0x53;
    //
    //      // Length of message contents in bytes,
    //      // including self.
    //      uint32          message_length;
    //    };
    // ```
    async fn sync(&mut self);

    // Should resemble the `mpsc::Channel`. Continuously awaiting new packets
    // and matching their contents to potential log messages.
    // Stop `recv`-ing once we get a message.
    //
    // Trait implementors should probably implement an optional private method
    // for `recv_match` based upon the "class" of message.
    async fn recv(&mut self);

    async fn send(&mut self, msg: &ClientMessage);

    // An alternative to `aclose` that emphasizes "undocking"
    // from the `C: Courier` relation, but maintaining its `C: Client` stream.
    async fn release(&self) -> Result<()>;
}

//! `Actor`
//! 
//! This trait defines shared behavior for two closely-related behaviors:
//!   1) Client-message creation
//!   2) Server-response handling
//! 
//! Earlier we noted that `Actor` is really just a proxy for the Edgemorph user.
//! Implementors of the `Actor` are responsible for building low-level communication
//! corresponding to the `START TRANSACTION` or `BEGIN MIGRATION` calls
//! originating in `edm make install`. 
//!
//! Additionally, `Actor` also templates the behavior for awaiting and processing
//! `edgemorph` related code in the Python and Rust codegen shared objects.
//! This will likely need to be re-factored into the `edgemorph-rs` top-level
//! workspace member.
//!
//!
#[async_trait]
trait Actor<T, S> 
    /* Generic implementations on [G] implicitly
       implement the trait on anything that
       dereferences to [G], including Vec<G>, Box<[G]>,
       and Rc<[G]>.
       */
    where T: From<[ClientMessage]> + From<[Data]>,
          S: From<[ServerMessage]> + From<[Data]>
{
    // `apply_policy` gives us deterministic control over how the 
    // the `Actor` will treat to treat the server response (and the lack of one).
    async fn apply_policy(&mut self, msg: &Self::T, policy: &Policy);

    // `get_policy` retrieves the actor's policy for processing
    // a server's reponse
    async fn get_policy(&self, msg: &Self::S) -> Policy;

    // `fmt_msg` incorporates the various 
    async fn fmt_msg(&mut self, cli_msg: &str) -> T;

   

    
}

/* In the standard library, pointer types generally do not have structural 
 * pinning, and thus they do not offer pinning projections. 
 * This is why Box<T>: Unpin holds for all T. 
 * It makes sense to do this for pointer types, because moving the Box<T> 
 * does not actually move the T: the Box<T> can be freely movable (aka Unpin) 
 * even if the T is not. 
 *
 * In fact, even Pin<Box<T>> and Pin<&mut T> are always Unpin themselves, for 
 * the same reason: their contents (the T) are pinned, but the pointers 
 * themselves can be moved without moving the pinned data. 
 *
 * For both Box<T> and Pin<Box<T>>, whether the content is pinned is entirely 
 * independent of whether the pointer is pinned, meaning pinning is not structural.
 * When implementing a Future combinator, you will usually need structural pinning 
 * for the nested futures, as you need to get pinned references to them to call poll. 
 * But if your combinator contains any other data that does not need to be pinned, 
 * you can make those fields not structural and hence freely access them with a 
 * mutable reference even when you just have Pin<&mut Self> 
 * (such as in your own poll implementation).
 */
pub enum EdgeDB {
    Some(Pin<Box<Builder>>),
    None
}

// Describes whether a single use connection
// is opened. It implicitly encodes how we should
// act: toggling whether we immediately send
// a ClientMessage to close the socket, or undergo
// some other behavior.
pub enum ConnectionStatus {
    open(ConnectionKind),
    closed
}

pub enum ConnectionKind {
    persistent,
    transient
}
pub type AsyncStatus = AsyncWeak<Duracell<ConnectionStatus>>;
pub type SyncStatus  = Weak<Cell<ConnectionStatus>>;
pub enum Status {
    Some(AsyncStatus),
    Some(SyncStatus),
    None
}

// Database
// 
// Manages the "shared state"-ness between Edgemorph
// and EdgeDB. 
//
// It scaffolds the `edgedb_client::Builder` struct
// with additonal methods for doing `edm` specific
// tasks. While this struct is useful for mocking module creation
// and performing introspection queries, its primary
// purpose is to write SIR (Serialized Introspective Representation)
// corresponding to a particular module.
pub struct Database {
    
    /* Self::db = Option<Pin<Box<Builder>>>, 
     * with
     *  Builder {
     *    addr: enum Addr {
     *            Tcp(String, u16),
     *            Unix(PathBuf)
     *    },
     *    user: String,
     *    password: String,
     *    wait: Option<std::time::Duration>,
     *    connect_timeout: std::time::Duration
     *   }>
     */
    
    db:     EdgeDB,
    status: Status
}

impl Database {
    pub fn new() -> Database {
        Database {
            db: None,
            status: None
        }
    }
    pub fn db(&mut self, builder: &Builder) -> Database {
        let Self { db, status } = self;
        Database {
            db: Some(pin_mut!(Box::new(builder.to_owned()))),
            status: status
        }   
    }
    pub fn status(&mut self, status: &Status) -> Database {
        let Self { db, status } = self;
        Database {
            db: db,
            status: status.to_owned()
        }
    }
}




// This section of the file is largely replicated from the official
// Edgedb-Cli repository. I make some minor modifications
// to suit the needs of Edgemorph.

const DEFAULT_EDGEDB_VERSION: &str = "1-alpha6";

// Can't run server on windows
#[cfg(not(windows))]
mod dump_restore;
#[cfg(not(windows))]
mod configure;
#[cfg(not(windows))]
mod non_interactive;
#[cfg(not(windows))]
mod migrations;

pub struct ShutdownInfo {
    process: process::Child,
    thread: Option<JoinHandle<()>>,
}

pub struct ServerGuard {
    port: u16,
    runstate_dir: String,
}

impl ServerGuard {
    fn start() -> ServerGuard {
        use std::process::{Command, Stdio};

        let bin_name = format!("edgedb-server-{}",
            env::var("EDGEDB_MAJOR_VERSION")
            .expect(DEFAULT_EDGEDB_VERSION));
        let mut cmd = Command::new(&bin_name);
        cmd.arg("--temp-dir");
        cmd.arg("--testmode");
        cmd.arg("--echo-runtime-info");
        cmd.arg("--port=auto");
        cmd.arg("--default-database=edgedb");
        cmd.arg("--default-database-user=edgedb");
        
        cmd.stdout(Stdio::piped());

        let mut process = cmd.spawn()
            .expect(&format!("Can run {}", bin_name));
        let process_in = process.stdout.take().expect("stdout is pipe");
        let (tx, rx) = sync_channel(1);
        let thread = thread::spawn(move || {
            let buf = BufReader::new(process_in);
            for line in buf.lines() {
                match line {
                    Ok(line) => {
                        if line.starts_with("EDGEDB_SERVER_DATA:") {
                            let data: serde_json::Value = from_str(
                                &line["EDGEDB_SERVER_DATA:".len()..])
                                .expect("valid server data");
                            println!("Server data {:?}", data);
                            let port = data.get("port")
                                .and_then(|x| x.as_u64())
                                .and_then(|x| x.try_into().ok())
                                .expect("valid server data");
                            let runstate_dir = data.get("runstate_dir")
                                .and_then(|x| x.as_str())
                                .map(|x| x.to_owned())
                                .expect("valid server data");
                            tx.send((port, runstate_dir))
                                .expect("valid channel");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from server: {}", e);
                        break;
                    }
                }
            }
        });
        let (port, runstate_dir) = rx.recv().expect("valid port received");

        let mut sinfo = SHUTDOWN_INFO.lock().expect("shutdown mutex works");
        if sinfo.is_empty() {
            shutdown_hooks::add_shutdown_hook(stop_processes);
        }
        sinfo.push(ShutdownInfo {
            process,
            thread: Some(thread),
        });

        ServerGuard {
            port,
            runstate_dir,
        }
    }

    pub fn admin_cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("edgedb").expect("binary found");
        cmd.arg("--no-version-check");
        cmd.arg("--admin");
        cmd.arg("--port").arg(self.port.to_string());
        cmd.env("EDGEDB_HOST", &self.runstate_dir);
        return cmd
    }
    
    /* Note:
     *      Interactive commands should not be necessary.
     *      Future development will require single-
     *      and multi-threaded connection persistence
     *      to maintain a lightweight communication, but
     *      for the time being these are not necessary.
     */

    #[cfg(not(windows))]
    pub fn admin_interactive(&self) -> rexpect::session::PtySession {
        use assert_cmd::cargo::CommandCargoExt;
        use rexpect::session::spawn_command;

        let mut cmd = process::Command::cargo_bin("edgedb")
            .expect("binary found");
        cmd.arg("--no-version-check");
        cmd.arg("--admin");
        cmd.arg("--port").arg(self.port.to_string());
        cmd.env("EDGEDB_HOST", &self.runstate_dir);
        return spawn_command(cmd, Some(5000)).expect("start interactive");
    }
    #[cfg(not(windows))]
    pub fn custom_interactive(&self, f: impl FnOnce(&mut process::Command))
        -> rexpect::session::PtySession
    {
        use assert_cmd::cargo::CommandCargoExt;
        use rexpect::session::spawn_command;

        let mut cmd = process::Command::cargo_bin("edgedb")
            .expect("binary found");
        cmd.arg("--no-version-check");
        cmd.arg("--admin");
        cmd.arg("--port").arg(self.port.to_string());
        cmd.env("EDGEDB_HOST", &self.runstate_dir);
        f(&mut cmd);
        return spawn_command(cmd, Some(5000)).expect("start interactive");
    }
    
    pub fn database_cmd(&self, database_name: &str) -> Command {
        let mut cmd = Command::cargo_bin("edgedb").expect("binary found");
        cmd.arg("--no-version-check");
        cmd.arg("--admin");
        cmd.arg("--port").arg(self.port.to_string());
        cmd.arg("--database").arg(database_name);
        cmd.env("EDGEDB_HOST", &self.runstate_dir);
        return cmd
    }
}


extern fn stop_processes() {
    let mut items = SHUTDOWN_INFO.lock().expect("shutdown mutex works");
    for item in items.iter_mut() {
        item.process.kill().ok();
    }
    for item in items.iter_mut() {
        item.process.wait().ok();
        item.thread.take().expect("not yet joined").join().ok();
    }
}




macro_rules! bconcat {
    ($($token: expr)*) => {
        &{
            let mut buf = ::bytes::BytesMut::new();
            $(
                buf.extend($token);
            )*
            buf
        }
    }
}

macro_rules! encoding_eq {
    ($message: expr, $bytes: expr) => {
        let data: &[u8] = $bytes;
        let mut bytes = BytesMut::new();
        $message.encode(&mut bytes)?;
        println!("Serialized bytes {:?}", bytes);
        let bytes = bytes.freeze();
        assert_eq!(&bytes[..], data);
        assert_eq!(ClientMessage::decode(&Bytes::copy_from_slice(data))?,
                   $message);
    }
}

macro_rules! map {
    ($($key:expr => $value:expr),*) => {
        {
            #[allow(unused_mut)]
            let mut h = HashMap::new();
            $(
                h.insert($key, $value);
            )*
            h
        }
    }
}

fn decode(bytes: &[u8]) -> Result<Vec<Descriptor>, DecodeError> {
    let bytes = Bytes::copy_from_slice(bytes);
    let mut cur = Cursor::new(bytes);
    let mut result = Vec::new();
    while cur.bytes() != b"" {
        result.push(Descriptor::decode(&mut cur)?);
    }
    assert!(cur.bytes() == b"");
    Ok(result)
}


// Because `credentials.json` can potentially exist
// either at the project or the $USER level,  it is
// necessary to de-couple the JSON loading from the
// edgemorph.toml lookup.

