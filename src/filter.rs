use walkdir::DirEntry;

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum CompOp {
    Lower,
    LowerEqual,
    Equal,
    Unequal,
    GreaterEqual,
    Greater,
    Like,
    Unlike,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Attribute {
    Name,
    Size,
    Mtime,
    Ctime,
    Atime,
    Filetype,
    Mimetype,
    Inode,
    Basename,
    Uid,
    Gid,
}

pub trait Filter{
    fn test(&self, dir_entry: &DirEntry) -> bool;
}
