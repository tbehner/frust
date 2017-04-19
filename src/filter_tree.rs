use filter::Filter;
use filter::CompOp;
use filter::Attribute;
use regex_filter::RegexFilter;
use name_filter::EqualNameFilter;
use name_filter::EqualBasenameFilter;
use size_filter::SizeFilter;
use time_filter::TimeFilter;
use filetype_filter::FiletypeFilter;
use walkdir::DirEntry;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum LogicOp {
    And,
    Or,
}

pub struct FilterTuple {
    pub attribute: Attribute,
    pub operator : CompOp,
    pub parameter: String,
}

impl FilterTuple {
    pub fn new(attribute: Attribute, operator: CompOp, parameter: String) -> FilterTuple {
        FilterTuple{attribute: attribute, operator: operator, parameter: parameter}
    }
}

pub fn create_filter(inp: FilterTuple) -> Box<Filter> {
    match inp.attribute {
        Attribute::Name => {
            match inp.operator {
                CompOp::Equal => Box::new(EqualNameFilter::new(inp.parameter.as_str())),
                CompOp::Like => Box::new(RegexFilter::new(&inp)),
                CompOp::Unlike => Box::new(RegexFilter::new(&inp)),
                _            => panic!("Operator is not implemented for attribute."),
            }
        },
        Attribute::Basename => {
            match inp.operator {
                CompOp::Equal => Box::new(EqualBasenameFilter::new(inp.parameter.as_str())),
                CompOp::Like => Box::new(RegexFilter::new(&inp)),
                CompOp::Unlike => Box::new(RegexFilter::new(&inp)),
                _            => panic!("Operator is not implemented for attribute."),
            }
        },
        Attribute::Size => {
            Box::new(SizeFilter::new(inp.operator, inp.parameter.as_str()))
        },
        Attribute::Mtime => {
            Box::new(TimeFilter::new(inp.attribute, inp.operator, inp.parameter.as_str()))
        },
        Attribute::Atime => {
            Box::new(TimeFilter::new(inp.attribute, inp.operator, inp.parameter.as_str()))
        },
        Attribute::Ctime => {
            Box::new(TimeFilter::new(inp.attribute, inp.operator, inp.parameter.as_str()))
        },
        Attribute::Filetype => {
            Box::new(FiletypeFilter::new(inp.parameter.as_str()))
        },
        _               => panic!("Not yet implemented!"),
    }
}

pub struct FilterTree {
    lhs: Option<Box<Filter>>,
    lop: Option<LogicOp>,
    rhs: Option<Box<FilterTree>>,
}

impl FilterTree {
    pub fn new(lhs: Option<FilterTuple>, op: Option<LogicOp>, rhs: Option<Box<FilterTree>>) -> FilterTree {
        if lhs.is_none() {
            if op.is_some() || rhs.is_some() {
                panic!("Cannot have an expression without a left hand side!");
            }
            return FilterTree{lhs: None, lop: None, rhs: None};
        }
        if op.is_none() && rhs.is_some() {
            panic!("Two logic expressions have to be connected by an logic operator!");
        }
        if op.is_some() && rhs.is_none() {
            panic!("Right hand side is missing!");
        }
        FilterTree{lhs: Some(create_filter(lhs.unwrap())), lop: op, rhs: rhs}
    }

    pub fn test(&self, dir_entry: &DirEntry) -> bool {
        if self.lhs.as_ref().is_none() {
            return true
        }
        match self.lop {
            Some(ref op) => {
                let rhs = self.rhs.as_ref().unwrap();
                let lhs = self.lhs.as_ref().unwrap();
                match *op {
                    LogicOp::And => lhs.test(dir_entry) && rhs.test(dir_entry),
                    LogicOp::Or  => lhs.test(dir_entry) || rhs.test(dir_entry),
                }
            },
            None => self.lhs.as_ref().unwrap().test(dir_entry),
        }
    }
}
