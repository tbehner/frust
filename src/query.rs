use filter;
use filter_tree;
use ColorConfig;
use walkdir::WalkDir;
use walkdir::DirEntry;
use walkdir::WalkDirIterator;
use parser;
use nom::IResult;
use nom::Needed;
use pretty_bytes::converter::convert as pretty_bytes_convert;
use chrono::{Local, TimeZone};
use mime_guess;
use liquid;
use liquid::{Renderable, Context, Value};
use termion::{is_tty, color};
use libc;
use std::fs;
use std::fmt;
use std::time;
use std::path::Path;
use std::ffi::OsStr;
use std::os::unix::fs::FileTypeExt;
use std::os::linux::fs::MetadataExt;
use std::process::Command;

struct RgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl RgbColor {
    fn new(r: u8, g: u8, b: u8) -> RgbColor {
        RgbColor{red: r, green: g, blue: b}
    }

    fn from_str(color_string: &str) -> RgbColor {
        let red = u8::from_str_radix(&color_string[0..2], 16).expect(&format!("Could not parse red value in color {}", color_string));
        let green = u8::from_str_radix(&color_string[2..4], 16).expect(&format!("Could not parse green value in color {}", color_string));
        let blue = u8::from_str_radix(&color_string[4..6], 16).expect(&format!("Could not parse blue value in color {}", color_string));
        RgbColor{red: red, green: green, blue: blue}
    }

    fn as_color(&self) -> color::Rgb {
        color::Rgb(self.red, self.green, self.blue)
    }
}

impl fmt::Display for RgbColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rgb({}, {}, {})", self.red, self.green, self.blue)
    }
}


fn format_mimetype(t: mime_guess::Mime, machine_readable: bool) -> String {
    format!("{}", t)
}

fn stdout_is_tty() -> bool {
    let is_tty = unsafe { libc::isatty(libc::STDOUT_FILENO as i32) } != 0;
	return is_tty;
}

fn format_filetype(ft: fs::FileType, machine_readable: bool) -> String {
    if ft.is_file() {
        String::from("file")
    } else if ft.is_dir() {
        String::from("dir")
    } else if ft.is_symlink() {
        String::from("symlink")
    } else if ft.is_block_device() {
        String::from("block_device")
    } else if ft.is_fifo() {
        String::from("pipe")
    } else if ft.is_socket() {
        String::from("socket")
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

fn format_path<P: color::Color, C: color::Color>(path: &Path, parent_color: color::Fg<P>, filename_color: color::Fg<C>) -> String {
    match path.parent() {
        Some(parent) => match path.file_name() {
            Some(filename) => format!("{reset}{path_color}{path}/{filename_color}{filename}{reset}", 
                                          reset=color::Fg(color::Reset),
                                          path_color=parent_color,
                                          path=parent.to_str().unwrap(), 
                                          filename_color=filename_color,
                                          filename=filename.to_str().unwrap()),
            None => format!("{}", parent.to_str().unwrap()),
        },
        None => panic!("This should not happen!")
    }
}


fn format_name(dir_entry: &DirEntry, color_config: &Option<ColorConfig>, color_mode: bool) -> String {
    let path = dir_entry.path();
    let default_format = format!("{}", path.to_str().unwrap());
    if !color_mode {
        return default_format;
    }

    // Default configuration taken from LS_COLORS aka the output of /usr/bin/dircolors
    let file_color = color::Fg(color::Reset);
    let dir_color = color::Fg(color::Blue);
    let symlink_color = color::Fg(color::Cyan);
    let block_device_color = color::Fg(color::LightRed);
    let fifo_color = color::Fg(color::LightRed);
    let socket_color = color::Fg(color::Magenta);

    let filetype = dir_entry.file_type();
    match *color_config {
        Some(ref config) => { 
            if filetype.is_file(){
                match config.file {
                    Some(ref fc) => format_path(path, dir_color, color::Fg(RgbColor::from_str(fc).as_color())),
                    None         => format_path(path, dir_color, file_color),
                }
            } else if filetype.is_symlink() {
                match config.symlink {
                    Some(ref sc) => format_path(path, dir_color, color::Fg(RgbColor::from_str(sc).as_color())),
                    None         => format_path(path, dir_color, symlink_color),
                }
            } else if filetype.is_block_device() {
                match config.device {
                    Some(ref bc) => format_path(path, dir_color, color::Fg(RgbColor::from_str(bc).as_color())),
                    None         => format_path(path, dir_color, block_device_color),
                }
            } else if filetype.is_fifo() {
                match config.fifo {
                        Some(ref fc) => format_path(path, dir_color, color::Fg(RgbColor::from_str(fc).as_color())),
                        None         => format_path(path, dir_color, fifo_color),
                }
            } else if filetype.is_socket() {
                match config.socket {
                    Some(ref sc) => format_path(path, dir_color, color::Fg(RgbColor::from_str(sc).as_color())),
                    None         => format_path(path, dir_color, socket_color),
                }    
            } else if filetype.is_dir() {
                format!("{reset}{dircolor}{dirname}{reset}", 
                                reset=color::Fg(color::Reset),
                                dircolor=dir_color,
                                dirname=path.to_str().unwrap())
            } else {
                format!("{}", path.to_str().unwrap())
            }
        },
        None => { 
            if filetype.is_file(){
                format_path(path, dir_color, file_color)
            } else if filetype.is_symlink() {
                format_path(path, dir_color, symlink_color)
            } else if filetype.is_block_device() {
                format_path(path, dir_color, block_device_color)
            } else if filetype.is_fifo() {
                format_path(path, dir_color, fifo_color)
            } else if filetype.is_socket() {
                format_path(path, dir_color, socket_color)
            } else if filetype.is_dir() {
                format!("{reset}{dircolor}{dirname}{reset}", 
                                reset=color::Fg(color::Reset),
                                dircolor=dir_color,
                                dirname=path.to_str().unwrap())
            } else {
                format!("{}", path.to_str().unwrap())
            }        
        },
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
                filter::Attribute::Name	    => format_name(entry, color_config, color_mode),
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

    fn dev_walk(&self, dir: &String, max_depth: usize, color_config: &Option<ColorConfig>, color_mode: bool){
        let dev_id = match WalkDir::new(dir).into_iter().next() {
            Some(e) => {
                let dir_entry = e.expect("Failed to open directory entry.",);
                dir_entry.metadata().map(|m| m.st_dev()).expect(&format!("Could not get device id from {:?}.", dir_entry)) 
            }
            None => panic!("{} not found!", dir)
        };
        let dir_iter = WalkDir::new(dir)
                            .max_depth(max_depth)
                            .into_iter()
                            .filter_entry(|e| e.metadata().map(|m| m.st_dev() == dev_id).unwrap_or(false));
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


    pub fn execute(&mut self, max_depth: usize, machine_mode: bool, same_device: bool, color_config: Option<ColorConfig>) {
        let color_mode = stdout_is_tty();

        if machine_mode {
            self.machine_mode = true
        }

        for dir in &self.directories {
            if same_device {
                self.dev_walk(dir, max_depth, &color_config, color_mode);
            } else {
                self.raw_walk(dir, max_depth, &color_config, color_mode);
            }
        }
    }
}
