use std::fmt;
use std::cmp;

pub struct Filter {
    attribute: String,
    op       : String,
    param    : String,
}

impl Filter {
    pub fn new(attribute: String, op: String, param: String) -> Filter {
        return Filter{attribute: attribute, op: op, param: param};
    }
}

impl fmt::Debug for Filter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "Filter{{ attribute: {}, op: {}, param: {} }}", self.attribute, self.op, self.param)
    }
}

impl cmp::PartialEq for Filter {
    fn eq(&self, other: &Filter) -> bool {
        self.attribute == other.attribute && self.op == other.op && self.param == other.param
    }
}
