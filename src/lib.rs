//! frustlib
//! Library for frust
//! 
//! # TODOs
//!   - [ ] '(' and ')' in filters
//!

#[macro_use]
extern crate nom;
extern crate regex;
extern crate walkdir;
extern crate env_logger;
extern crate pretty_bytes;
extern crate chrono;
extern crate mime_guess;
extern crate liquid;
extern crate colored;
extern crate termion;
extern crate libc;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod query;
pub mod filter;
pub mod parser;
pub mod formatter;
pub mod filter_tree;
pub mod regex_filter;
pub mod name_filter;
pub mod size_filter;
pub mod time_filter;
pub mod filetype_filter;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub color : Option<ColorConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ColorConfig {
    prefix: Option<String>,
    dir:  Option<String>,
    file: Option<String>,
    fifo: Option<String>,
    socket: Option<String>,
    device: Option<String>,
    symlink: Option<String>,
}


