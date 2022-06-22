#![allow(unused_variables)]
#![allow(unused_imports)]

use std::path::{Path, PathBuf};
use std::fs::DirEntry;
use std::io::{stdout, Write};

use crossterm::{
    terminal,
    queue,
    execute,
    cursor::MoveTo,
    style::{Print,},
    event::{
        read as await_next_event, 
        Event, 
        KeyCode, 
        KeyEvent, 
        KeyModifiers,
    },
};

use crate::tui_program::Program;

mod tui_program;

struct Model {
    cwd: PathBuf,
    entries: Vec<Entry>,
    focused_entry_index: u32,
    filter_text: String,
}



#[derive(Debug)]
struct Entry {
    path: PathBuf,
    is_dir: bool,
    name: String,
}

impl std::convert::From<DirEntry> for Entry {
    fn from(oldentry: DirEntry) -> Self {
        let path = oldentry.path();
        let metadata = oldentry.metadata().unwrap();
        let is_dir = path.is_dir();
        let mut name = path.file_name().unwrap().to_str().unwrap().to_string();
        if is_dir {
            name = format!("{}/", name);

        }
        Self {
            path: path,
            is_dir: is_dir,
            name: name,
        }
    }
}

fn read_entries(dir: &PathBuf) -> Vec<Entry> {
    dir.read_dir().unwrap()
        .map(|e| Entry::from(e.unwrap()) )
        .collect()
}

fn init() -> Model {
    let cwd = std::env::current_dir().unwrap();
    let entries = read_entries(&cwd);
    Model {
        cwd: cwd,
        entries: entries,
        focused_entry_index: 0,
        filter_text: "filter".to_string(),
    }
}

fn update(m: &mut Model, e: Event) -> Option<()> {
    match e {
        Event::Key(keyevent) => {
            match keyevent.code {
                KeyCode::Char('q') => return None,
                KeyCode::Char(c) => {
                    m.filter_text = format!("{}{}", m.filter_text, c);
                }
                _ => (),
            }
        },
        _ => (),
    }
    return Some(())
}

fn view(m: &Model) -> String {
    format!("{:?}\n{:?}\n{}\n{}",
        m.cwd,
        m.entries,
        m.focused_entry_index,
        m.filter_text,
    )
}


fn main() {
    println!("");
    Program {init, view, update}.run();
}
