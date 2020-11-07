/* Large portions of this file were originally published
 * by contributors/developers at [EdgeDB](https://github.com/edgedb).
 * 
 * I own nothing.
 */
#[cfg(not(windows))]
#[macro_use] extern crate pretty_assertions;

use std::sync::Mutex;
use std::convert::TryInto;
use std::io::{BufReader, BufRead};
use std::sync::mpsc::sync_channel;
use std::thread::{self, JoinHandle};
use std::process;
use std::env;
use assert_cmd::Command;
use serde::{Deserialize};
use serde_json::{Result, from_str};
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

/*
use std::collections::HashMap;
use std::error::Error;

use uuid::Uuid;
use bytes::{Bytes, BytesMut};

use edgedb_protocol::server_message::{ServerMessage};
use edgedb_protocol::server_message::{ServerHandshake};
use edgedb_protocol::server_message::{ErrorResponse, ErrorSeverity};
use edgedb_protocol::server_message::{ReadyForCommand, TransactionState};
use edgedb_protocol::server_message::{ServerKeyData, ParameterStatus};
use edgedb_protocol::server_message::{CommandComplete};
use edgedb_protocol::server_message::{PrepareComplete, Cardinality};
use edgedb_protocol::server_message::{CommandDataDescription, Data};
use edgedb_protocol::server_message::{Authentication};
use edgedb_protocol::server_message::{LogMessage, MessageSeverity};
use edgedb_protocol::server_message::{RestoreReady};

use edgedb_protocol::client_message::{ClientMessage, ClientHandshake};
use edgedb_protocol::client_message::{ExecuteScript, Execute};
use edgedb_protocol::client_message::{Prepare, IoFormat, Cardinality};
use edgedb_protocol::client_message::{DescribeStatement, DescribeAspect};
use edgedb_protocol::client_message::{SaslInitialResponse};
use edgedb_protocol::client_message::{SaslResponse};
use edgedb_protocol::client_message::Restore;


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
*/

// Because `credentials.json` can potentially exist
// either at the project or the $USER level,  it is
// necessary to de-couple the JSON loading from the
// edgemorph.toml lookup.

