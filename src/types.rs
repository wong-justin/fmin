use std::path::PathBuf;
use std::collections::HashSet;
use std::fs::DirEntry;

#[derive(PartialEq, Clone)]
pub enum FileProperty {
    Name(String),
    Size(Option<f64>),
    Modified(Option<String>),
}

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

