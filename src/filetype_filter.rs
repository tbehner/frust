use filter::Filter;
use walkdir::DirEntry;

enum Filetype {
    File,
    Dir,
    Link,
}

pub struct FiletypeFilter {
    filetype: Filetype,
}

impl FiletypeFilter {
    pub fn new(ft_string: &str) -> FiletypeFilter {
        let t = match ft_string {
            "d" => Filetype::Dir,
            "dir" => Filetype::Dir,
            "directory" => Filetype::Dir,
            "f" => Filetype::File,
            "file" => Filetype::File,
            "l" => Filetype::Link,
            "link" => Filetype::Link,
            "slink" => Filetype::Link,
            "symlink" => Filetype::Link,
            _   => panic!("{} is not a valid filetype. Choose either of directory, link or file.", ft_string),
        };
        FiletypeFilter{filetype: t}
    }
}

impl Filter for FiletypeFilter {
    fn test(&self, dir_entry: &DirEntry) -> bool {
        match self.filetype {
            Filetype::Dir => dir_entry.metadata().unwrap().file_type().is_dir(),
            Filetype::File => dir_entry.metadata().unwrap().file_type().is_file(),
            Filetype::Link => dir_entry.metadata().unwrap().file_type().is_symlink(),
        }
    }
}
