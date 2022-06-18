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
use crate::types::{Entry, FileProperty, FileSize, FileDate, FileName, SortOrder,};
use crate::ui::{Layout,};

mod tuimodel;
mod types;
mod ui;

fn main() {
    let cwd = env::current_dir().unwrap();

    start_event_loop(AppModel::new(cwd));
}

struct AppModel {
    cwd: PathBuf,
    // sorted according to sort_order
    entries: Vec<Entry>, 
    // focusedEntry could be None at any time.
    // Also would be more efficient to store only a ref,
    // but that requires marking lifetimes which is too advanced for me
    focused_entry: Option<Entry>,
    sort_order: SortOrder, 
    // lastKey: String,
    // lastModifier: String,
    layout: Layout,
}


impl Model for AppModel {

    fn update(&mut self, ev : Event) -> Option<()> { 
        match ev {
            Event::Key(keyevent) => {
                match keyevent.code {
                    KeyCode::Char('k') => self.nav_up(),
                    KeyCode::Char('j') => self.nav_down(),
                    KeyCode::Enter => if let Some(entry) = &self.focused_entry  {
                        if entry.is_dir {
                            self.forward_dir(entry.path.clone())
                        }
                        else {
                            
                        }
                    },
                    KeyCode::Backspace => self.back_dir(),
                    KeyCode::Esc | KeyCode::Char('q') => return None,
                    _ => (),
                };
            },
            Event::Resize(w, h) => self.layout.resize(w, h),
            _ => (),
        }
        return Some(());
    }
    
    // ┬ ┴ │ ─

    fn view(&self, buf: &mut impl Write) {
        // crossterm returns u16, but all other math in view() defaults
        // to usize, so just change it here
        let (w, h) = (self.layout.W as usize, self.layout.H as usize);  // (w as usize, h as usize);
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
        for entry in self.viewable_entries().iter() {
//        for entry in &self.entries[self.layout.list_min_pos..self.layout.list_max_pos] {

            // let name = entry.name.to_string();
            let name = entry.name.fit(self.layout.col1_end - self.layout.col1_start + 1);

            let size_width = self.layout.col2_end - self.layout.col2_start;  // + 1;

            let size = match entry.is_dir {
                true => " ".repeat(size_width),
                false => entry.size.fit(size_width),
            };
            // let date = entry.modified.to_string();
            let date = entry.modified.fit(self.layout.col3_end - self.layout.col3_start);  // + 1);

            // chars.count is num unicode points
            // which i assume equals num char widths shown on terminal
            // bad assumption bc diff fonts could render some unicode
            // in two char widths instead of one,
            // or even more accurately some fontthings are composed
            // of two unicode points, the first (and maybe second)
            // of which are a normal fontthing (eg black person emoji),
            // but seems like best possible with std
            queue!(
                buf,
                cursor::MoveTo(self.layout.col1_start as u16, lineno),
            );

            let is_focused = self.focused_entry.as_ref() == Some(entry);
            if is_focused {
                queue!(buf, SetBackgroundColor(Color::DarkGrey));
            }

            queue!(
                buf,
                Print(format!("{}  {}  {}", name, size, date)),
                ResetColor,
            );
            lineno += 1;
        }
        /*
        let divider = "─".repeat(w - 2);
        queue!(
            buf,
            cursor::MoveTo(1, (h as u16) - 2),
            Print(divider),
        );
        */
        
        /*
        queue!(
            buf,
            cursor::MoveTo(10,10),
            Print(&self.lastKey),
            Print(format!("{} {}", w, h)),
        ).unwrap();
        */
    }
}

impl AppModel where {

    fn cd_into(&mut self, dir: PathBuf) {
        let mut entries = dir_entries(&dir);
        // let default_sort_order = SortOrder::<FileName>::ascending(true);
        let default_sort_order = SortOrder{ fileproperty: FileProperty::Size, ascending: true };

        entries.sort_by(|a,b| default_sort_order.cmp_entries(a,b));

        let focused_entry = match entries.len() {
            0 => None,
            _ => Some(entries[0].clone()),
        };
        
        self.cwd = dir;
        self.entries = entries;
        self.focused_entry = focused_entry;
        self.layout.reset_list_pos();
    }

    fn new(dir: PathBuf) -> Self {
        let mut entries = dir_entries(&dir);
        // let default_sort_order = SortOrder::<FileName>::ascending(true);
        let default_sort_order = SortOrder{ fileproperty: FileProperty::Size, ascending: true };

        entries.sort_by(|a,b| default_sort_order.cmp_entries(a,b));

        let focused_entry = match entries.len() {
            0 => None,
            _ => Some(entries[0].clone()),
        };

        let (w, h) = terminal::size().unwrap();
        let mut layout = Layout::default();
        layout.resize(w, h);

        Self {
            cwd: dir,
            entries: entries,
            focused_entry: focused_entry,
            sort_order: default_sort_order,
            layout: layout,
        }
    }

    fn viewable_entries(&self) -> &[Entry] {
        let begin = self.layout.list_min_pos;
        let end = std::cmp::min( self.layout.list_max_pos + 1, self.entries.len() );

        return &self.entries[begin..end];
    }

    fn forward_dir(&mut self, p: PathBuf) {
        self.cd_into(p);
    }

    fn back_dir(&mut self) {
        let parent = match self.cwd.parent() {
            Some(path) => path.to_owned(),
            None => return,
        };
        self.cd_into(parent);
    }

    fn nav_up(&mut self) {
        if let Some(i) = self._focused_entry_pos() {
            if i == 0 {
                return;
            }
            self.focused_entry = Some(self.entries[i-1].clone());
        }
    }

    fn nav_down(&mut self) {
        if let Some(i) = self._focused_entry_pos() {
            if i == self.entries.len()-1 {
                return;
            }
            self.focused_entry = Some(self.entries[i+1].clone());
        }
    }

    fn _focused_entry_pos(&self) -> Option<usize> {
        let e: &Entry = match &self.focused_entry {
            Some(e) => e,
            None => return None,
        };

        let i = self.entries.iter().position(|x| x == e).unwrap();
        Some(i)
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

