use std::path::PathBuf;
use std::collections::HashSet;
use std::fs::DirEntry;
use std::marker::PhantomData;
use std::fmt::{Display, Formatter, Error};
use std::cmp::Ordering;

#[derive(PartialEq, Clone)]
pub struct FileSize(u64);
#[derive(PartialEq, Clone)]
pub struct FileName(String);
#[derive(PartialEq, Clone)]
pub struct FileDate(String);

/*
pub trait FileProperty {}

impl FileProperty for FileSize {}
impl FileProperty for FileName {}
impl FileProperty for FileDate {}
*/

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

pub struct Layout {
    pub W: usize,
    pub H: usize,
    pub list_min_pos: usize,
    pub list_max_pos: usize,
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
        
        return Self {
            path: p,
            is_dir: is_dir, 
            name: FileName(name),
            size: FileSize(metadata.len()),
            modified: FileDate("Jan 1".to_string()),
        };
    }
}

/*
impl FileProperty {
    fn cmp(&self, other: Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Name(a), Self::Name(b)) => {
                if a.is_dir && !b.is_dir {
                    return Ordering::Less;
                }
                if !a.is_dir && b.is_dir {
                    return Ordering::Greater;
                }
                return a.name.to_string().to_lowercase().cmp(&b.name.to_string().to_lowercase());
         
            },
        }

    }
}
*/

impl Layout {

    pub fn empty() -> Self {
        Self {
            W: 0,
            H: 0,
            list_min_pos: 0,
            list_max_pos: 0,
        }
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.W = w as usize;
        self.H = h as usize;
        self.list_max_pos = self.H - 6 + self.list_min_pos;
    }

    pub fn reset_list_pos(&mut self) {
        self.list_min_pos = 0;
        self.list_max_pos = self.H - 6
    }
}

impl Display for FileSize {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}B", self.0)
    }
}

impl Display for FileName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
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

/*
pub struct SortOrder<T> where T: FileProperty {
    phantom: PhantomData<T>,
    pub ascending: bool,
}

impl<T> SortOrder<T> where T: FileProperty {
    pub fn ascending(b : bool) -> Self {
        Self {
            ascending: b,
            phantom: PhantomData,
        }
    }
}

impl<T> Display for SortOrder<T> where T: FileProperty {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}: {}", std::any::type_name::<T>(), self.ascending)
    }
}
*/

