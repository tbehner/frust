use filter::Filter;
use walkdir::DirEntry;
use regex::Regex;

pub struct RegexNameFilter {
    regex    : Regex,
}

impl RegexNameFilter {
    pub fn new(param: &str) -> RegexNameFilter {
        let re = match Regex::new(param) {
            Ok(r) => r,
            Err(e) => panic!("Regex error {}", e),
        };
        RegexNameFilter{regex: re}
    }
}

impl Filter for RegexNameFilter {
    fn test(&self, dir_entry: &DirEntry) -> bool {
        self.regex.is_match(dir_entry.path().to_str().unwrap())
    }
}

pub struct RegexBasenameFilter {
    regex    : Regex,
}

impl RegexBasenameFilter {
    pub fn new(param: &str) -> RegexBasenameFilter {
        let re = match Regex::new(param){
            Ok(r) => r,
            Err(e) => panic!("Regex error {}", e),
        };
        RegexBasenameFilter{regex: re}
    }
}

impl Filter for RegexBasenameFilter {
    fn test(&self, dir_entry: &DirEntry) -> bool {
        self.regex.is_match(dir_entry.file_name().to_str().unwrap())
    }
}

