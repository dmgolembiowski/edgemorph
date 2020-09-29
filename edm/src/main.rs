use clap::{App, Arg};
use std::env;

fn main() {
    let matches = App::new("edm")
        .version("0.1.0")
        .author("David G. <david@dgolembiowski.com>")
        .about("A command-line utility for Edgemorph \
               (EdgeDB Manipulator of Relational Polymorphic Hierarchies)")
        .arg(
            Arg::new("init")
            .short('i')
            .long("init")
            .default_value(
                env::current_dir()
                    .unwrap()
                    .expect()
                    .to_owned()
            )
            /* Create a folder `<dir>/edmodules`
               Create inital config `<dir>/edgemorph.toml`
             */
            .about("Starts a new edgemorph project in the specified directory.")
            .takes_value(true)
        )
        .subcommand(
            Arg::new("make")
            .about("Parses each `.edgeql` file in the `modules_dir` \
                unless a single EdgeQL module file name is supplied")
            .subcommand(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .about("The module filename without any extensions")
                    .value_name("FILE")
                    .default_value("*")
            )
            .subcommand(
                Arg::new("install")
                    .about("Compile the EdgeDB modules and test connectivity")
            )
        )
}
