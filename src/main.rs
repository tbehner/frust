extern crate clap;
extern crate walkdir;
extern crate frust;
extern crate regex;

// use regex::Regex;
use clap::{App, Arg};

use walkdir::WalkDir;
use walkdir::DirEntry;

fn main() {
    // let matches = App::new("frust")
    //     .version("0.0.1")
    //     .about("Does great stuff, in the future, hopefully...")
    //     .author("Timm Behner, Martin Clauß")
    //     .arg(Arg::with_name("QUERY")
    //          .help("find files according to the query the directory tree")
    //          .index(1)
    //          .required(false)
    //      )
    //     .get_matches();

    // let mut query = if matchs.is_present("QUERY") {
    // } else {
    //     Query::new()
    // }

	// let mut constraints = Vec::new();

    // let dir_iter = WalkDir::new().max_depth();

	// 'files: for entry in dir_iter {
    //     let entry = entry.unwrap();
    //     for constraint in &constraints {
    //         if constraint.test(&entry) != true {
    //             continue 'files;
    //         }
    //     }
    //     println!("{}", entry.path().display());
	// }
}
