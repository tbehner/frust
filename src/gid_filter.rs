use filter;
use filter::Filter;
use walkdir::DirEntry;
use std::os::unix::fs::MetadataExt;
use std::process;

pub struct GidFilter {
    gid: u32,
    comp_op: filter::CompOp,
}

impl GidFilter {
    pub fn new(comp_op: filter::CompOp, gid: u32) -> GidFilter {
        GidFilter{comp_op: comp_op, gid: gid}
    }
}

impl Filter for GidFilter {
    fn test(&self, dir_entry: &DirEntry) -> bool {
        match self.comp_op {
            filter::CompOp::Equal => self.gid == dir_entry.metadata().unwrap().gid(),
            filter::CompOp::Unequal => self.gid != dir_entry.metadata().unwrap().gid(),
            _                       => {
                eprintln!("Operator {:?} not covered for attribute gid!", self.comp_op);
                process::exit(1);
            },
        }
    }
}
