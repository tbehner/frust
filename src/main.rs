//! frust
//! A find replacement with SQL-like syntax.
//!
//! # TODOs
//!    - [ ] if query does not end with ';' or is empty, append a ';'
//!    - [X] implement the --depth option
//!

extern crate clap;
extern crate walkdir;
extern crate frustlib;
extern crate regex;
// #[macro_use] extern crate log;
// extern crate env_logger;

use regex::Regex;
use clap::{App, Arg};
use frustlib::query::Query;

fn is_integer(inp: String) -> Result<(), String> {
    match inp.parse::<u32>() {
        Ok(_) => Ok(()),
        _     => Err(String::from("The argument does not seem to be an unsigned integer.")),
    }
}

fn main() {
    let matches = App::new("frust")
        .version("0.0.1")
        .about("Does great stuff, in the future, hopefully...")
        .author("Timm Behner, Martin Clauß")
        .arg(Arg::with_name("QUERY")
             .help("find files according to the query the directory tree")
             .index(1)
             .required(false)
         )
        .arg(Arg::with_name("depth")
             .short("d")
             .long("depth")
             .help("maximum depth of search")
             .required(false)
             .takes_value(true)
             .value_name("DEPTH")
             .default_value("4096") // current maximum directory tree depth on my linux machine
             .validator(is_integer)
         )
        .arg(Arg::with_name("machine-readable")
             .short("m")
             .long("machine-readable")
             .help("print attributes in machine readable syntax")
             .required(false)
             .takes_value(false)
        )
        .get_matches();

    let mut q = if matches.is_present("QUERY") {
        let mut query_string = String::from(matches.value_of("QUERY").unwrap());
        let semicolon_check = Regex::new(";\\s*$").unwrap();
        if !semicolon_check.is_match(&query_string) {
            query_string.push(';');
        }
        Query::parse(&query_string)
    } else {
        panic!("frust without a query is not yet supported!");
    };

    let max_depth = matches.value_of("depth").unwrap().parse::<usize>().unwrap();
    let machine_mode = matches.is_present("machine-readable");
    q.execute(max_depth, machine_mode);
}
