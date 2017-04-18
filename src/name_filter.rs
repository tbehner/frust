use filter::Filter;
use walkdir::DirEntry;

pub struct EqualNameFilter {
    string: String,
}

impl EqualNameFilter {
    pub fn new(string: &str) -> EqualNameFilter {
        EqualNameFilter{string: String::from(string)}
    }
}

impl Filter for EqualNameFilter {
    fn test(&self, dir_entry: &DirEntry) -> bool {
        self.string == dir_entry.path().to_str().unwrap()
    }
}

pub struct EqualBasenameFilter {
    string: String,
}

impl EqualBasenameFilter {
    pub fn new(string: &str) -> EqualBasenameFilter {
        EqualBasenameFilter{string: String::from(string)}
    }
}

impl Filter for EqualBasenameFilter {
    fn test(&self, dir_entry: &DirEntry) -> bool {
        self.string == dir_entry.file_name().to_str().unwrap()
    }
}
