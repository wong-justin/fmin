use std::path::PathBuf;
use std::collections::HashSet;
use std::fs::DirEntry;

#[derive(PartialEq, Clone)]
pub enum FileProperty {
    Name(String),
    Size(Option<f64>),
    Modified(Option<String>),
}

/*
pub type FileName = String;
pub type FileSize = u64;  // num bytes returned by path.metadata.len
pub type FileDate = String;

pub type SortOrder<T> = bool;
*/

pub struct SortOrder {
    pub property: FileProperty,
    pub ascending: bool,
}

#[derive(PartialEq, Clone)]
pub struct Entry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub name: FileProperty,
    pub size: FileProperty,
    pub modified: FileProperty,
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
        let mut name : String = p.file_name().unwrap().to_str().unwrap().to_string();
        let is_dir = p.is_dir();
        if is_dir {
            name = format!("{}/", name);
        }

        return Self {
            path: p,
            is_dir: is_dir, 
            name: FileProperty::Name(name),
            size: FileProperty::Size(None),
            modified: FileProperty::Modified(None),
        };
    }
}

impl std::fmt::Display for FileProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let emptystr = String::from(""); 
        let emptystr = String::new();
        let o : String = match self {
            Self::Name(o) => o.clone(),
            // Self::Name(o) => o.as_deref().unwrap_or(&emptystr).to_string(),
            Self::Size(o) => match o {
                Some(n) => n.to_string(),
                None => emptystr, 
            },
            Self::Modified(o) => match o {
                Some(d) => d.to_string(),
                None => emptystr,
            },
        };
        write!(f, "{}",  o)
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
