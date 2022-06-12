#![allow(unused_imports)]
#![allow(unused_variables)]

use std::io::Write;
use std::env;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::cmp::Ordering;

use crossterm::{
    queue,
    // groups of commands that return data, oft special ansi terminal sequences
    cursor,
    terminal,
    style::{Print, Color, SetBackgroundColor, SetForegroundColor, ResetColor},
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
};

use crate::tuimodel::{Model, start_event_loop};
use crate::types::{Entry, FileProperty, SortOrder,};

mod tuimodel;
mod types;

fn main() {
    let cwd = env::current_dir().unwrap();
    let mut entries = dir_entries(&cwd);
    let defaultSortOrder = SortOrder{property: FileProperty::Name("".to_string()), ascending: true};
    sort_entries(&mut entries);

    let focused_entry = Some(entries[0].clone());
    // let focused_entry = entries.get(0);

    start_event_loop(AppModel{
        cwd: cwd, 
        entries: entries,
        focused_entry: focused_entry,
        sortOrder: defaultSortOrder,
        lastKey: " ".into(),
        lastModifier: " ".into(),
    });
}

fn into_name_str(p: &Entry) -> String {
    return p.name.to_string();
    // let osstr = p.file_name().unwrap();
    // return osstr.to_str().unwrap_or("").to_string();
}


struct AppModel {
    cwd: PathBuf,
    entries: Vec<Entry>, 
    // focusedEntry could be None at any time.
    // Also would be more efficient to store only a ref,
    // but that requires marking lifetimes which is too advanced for me
    focused_entry: Option<Entry>,
    sortOrder: SortOrder,
    lastKey: String,
    lastModifier: String,
}

impl Model for AppModel {

    fn update(&mut self, ev : Event) -> Option<()> { 
        match ev {
            Event::Key(keyevent) => {
                self.lastKey = format!("{:?}", keyevent.code);
                if keyevent.code == KeyCode::Esc || keyevent.code == KeyCode::Char('q') {
                    return None;
                }
                match keyevent.code {
                    KeyCode::Char('k') => self.nav_up(),
                    KeyCode::Char('j') => self.nav_down(),
                    KeyCode::Enter => match &self.focused_entry {
                        Some(entry) => self.forward_dir(entry.path.clone()),
                        None => (),
                    },
                    _ => (),
                };
            },
            Event::Resize(cols, rows) => {
            },
            _ => (),
        }
        return Some(());
    }
    /*
    Tdown = "┬";
    Tup = "┴";
    Vbar = "│";
    Hbar = "─";
    */

    fn view(&self, buf: &mut impl Write) {
        let (w, h) = terminal::size().unwrap();
        // crossterm returns u16, but all other math in view() defaults
        // to usize, so just change it here
        let (w, h) = (w as usize, h as usize);
        let divider : String = "=".repeat((w - 2).into());
        let top_divider_2 = "─┬────────┬────────────";
        let labels2 =       " │ Size   │ Modified   ";
        let bot_divider_2 = "─┴────────┴────────────";
        let label1 = " Name";
        let spaces_width = w - 2 - str_width(label1) - str_width(labels2);
        let divider1 = "─".repeat(str_width(label1) + spaces_width);
        let labels = format!("{}{}{}",
            label1, " ".repeat(spaces_width), labels2,
        );
        let top_divider = format!("{}{}", divider1, top_divider_2);
        let bot_divider = format!("{}{}", divider1, bot_divider_2);
        
        queue!(
            buf,
            // SetForegroundColor(Color::White),
            cursor::MoveTo(1,1),
            Print(self.cwd.to_str().unwrap()),
            cursor::MoveTo(1,2),
            Print(&top_divider),
            cursor::MoveTo(1,3),
            Print(labels),
            cursor::MoveTo(1,4),
            Print(&bot_divider),
        );
        let mut lineno = 5;
        for entry in &self.entries { 
            let child_filename = entry.name.to_string();

            // chars.count is num unicode points
            // which i assume equals num char widths shown on terminal
            // bad assumption bc diff fonts could render some unicode
            // in two char widths instead of one,
            // or even more accurately some fontthings are composed
            // of two unicode points, the first (and maybe second)
            // of which are a normal fontthing (eg black person emoji),
            // but seems like best possible with std
            let name_width = str_width(&child_filename);
            let spaces_width = w - name_width - 2;
            let spaces = " ".repeat(spaces_width);
            queue!(
                buf,
                cursor::MoveTo(1, lineno),
            );

            let is_focused = self.focused_entry.as_ref() == Some(entry);
            if is_focused {
                queue!(buf, SetBackgroundColor(Color::DarkGrey));
            }

            queue!(
                buf,
                Print(format!("{}{}", &child_filename, spaces)),
                // SetBackgroundColor(Color::DarkGrey),
                ResetColor,
                // SetForegroundColor(Color::White),
            );
            lineno += 1;
        }
        
        queue!(
            buf,
            cursor::MoveTo(10,10),
            Print(&self.lastKey),
            Print(format!("{} {}", w, h)),
        ).unwrap();
    }
}

impl AppModel {

    fn curr_entry_ref(&self) -> Option<&Entry> {
        return self.focused_entry.as_ref();
    }

    fn forward_dir(&mut self, p: PathBuf) {
        let mut entries = dir_entries(&p);
        let defaultSortOrder = SortOrder{property: FileProperty::Name("".to_string()), ascending: true};
        sort_entries(&mut entries);

        let focused_entry = match entries.len() {
            0 => None,
            _ => Some(entries[0].clone()),
        };
        
        self.cwd = p;
        self.entries = entries;
        self.focused_entry = focused_entry;
        self.sortOrder = defaultSortOrder;
    }

    fn nav_up(&mut self) {
        /*
        let e = match &self.focused_entry {
            Some(entry) => entry,
            None => return,
        };
        */
        // let e = self.focused_entry.unwrap_or_else(|| return);
        let e : &Entry = match &self.focused_entry {
            Some(e) => e,
            None => return
        };

        let i = self.entries.iter().position(|x| x == e).unwrap();
        if i == 0 {
            return;
        }
        self.focused_entry = Some(self.entries[i-1].clone());
    }

    fn nav_down(&mut self) {
        let e = match &self.focused_entry {
            Some(e) => e,
            None => return,
        };

        let i = self.entries.iter().position(|x| x == e).unwrap();
        if i == self.entries.len()-1 {
            return;
        }
        self.focused_entry = Some(self.entries[i+1].clone());
    }
}

fn str_width<S: AsRef<str>> (s: S) -> usize {
    return s.as_ref().chars().count();
}

fn dir_entries(dir: &PathBuf) -> Vec<Entry> {
    let mut entries = Vec::<Entry>::new();

    for entry in dir.read_dir().unwrap() {
        let e = Entry::from(entry.unwrap());
        entries.push(e);
    }
    return entries;
}

fn sort_entries(entries: &mut Vec<Entry>) {
    // ordered.sort_by_key(|a| std::cmp::Reverse( into_name_str(a) ) );
    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            return Ordering::Less;
        }
        if !a.is_dir && b.is_dir {
            return Ordering::Greater;
        }
        return a.name.to_string().to_lowercase().cmp(&b.name.to_string().to_lowercase());
    });
}

