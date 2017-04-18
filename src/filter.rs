use walkdir::DirEntry;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum CompOp {
    Lower,
    LowerEqual,
    Equal,
    GreaterEqual,
    Greater,
    Like,
}

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
}

pub trait Filter{
    fn test(&self, dir_entry: &DirEntry) -> bool;
}
