use filter;
use std::fmt;
use std::cmp;

pub struct Query {
    attributes: Vec<String>,
    directories: Vec<String>,
    filters: Vec<filter::Filter>,
    command: String,
}

impl Query {
    pub fn new(attributes: Option<Vec<String>>, directories: Option<Vec<String>>, filters: Option<Vec<filter::Filter>>, command: Option<String>) -> Query{
        let attr = attributes.unwrap_or(vec![String::from("name")]);
        let dirs = directories.unwrap_or(vec![String::from(".")]);
        let filters = filters.unwrap_or(Vec::new());
        let command = command.unwrap_or(String::from(""));
        Query{attributes: attr, directories: dirs, filters: filters, command: command}
    }
}

impl fmt::Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "Query{{ {:?} from {:?} where {:?} exec {:?} }}", self.attributes, self.directories, self.filters, self.command)
    }
}

impl cmp::PartialEq for Query {
    fn eq(&self, other: &Query) -> bool {
        self.attributes == other.attributes && self.directories == other.directories && self.filters == other.filters
    }
}
