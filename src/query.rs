use filter;
use filter_tree;
use ColorConfig;
use formatter;
use walkdir;
use walkdir::WalkDir;
use walkdir::DirEntry;
use walkdir::WalkDirIterator;
use parser;
use nom::IResult;
use nom::Needed;
use mime_guess;
use liquid;
use liquid::{Renderable, Context, Value};
use termion::{is_tty};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::process::Command;

fn stdout_is_tty() -> bool {
    is_tty(&fs::File::create("/dev/stdout").unwrap())
}

pub struct Query {
    attributes: Vec<filter::Attribute>,
    directories: Vec<String>,
    filters: filter_tree::FilterTree,
    command: Option<String>,
    machine_mode: bool,
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.path()
         .to_str()
         .map(|s| s.contains("/."))
         .unwrap_or(false)
}

impl Query {
    pub fn new(attributes: Option<Vec<filter::Attribute>>, directories: Option<Vec<String>>, filters: Option<filter_tree::FilterTree>, command: Option<String>) -> Query{
        let mut attr = attributes.unwrap_or(vec![filter::Attribute::Name]);
        let dirs = directories.unwrap_or(vec![String::from(".")]);
        let filters = filters.unwrap_or(filter_tree::FilterTree::new(None, None, None));
        if attr.len() == 0 {
            attr.push(filter::Attribute::Name);
        }
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

    fn print_attributes(&self, entry: &DirEntry, color_config: &Option<ColorConfig>, color_mode: bool) {
        let mut print_string = String::from("");
        for attribute in &self.attributes {
            let attr_str = match *attribute {
                filter::Attribute::Name	    => formatter::format_name(entry, color_config, color_mode),
                filter::Attribute::Basename	=> format!("{}", entry.file_name().to_str().unwrap()),
                filter::Attribute::Size	    => formatter::format_filesize(entry.metadata().unwrap().len(), self.machine_mode),
                filter::Attribute::Mtime	=> formatter::format_systime(entry.metadata().unwrap().modified().unwrap(), self.machine_mode),
                filter::Attribute::Ctime	=> {
                    match entry.metadata().unwrap().created() {
                        Ok(t)   => formatter::format_systime(t, self.machine_mode),
                        Err(_)  => String::from("N/A"),
                    }},
                filter::Attribute::Atime    => {
                    match entry.metadata().unwrap().accessed() {
                        Ok(t)   => formatter::format_systime(t, self.machine_mode),
                        Err(_)  => String::from("N/A"),
                    }},                
                filter::Attribute::Filetype	=> formatter::format_filetype(entry.metadata().unwrap().file_type(), self.machine_mode),
                filter::Attribute::Mimetype	=> {
                    let filepath = format!("{}", entry.path().display());
                    formatter::format_mimetype(mime_guess::guess_mime_type(filepath), self.machine_mode)
                },
                filter::Attribute::Inode	=> format!("{}", entry.ino()),
                filter::Attribute::Uid 	    => format!("{}", entry.metadata().unwrap().uid()),
                filter::Attribute::Gid 	    => format!("{}", entry.metadata().unwrap().gid()),
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
        context.set_val("size", Value::Str(formatter::format_filesize(dir_entry.metadata().unwrap().len(), self.machine_mode)));
        context.set_val("mtime", Value::Str(formatter::format_systime(dir_entry.metadata().unwrap().modified().unwrap(), self.machine_mode)));
        let ctime = match dir_entry.metadata().unwrap().created() {
                Ok(t)   => formatter::format_systime(t, self.machine_mode),
                Err(_)  => String::from("N/A"),
        };
        context.set_val("ctime", Value::Str(ctime));

        let atime = match dir_entry.metadata().unwrap().accessed() {
                Ok(t)   => formatter::format_systime(t, self.machine_mode),
                Err(_)  => String::from("N/A"),
            };
        context.set_val("atime", Value::Str(atime));

        context.set_val("type", Value::Str(formatter::format_filetype(dir_entry.metadata().unwrap().file_type(), self.machine_mode)));
        context.set_val("mimetype", Value::Str(formatter::format_mimetype(mime_guess::guess_mime_type(dir_entry.path().to_str().unwrap()), self.machine_mode)));
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

    fn raw_walk(&self, dir: &String, max_depth: usize, color_config: &Option<ColorConfig>, color_mode: bool) {
        let dir_iter = WalkDir::new(dir).max_depth(max_depth).into_iter();

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
            self.print_attributes(&entry, color_config, color_mode);
            self.run_command(&entry);
        }
    }

    fn process_entry(&self, entry: Result<DirEntry, walkdir::Error>, color_config: &Option<ColorConfig>, color_mode: bool){
        let entry = match entry{
            Ok(e)  => e,
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        };
        if self.filters.test(&entry) == true {
            self.print_attributes(&entry, color_config, color_mode);
            self.run_command(&entry);
        }
    }

    fn dev_walk(&self, dir: &String, max_depth: usize, color_config: &Option<ColorConfig>, color_mode: bool, same_device: bool, ignore_hidden: bool){
        let dir_iter = WalkDir::new(dir).max_depth(max_depth).into_iter();

        let dev_id = match WalkDir::new(dir).into_iter().next() {
            Some(e) => {
                let dir_entry = e.expect("Failed to open directory entry.",);
                dir_entry.metadata().map(|m| m.dev()).expect(&format!("Could not get device id from {:?}.", dir_entry)) 
            }
            None => panic!("{} not found!", dir)
        };


        if same_device && ignore_hidden {
            let filtered_iter = dir_iter.filter_entry(|e| e.metadata().map(|m| m.dev() == dev_id).unwrap_or(false))
                                        .filter_entry(|e| !is_hidden(e)); 
            for entry in filtered_iter {
                self.process_entry(entry, color_config, color_mode);
            }
        } else if same_device {
            let filtered_iter = dir_iter.filter_entry(|e| e.metadata().map(|m| m.dev() == dev_id).unwrap_or(false));
            for entry in filtered_iter {
                self.process_entry(entry, color_config, color_mode);
            }
        } else if ignore_hidden {
            let filtered_iter = dir_iter.filter_entry(|e| !is_hidden(e));
            for entry in filtered_iter {
                self.process_entry(entry, color_config, color_mode);
            }
        } else {
            panic!("Implementation error: Use raw_walk instead!");
        }
    }


    pub fn execute(&mut self, max_depth: usize, machine_mode: bool, ignore_hidden: bool, same_device: bool, color: bool, color_config: Option<ColorConfig>) {
        let color_mode = if color {
            stdout_is_tty()
        } else {
            false
        };

        if machine_mode {
            self.machine_mode = true
        }

        for dir in &self.directories {
            if same_device || ignore_hidden {
                self.dev_walk(dir, max_depth, &color_config, color_mode, same_device, ignore_hidden);
            } else {
                self.raw_walk(dir, max_depth, &color_config, color_mode);
            }
        }
    }
}
