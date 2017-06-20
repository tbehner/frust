use filter;
use filter::Filter;
use filter_tree::FilterTuple;
use walkdir::DirEntry;
use regex::Regex;
use std::process;

pub struct RegexFilter {
    attribute : filter::Attribute,
    regex     : Regex,
    flip      : bool,
}

impl RegexFilter {
    pub fn new(ft: &FilterTuple) -> RegexFilter {
        let re = match Regex::new(&ft.parameter) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Regex error {}", e);
                process::exit(1);
            },
        };
        let flip = match ft.operator {
            filter::CompOp::Unlike => true,
            _ => false,
        };
        RegexFilter{regex: re, attribute: ft.attribute.clone(), flip: flip}
    }

    fn get_attribute<'a>(&self, direntry: &'a DirEntry) -> Option<&'a str> {
        match self.attribute {
            filter::Attribute::Name => direntry.path().to_str(),
            filter::Attribute::Basename => direntry.file_name().to_str(),
            _ => {
                eprintln!("Operator ~ not supported for attribute {:?}", self.attribute);
                process::exit(1);
            },
        }
    }
}

impl Filter for RegexFilter {
    fn test(&self, dir_entry: &DirEntry) -> bool {
        let attr = self.get_attribute(dir_entry);
        if attr.is_none() {
            eprintln!("UTF-8 Error");
            return false;
        }
        if !self.flip {self.regex.is_match(attr.unwrap())} else {!self.regex.is_match(attr.unwrap())}
    }
}
