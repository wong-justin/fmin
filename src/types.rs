use std::path::PathBuf;
use std::collections::HashSet;
use std::fs::DirEntry;
use std::marker::PhantomData;
use std::fmt::{Display, Formatter, Error};
use std::cmp::Ordering;

use chrono::{DateTime, TimeZone, Local};
use byte_unit::Byte;

#[derive(PartialEq, Clone)]
pub struct FileSize(u64);
#[derive(PartialEq, Clone)]
pub struct FileName(String);
#[derive(PartialEq, Clone)]
pub struct FileDate(DateTime<Local>);

pub enum FileProperty {
    Name,
    Size,
    Date,
}

#[derive(PartialEq, Clone)]
pub struct Entry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub name: FileName,
    pub size: FileSize,
    pub modified: FileDate,
}

impl std::convert::From<DirEntry> for Entry {
    fn from(de: DirEntry) -> Self {
        let p = de.path();
        let metadata = de.metadata().unwrap();

        let mut name : String = p.file_name().unwrap().to_str().unwrap().to_string();
        let is_dir = p.is_dir();
        /*
        let byte_size = match is_dir {
            true => Some(FileSize(metadata.len())),
            false => None,
        };
        */
        if is_dir {
            name = format!("{}/", name);
        }

        let date = DateTime::<Local>::from(metadata.modified().unwrap());
        
        return Self {
            path: p,
            is_dir: is_dir, 
            name: FileName(name),
            size: FileSize(metadata.len()),
            modified: FileDate(date),
        };
    }
}

fn str_width<S: AsRef<str>> (s: S) -> usize {
    return s.as_ref().chars().count();
}


impl Display for FileSize {
    // "999.99 GB"  "1 B"
    // max 9 chars, min 3
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        // write!(f, "{}B", self.0)
        write!(f, "{}", Byte::from_bytes(self.0.into()).get_appropriate_unit(false).to_string())
    }
}

impl Display for FileName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl FileName {
    pub fn fit(&self, nchars: usize) -> String {
        match str_width(&self.0) {
            n if n <= nchars => {
                let spaces = " ".repeat(nchars - n);
                return format!("{}{}", self.0, spaces)
            },
            n => {
                let prefix = "...";
                let prefix_len = str_width(prefix);
                let discard_len : usize = (n - nchars) + prefix_len; 
                let suffix : String = self.0.chars().skip(discard_len).take(nchars - prefix_len).collect::<String>();
                return format!("{}{}", prefix, suffix);
            }
        }
    }
}

impl FileDate {
    pub fn fit(&self, nchars: usize) -> String {
        let fullstr = self.0.format("%-m/%-d/%y");
        return format!("{: <width$}", fullstr, width=nchars);
    }
}

impl FileSize {
    pub fn fit(&self, nchars: usize) -> String {
        let fullstr = Byte::from_bytes(self.0.into()).get_appropriate_unit(false).to_string();
        return format!("{: <width$}", fullstr, width=nchars);
    }
}





impl Display for FileDate {
    // "10/10/10 10:10 PM"  "1/01/01 1:01 AM"
    // max 17 char widths, min 15
    //
    // "10/10/10" max 9 chars
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        // write!(f, "{}", self.0.format("%-m/%-d/%y %-I:%M %p"))
        write!(f, "{}", self.0.format("%-m/%-d/%y"))
    }
}

pub struct SortOrder {
    pub fileproperty: FileProperty,
    pub ascending: bool,
}

impl SortOrder {
    pub fn cmp_entries(&self, a: &Entry, b: &Entry) -> Ordering {
        match &self.fileproperty {
            FileProperty::Name => {
                if a.is_dir && !b.is_dir {
                    return Ordering::Less;
                }
                if !a.is_dir && b.is_dir {
                    return Ordering::Greater;
                }
                return a.name.to_string().to_lowercase().cmp(&b.name.to_string().to_lowercase());
            },
            FileProperty::Size => {
                match (a.size.0, b.size.0) {
                    (a,b) if a < b => Ordering::Less, 
                    (a,b) if a > b => Ordering::Greater,
                    _ => Ordering::Equal,
                }
            }
            FileProperty::Date => Ordering::Equal,
        }
    }
}


