use walkdir::DirEntry;
use termion::{is_tty, color};
use std::path::Path;
use std::fmt;
use mime_guess;
use std::fs;
use pretty_bytes::converter::convert as pretty_bytes_convert;
use std::time;
use chrono::{Local, TimeZone};
use ColorConfig;
use std::ffi::OsStr;
use std::os::unix::fs::FileTypeExt;
use std::os::linux::fs::MetadataExt;

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

pub fn format_mimetype(t: mime_guess::Mime, machine_readable: bool) -> String {
    format!("{}", t)
}

pub fn format_filetype(ft: fs::FileType, machine_readable: bool) -> String {
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

pub fn format_filesize(size: u64, machine_readable: bool) -> String {
    if machine_readable {
        return format!("{}", size)
    } else {
        pretty_bytes_convert(size as f64)
    }
}

pub fn format_systime(t: time::SystemTime, machine_readable: bool) -> String {
    let duration = t.duration_since(time::UNIX_EPOCH).unwrap();
    if machine_readable {
        format!("{}", duration.as_secs())
    } else {
       format!("{}", Local.timestamp(duration.as_secs() as i64, 0).format("%F %T"))
    }
}

pub fn format_path<P: color::Color, C: color::Color>(path: &Path, parent_color: color::Fg<P>, filename_color: color::Fg<C>) -> String {
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

pub fn format_dir<P: color::Color>(path: &Path, dir_color: color::Fg<P>) -> String {
    format!("{reset}{dircolor}{dirname}{reset}", 
                                reset=color::Fg(color::Reset),
                                dircolor=dir_color,
                                dirname=path.to_str().unwrap())
}



pub fn format_name(dir_entry: &DirEntry, color_config: &Option<ColorConfig>, color_mode: bool) -> String {
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
        Some(ref config) => match config.prefix {
            Some(ref pre) => { 
                if filetype.is_file(){
                    match config.file {
                        Some(ref fc) => format_path(path, color::Fg(RgbColor::from_str(pre).as_color()), color::Fg(RgbColor::from_str(fc).as_color())),
                        None         => format_path(path, color::Fg(RgbColor::from_str(pre).as_color()), file_color),
                    }
                } else if filetype.is_symlink() {
                    match config.symlink {
                        Some(ref sc) => format_path(path, color::Fg(RgbColor::from_str(pre).as_color()), color::Fg(RgbColor::from_str(sc).as_color())),
                        None         => format_path(path, color::Fg(RgbColor::from_str(pre).as_color()), symlink_color),
                    }
                } else if filetype.is_block_device() {
                    match config.device {
                        Some(ref bc) => format_path(path, color::Fg(RgbColor::from_str(pre).as_color()), color::Fg(RgbColor::from_str(bc).as_color())),
                        None         => format_path(path, color::Fg(RgbColor::from_str(pre).as_color()), block_device_color),
                    }
                } else if filetype.is_fifo() {
                    match config.fifo {
                            Some(ref fc) => format_path(path, color::Fg(RgbColor::from_str(pre).as_color()), color::Fg(RgbColor::from_str(fc).as_color())),
                            None         => format_path(path, color::Fg(RgbColor::from_str(pre).as_color()), fifo_color),
                    }
                } else if filetype.is_socket() {
                    match config.socket {
                        Some(ref sc) => format_path(path, color::Fg(RgbColor::from_str(pre).as_color()), color::Fg(RgbColor::from_str(sc).as_color())),
                        None         => format_path(path, color::Fg(RgbColor::from_str(pre).as_color()), socket_color),
                    }    
                } else if filetype.is_dir() {
                    match config.dir {
                        Some(ref dc) => format_dir(path, color::Fg(RgbColor::from_str(dc).as_color())),
                        None => format_dir(path, dir_color),
                    }
                } else {
                    format!("{}", path.to_str().unwrap())
                }
            },
            None         => { 
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
                    match config.dir {
                        Some(ref dc) => format_dir(path, color::Fg(RgbColor::from_str(dc).as_color())),
                        None => format_dir(path, dir_color),
                    }
                } else {
                    format!("{}", path.to_str().unwrap())
                }
            },
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
                format_dir(path, dir_color)
            } else {
                format!("{}", path.to_str().unwrap())
            }        
        },
    }
}
