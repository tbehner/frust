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
extern crate toml;
extern crate serde;

use regex::Regex;
use clap::{App, Arg};
use frustlib::query::Query;
use std::fs::File;
use std::io::prelude::*;
use frustlib::Config;
use std::env;

fn is_integer(inp: String) -> Result<(), String> {
    match inp.parse::<u32>() {
        Ok(_) => Ok(()),
        _     => Err(String::from("The argument does not seem to be an unsigned integer.")),
    }
}

fn check_semicolon(inp: &mut String){
    let semicolon_check = Regex::new(";\\s*$").unwrap();
    if !semicolon_check.is_match(inp) {
        inp.push(';');
    }
}

fn main() {
    let matches = App::new("frust")
        .version("0.0.2")
        .about("find alternative with SQL-like syntax.")
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
		.arg(Arg::with_name("same-device")
			 .short("s")
			 .long("same-device")
			 .help("Don't descend directories on other filesystems.")
			 .required(false)
			 .takes_value(false)
			)
		.arg(Arg::with_name("ignore-hidden")
			 .short("i")
			 .long("ignore-hidden")
			 .help("Ignore hidden files and directories.")
			 .required(false)
			 .takes_value(false)
			)
		.arg(Arg::with_name("no-color")
			 .short("c")
			 .long("no-color")
			 .help("Non colored output.")
			 .required(false)
			 .takes_value(false)
			)
		.get_matches();


    let mut config_contents = String::new();
    match env::home_dir() {
        Some(mut home_path) => {
            home_path.push(".config/frust/config.toml");
            match File::open(home_path){
                Ok(mut config_file) => {
                    config_file.read_to_string(&mut config_contents).unwrap();
                },
                Err(_) => {
                    config_contents = String::from("[color]");
                },
            }
        },
        None => {
            config_contents = String::from("[color]");
        },
    }

	let config: Config = match toml::from_str(&config_contents) {
        Ok(c) => c,
        Err(e) => panic!("Could not parse config {}", e),
    };

    let mut q = match matches.value_of("QUERY") {
        Some(query_inp) => {
            let mut query_string = String::from(query_inp);
            check_semicolon(&mut query_string);
            Query::parse(&query_string)
        },
        None => {
            let query_string = String::from("name;");
            Query::parse(&query_string)
        }
    };

    let max_depth = matches.value_of("depth").unwrap().parse::<usize>().expect("Given depth cannot be parsed to an integer!");
    let machine_mode = matches.is_present("machine-readable");
    let same_device = matches.is_present("same-device");
    let ignore_hidden = matches.is_present("ignore-hidden");
    let color = !matches.is_present("no-color");
    q.execute(max_depth, machine_mode, ignore_hidden, same_device, color, config.color);
}
