use filter;
use filter::Filter;
use filter_tree::FilterTuple;
use walkdir::DirEntry;
use regex::Regex;

pub struct RegexFilter {
    attribute : filter::Attribute,
    regex     : Regex,
    flip      : bool,
}

impl RegexFilter {
    pub fn new(ft: &FilterTuple) -> RegexFilter {
        println!("Got operator {:?}", ft.operator);
        println!("Got parameter {}", ft.parameter);
        let re = match Regex::new(&ft.parameter) {
            Ok(r) => r,
            Err(e) => panic!("Regex error {}", e),
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
            _ => panic!("Operator ~ not supported for attribute {:?}", self.attribute),
        }
    }
}

impl Filter for RegexFilter {
    fn test(&self, dir_entry: &DirEntry) -> bool {
        let attr = self.get_attribute(dir_entry);
        if attr.is_none() {
            println!("UTF-8 Error");
            return false;
        }
        if !self.flip {self.regex.is_match(attr.unwrap())} else {!self.regex.is_match(attr.unwrap())}
    }
}
