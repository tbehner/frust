use filter;
use filter::Filter;
use walkdir::DirEntry;
use std::os::unix::fs::MetadataExt;
use std::process;

pub struct UidFilter {
    uid: u32,
    comp_op: filter::CompOp,
}

impl UidFilter {
    pub fn new(comp_op: filter::CompOp, uid: u32) -> UidFilter {
        UidFilter{comp_op: comp_op, uid: uid}
    }
}

impl Filter for UidFilter {
    fn test(&self, dir_entry: &DirEntry) -> bool {
        match self.comp_op {
            filter::CompOp::Equal => self.uid == dir_entry.metadata().unwrap().uid(),
            filter::CompOp::Unequal => self.uid != dir_entry.metadata().unwrap().uid(),
            _                       => {
                eprintln!("Operator {:?} not covered for attribute uid!", self.comp_op);
                process::exit(1);
            },
        }
    }
}
