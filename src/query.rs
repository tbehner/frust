use filter;
use filter_tree;
use walkdir::WalkDir;
use walkdir::DirEntry;
use parser;
use nom::IResult;
use nom::Needed;
use std::fs;
use pretty_bytes::converter::convert as pretty_bytes_convert;
use std::time;
use chrono::{Local, TimeZone};
use mime_guess;
use std::process::Command;
use liquid;
use liquid::{Renderable, Context, Value};

fn format_mimetype(t: mime_guess::Mime, machine_readable: bool) -> String {
    format!("{}", t)
}

fn format_filetype(ft: fs::FileType, machine_readable: bool) -> String {
    if ft.is_file() {
        String::from("file")
    } else if ft.is_dir() {
        String::from("dir")
    } else if ft.is_symlink() {
        String::from("slink")
    } else {
        String::from("unkown")
    }
}

fn format_filesize(size: u64, machine_readable: bool) -> String {
    if machine_readable {
        return format!("{}", size)
    } else {
        pretty_bytes_convert(size as f64)
    }
}

fn format_systime(t: time::SystemTime, machine_readable: bool) -> String {
    let duration = t.duration_since(time::UNIX_EPOCH).unwrap();
    if machine_readable {
        format!("{}", duration.as_secs())
    } else {
       format!("{}", Local.timestamp(duration.as_secs() as i64, 0).format("%F %T"))
    }
}

pub struct Query {
    attributes: Vec<filter::Attribute>,
    directories: Vec<String>,
    filters: filter_tree::FilterTree,
    command: Option<String>,
    machine_mode: bool,
}

impl Query {
    pub fn new(attributes: Option<Vec<filter::Attribute>>, directories: Option<Vec<String>>, filters: Option<filter_tree::FilterTree>, command: Option<String>) -> Query{
        let mut attr = attributes.unwrap_or(vec![filter::Attribute::Name]);
        if attr.len() < 1 {
            attr.push(filter::Attribute::Name);
        }
        let dirs = directories.unwrap_or(vec![String::from(".")]);
        let filters = filters.unwrap_or(filter_tree::FilterTree::new(None, None, None));
        Query{attributes: attr, directories: dirs, filters: filters, command: command, machine_mode: false}
    }

    pub fn parse(inp: &str) -> Query {
        match parser::query(inp.as_bytes()){
            IResult::Done(leftovers, q)     => {
                if leftovers.len() > 1 {
                    panic!("Could not parse from here -> {}\nDid you make a typo?\n\n", String::from_utf8_lossy(leftovers).into_owned());
                };
                q
            },
            IResult::Error(e)               => panic!("Syntax error {}", e),
            IResult::Incomplete(n)          => match n {
                    Needed::Unknown         => panic!("Need more input, but I haven't got a clou how much!"),
                    Needed::Size(n)         => panic!("Need {}bytes more input!", n),
                },
        }
    }

    fn print_attributes(&self, entry: &DirEntry) {
        let mut print_string = String::from("");
        for attribute in &self.attributes {
            let attr_str = match *attribute {
                filter::Attribute::Name	    => format!("{}", entry.path().display()),
                filter::Attribute::Basename	=> format!("{}", entry.file_name().to_str().unwrap()),
                filter::Attribute::Size	    => format_filesize(entry.metadata().unwrap().len(), self.machine_mode),
                filter::Attribute::Mtime	=> format_systime(entry.metadata().unwrap().modified().unwrap(), self.machine_mode),
                filter::Attribute::Ctime	=> {
                    match entry.metadata().unwrap().created() {
                        Ok(t)   => format_systime(t, self.machine_mode),
                        Err(_)  => String::from("N/A"),
                    }},
                filter::Attribute::Atime    => {
                    match entry.metadata().unwrap().accessed() {
                        Ok(t)   => format_systime(t, self.machine_mode),
                        Err(_)  => String::from("N/A"),
                    }},                
                filter::Attribute::Filetype	=> format_filetype(entry.metadata().unwrap().file_type(), self.machine_mode),
                filter::Attribute::Mimetype	=> {
                    let filepath = format!("{}", entry.path().display());
                    format_mimetype(mime_guess::guess_mime_type(filepath), self.machine_mode)
                },
                filter::Attribute::Inode	=> format!("{}", entry.ino()),
            };
            if !print_string.is_empty() {
                print_string.push(',');
            }
            print_string.push_str(attr_str.as_str());
        }
        println!("{}", print_string);
    }

    fn setup_context(&self, dir_entry: &DirEntry) -> Context {
        // TODO does this need speed improvement? All attributes will be needed rarely...
        let mut context = Context::new();
        context.set_val("name", Value::Str(String::from(dir_entry.path().to_str().unwrap())));
        context.set_val("basename", Value::Str(String::from(dir_entry.file_name().to_str().unwrap())));
        context.set_val("size", Value::Str(format_filesize(dir_entry.metadata().unwrap().len(), self.machine_mode)));
        context.set_val("mtime", Value::Str(format_systime(dir_entry.metadata().unwrap().modified().unwrap(), self.machine_mode)));
        let ctime = match dir_entry.metadata().unwrap().created() {
                Ok(t)   => format_systime(t, self.machine_mode),
                Err(_)  => String::from("N/A"),
        };
        context.set_val("ctime", Value::Str(ctime));

        let atime = match dir_entry.metadata().unwrap().accessed() {
                Ok(t)   => format_systime(t, self.machine_mode),
                Err(_)  => String::from("N/A"),
            };
        context.set_val("atime", Value::Str(atime));

        context.set_val("filetype", Value::Str(format_filetype(dir_entry.metadata().unwrap().file_type(), self.machine_mode)));
        context.set_val("mimetype", Value::Str(format_mimetype(mime_guess::guess_mime_type(dir_entry.path().to_str().unwrap()), self.machine_mode)));
        context.set_val("inode", Value::Str(format!("{}", dir_entry.ino())));
        return context;
    }

    fn run_command(&self, dir_entry: &DirEntry) {
        match self.command {
            None    => {},
            Some(ref c) => { 
                let template = liquid::parse(c, Default::default()).unwrap();
                let mut context = self.setup_context(dir_entry);
                let output = template.render(&mut context);
                match output {
                    Ok(res) => Command::new("sh").arg("-c").arg(res.unwrap()).spawn().expect("Failed to start command."),
                    Err(e)    => panic!("Command template error: {}", e),
                };
            },
        }
    }

    pub fn execute(&mut self, max_depth: usize, machine_mode: bool) {
        if machine_mode {
            self.machine_mode = true
        }

        for dir in &self.directories {
            let dir_iter = WalkDir::new(dir).max_depth(max_depth);

            'files: for entry in dir_iter {
                let entry = match entry{
                    Ok(e)  => e,
                    Err(e) => {
                        println!("Error: {}", e);
                        continue 'files;
                    }
                };
                if self.filters.test(&entry) != true {
                        continue 'files;
                }
                self.print_attributes(&entry);
                self.run_command(&entry);
            }
        }
    }
}
