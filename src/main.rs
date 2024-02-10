// fmin, a temrinal file manager inspired by fman + vim

#![allow(unused_variables)]
#![allow(unused_imports)]

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Display, Formatter, Error};
use std::fs::DirEntry;
use std::hash::{Hash, Hasher};
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};

use crossterm::{
    terminal,
    queue,
    execute,
    cursor::MoveTo,
    style::{Print, Color, SetBackgroundColor, SetForegroundColor, ResetColor},
    event::{
        read as await_next_event, 
        Event, 
        KeyCode, 
        KeyEvent, 
        KeyModifiers,
    },
};
use log::{info};

use crate::tui_program::Program;

mod tui_program;

struct Model {
    mode: Mode,
    cwd: PathBuf,
    cwd_sort: SortBy,
    // small optimization i think: sort entries at cd time, then filter from the sorted list on
    // each keypress
    // rather than sort from filtered list every keypress
    // should calculate and store filtered and sorted entries at cd time
    // so that less work is required during render time
    all_entries: HashSet<Entry>,
    sorted_entries: Vec<StringifiedEntry>,
    filter_text: String,
    cols: usize,
    rows: usize,
}
enum Mode {
    // when filter input box is focused. could be empty but still have focused. can still exec
    // other comamnds with control characters
    Filter,
    Normal,
    Jump,
    // CommandPalette,
}

// change state and do side effects
enum Action {
    GotoDir(PathBuf),
    SetFilterText(String),
    SelectFirstEntry,
    StartFilterMode,
    StartJumpMode,
    // StartCommandPaletteMode,
    Noop,
    Quit,
}

#[derive(Copy, Clone)]
enum SortAttribute {
    Name,
    // Size,
    // Date,
}

#[derive(Copy, Clone)]
struct SortBy {
    attribute: SortAttribute,
    // ascending: bool,
}

struct Entry {
    path: PathBuf,
    is_dir: bool,
    name: FileName,
    // size: Maybe(FileSize),
    // date: Maybe(FileDate),
}

struct StringifiedEntry {
    name: String,
}

impl Clone for StringifiedEntry {
    fn clone(&self) -> StringifiedEntry {
        StringifiedEntry {
            name: self.name.clone(),
        }
    }
}

impl Clone for Entry {
    fn clone(&self) -> Entry {
        Entry {
            path: self.path.clone(),
            is_dir: self.is_dir.clone(),
            name: self.name.clone(),
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Entry {}

impl Hash for Entry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(PartialEq, Clone)]
struct FileName(String);

// #[derive(PartialEq, Clone)]
// struct FileSize(u64);
// #[derive(PartialEq, Clone)]
// struct FileDate(DateTime<Local>);

impl Display for FileName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl SortBy {
    pub fn compare_entries(&self, a: &Entry, b: &Entry) -> Ordering {
        match &self.attribute {
            SortAttribute::Name => {
                if a.is_dir && !b.is_dir {
                    return Ordering::Less;
                }
                if !a.is_dir && b.is_dir {
                    return Ordering::Greater;
                }
                return a.name.to_string().to_lowercase().cmp(&b.name.to_string().to_lowercase());
            },
            // SortAttribute::Size => {
            //     match (a.size.0, b.size.0) {
            //         (a,b) if a < b => Ordering::Less, 
            //         (a,b) if a > b => Ordering::Greater,
            //         _ => Ordering::Equal,
            //     }
            // }
            // SortAttribute::Date => Ordering::Equal,
        }
    }
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::convert::From<DirEntry> for Entry {
    // todo: metadata unwrap() fail, likely unauthorized special windows folders
    // to reproduce: read_entries(/mnt/c)
    fn from(oldentry: DirEntry) -> Self {
        let path = oldentry.path();
        // let metadata = oldentry.metadata().unwrap();
        let is_dir = path.is_dir();
        let mut name = path.file_name().unwrap().to_str().unwrap().to_string();
        if is_dir {
            name = format!("{}/", name);
        }
        Self {
            path: path,
            is_dir: is_dir,
            name: FileName(name),
        }
    }
}

fn read_entries(dir: &PathBuf, sort: SortBy) -> HashSet<Entry> {
    match dir.read_dir() {
        Ok(readdir) => {
            readdir
                .map( |e| {
                    match e {
                        Ok(direntry) => Ok(Entry::from(direntry)),
                        // todo: give better msg if io err happens getting dir entry
                        Err(err) => Err(err),
                    }
                })
                .filter_map(Result::ok)
                .collect()
        },
        // todo: give better msg if io err reading dir
        Err(err) => HashSet::new()
    }
    // entries.sort_by(|a,b| sort.compare_entries(a,b) );
    // return entries;
}

// APP LOGIC

fn main() {
    println!("");
    Program {init, view, update}.run();
}

fn init() -> Model {
    let cwd = std::env::current_dir().unwrap();
    // let cwd = PathBuf::from("/mnt/c/Users");
    let sort = SortBy{ attribute: SortAttribute::Name };
    let all_entries = read_entries(&cwd, sort);
    let sorted_entries = sort_entries(&all_entries, sort);
    let (cols, rows) = match terminal::size() {
        Ok((cols, rows)) => (usize::from(cols), usize::from(rows)),
        // TODO: response to error of not knowing terminal size
        Err(err) => (0,0)
    };

    Model {
        cwd: cwd,
        cwd_sort: sort,
        all_entries: all_entries,
        sorted_entries: sorted_entries,
        filter_text: "".to_string(),
        mode: Mode::Filter,
        cols: cols,
        rows: rows,
    }
}

fn sort_entries(entries: &HashSet<Entry>, sort: SortBy) -> Vec<StringifiedEntry> {
    let mut entries_vec : Vec<&Entry> = entries
        .into_iter()
        .collect::<Vec<&Entry>>();
    entries_vec.sort_by(|a,b| sort.compare_entries(a,b));
    entries_vec
        .into_iter()
        .map(|entry| StringifiedEntry { name : entry.name.0.clone() })
        .collect::<Vec<StringifiedEntry>>()
}

fn update(m: &mut Model, terminal_event: Event) -> Option<()> {
    // exit early if ctrl+c, no matter what
    match terminal_event {
        Event::Key(keyevent) => {
            if
                keyevent.modifiers == KeyModifiers::CONTROL &&
                keyevent.code == KeyCode::Char('c')
            {
                return None;
            }
        },
        Event::Resize(cols, rows) => {
            m.cols = usize::from(cols);
            m.rows = usize::from(rows);
        }
        _ => ()
    };
    // respond to crossterm event and output an action
    let action = match m.mode {
        Mode::Normal => {
            match terminal_event {
                Event::Key(keyevent) => {
                    match keyevent.code {
                        KeyCode::Char('/') => Action::StartFilterMode,
                        KeyCode::Char('>') => Action::StartJumpMode, // maybe @ would work?
                        // KeyCode::Char('?') => Action::StartCommandPaletteMode,
                        KeyCode::Backspace => {
                            match m.cwd.parent() {
                                Some(path) => Action::GotoDir(path.to_owned()),
                                None => Action::Noop,
                            }
                        },
                        // KeyCode::Char('j') => {
                        //     match keyevent.modifiers {
                        //         KeyModifiers::CONTROL => Action::StartJumpMode,
                        //         _ => Action::Noop,
                        //     }
                        // },
                        KeyCode::Char('q') => Action::Quit,
                        _ => Action::Noop,
                    }
                },
                _ => Action::Noop,
            }
        },
        Mode::Filter => {
            match terminal_event {
                Event::Key(keyevent) => {
                    // todo: listen for end-of-input ctrl+d, arrow keys left and right, paste
                    // ideally a readline lib is used to listen for input here, but
                    // - dont need history or multiline editing, and probably not vim/emacs shortcuts
                    // - not sure how to include static .so external dependency
                    // so probably not worth including that extra dependency
                    match keyevent.code {
                        KeyCode::Esc => Action::SetFilterText("".to_string()),
                        // KeyCode::Char('j') => {
                        //     match keyevent.modifiers {
                        //         KeyModifiers::CONTROL => Action::StartJumpMode,
                        //         _ => Action::SetFilterText(format!("{}j", m.filter_text)),
                        //     }
                        // },
                        KeyCode::Char(c) => {
                            Action::SetFilterText(format!("{}{}", m.filter_text, c))
                        },
                        KeyCode::Backspace => {
                            match m.filter_text.is_empty() {
                                // if filter text input is empty, backspace will nav back as if in
                                // normal mode.
                                // this reduces keypresses when navigating since normal mode is not
                                // default.
                                true => match m.cwd.parent() {
                                    Some(path) => Action::GotoDir(path.to_owned()),
                                    None => Action::SetFilterText("".to_string()), // clear and go normal mode
                                },
                                // remove last char. warning: doesnt account for unicode.
                                // but this quick and dirty solution is ok for now.
                                // and most filepath inputs will only have ascii anyways.
                                false => {
                                    let mut chars = m.filter_text.chars();
                                    chars.next_back();
                                    let all_chars_but_last = chars.as_str().to_string();
                                    Action::SetFilterText(all_chars_but_last) // potentially clear and go normal mode
                                },
                            }
                        },
                        KeyCode::Enter => Action::SelectFirstEntry,
                        _ => Action::Noop,
                    }
                },
                _ => Action::Noop,
            }
        },
        Mode::Jump => {
            match terminal_event {
                Event::Key(keyevent) => {
                    match keyevent.code {
                        KeyCode::Esc => Action::SetFilterText("".to_string()),
                        _ => Action::Noop,
                    }
                },
                _ => Action::Noop,
            }
        },
    };
    // update state
    match action {
        Action::GotoDir(pathbuf) => {
            m.all_entries = read_entries(&pathbuf, m.cwd_sort);
            m.sorted_entries = sort_entries(&m.all_entries, m.cwd_sort);
            m.mode = Mode::Filter;
            m.cwd = pathbuf;
            Some(())
        },
        Action::SetFilterText(text) => {
            m.mode = match text.is_empty() {
                true => Mode::Normal,
                false => Mode::Filter,
            };
            m.filter_text = text;
            Some(())
        },
        Action::SelectFirstEntry => {
            let first = m.all_entries
                .iter()
                .filter(|entry| entry.name.0.to_lowercase().contains(&m.filter_text.to_lowercase()) )
                .filter(|entry| entry.is_dir )
                .next();
            match first {
                Some(entry) => {
                    m.cwd = entry.path.clone();
                    m.all_entries = read_entries(&entry.path, m.cwd_sort);
                    m.sorted_entries = sort_entries(&m.all_entries, m.cwd_sort);
                    m.mode = Mode::Filter;
                    m.filter_text = "".to_string();
                    Some(())
                },
                None => Some(()),
            }
        },
        Action::StartFilterMode => {
            m.mode = Mode::Filter;
            Some(())
        },
        Action::StartJumpMode => {
            m.mode = Mode::Jump;
            Some(())
        },
        Action::Noop => Some(()),
        Action::Quit => None,
    }
}

// UI AND DIRTY STRING HANDLING BELOW

fn view(m: &Model, stdout: &mut std::io::Stdout) {
    // must be impure function writing to mutable buf stdout
    // since crossterm lib puts control bytes in custom types like SetBackgroundColor
    // that must be used here in impure queue!(buf, ...) function
    // and not postponed for agnostic model/update/view loop
    // maybe i can send a list of crossterm::Commands to queue...

    // OLD PRINT LINES:
    
    // let output_string = format!("\n {}\n\n{}\n\n {}{}\n\n controls:\n{}",
    //     m.cwd.display().to_string(),
    //     match m.mode {
    //         Mode::Filter | Mode::Normal => {
    //             m.all_entries
    //                 .iter()
    //                 .map(|entry| format!("  {}", entry.name) )
    //                 .filter(|name| name.to_lowercase().contains(&m.filter_text.to_lowercase()) )
    //                 .collect::<Vec<String>>()
    //                 .join("\n")
    //         },
    //         Mode::Jump => "".to_string(),
    //     },
    //     match m.mode {
    //         Mode::Filter => "(filter)           /",
    //         Mode::Normal => "(normal)",
    //         Mode::Jump => "(jump to dir)          jump to:",
    //         // Mode::CommandPalette => "(commands)          ?",
    //     },
    //     m.filter_text,
    //     match m.mode {
    //         Mode::Filter => "   type to filter\n   esc to clear\n   enter to open dir\n   ctrl+j to jump to frecent dir",
    //         Mode::Normal => "   / to filter curr dir\n   ctrl+j to jump to frecent dir\n   backspace to nav up dir\n   enter to open dir or file", // ? for command palette
    //         Mode::Jump => "   type to filter frecent dirs\n   esc to clear\n   enter to nav to dir",
    //         // Mode::CommandPalette => "type to filter commands, esc to clear, enter to execute",
    //         // potential command palette commands:
    //         // (y)ank selected files
    //         // (p)aste selected files
    //         // (m)ark file as selected
    //         // (g)o to top of list
    //         // (G)o to bottom of list
    //         // (ctrl+j)ump to frecent dir
    //         // open command (ctrl+p)alette
    //         // (Q)uit fmin
    //         // print selected filepaths to stdout
    //         // copy selected filepaths to clipboard

    //         // consider also server/client setup
    //         // so dual-pane is optional and happens by twm / terminal panes
    //         // and one client can yank and the other client can paste
    //         // and also getting cli options while client tui is running, like
    //         // (yank in client and then elsewhere) fmin --print-selected-filepaths

    //         // also remember feature of updating cwd if other programs update files in the dir
    //         // need directory watcher functionality, kinda like entr
    //         // although its workaroundable by just going back and forth, essentially refreshing
    //     },
    // );
    // let mut i = 0;
    // let lines = output_string.split("\n");
    // for line in lines {
    //     queue!(stdout, MoveTo(0, i), Print(line),);
    //     i += 1;
    // }
    

    // NEW PRINT
    //
    // idea for declarative view, without implementing a whole ui framework

    //  C:\users\jkwon\desktop\programming\modenv
    //  ___________________________________________
    //   Name                v | Size   | Modified
    //  ___________________________________________
    //  loopy/                   12       2022-06
    //  droopy/                  4        2022-06
    //  grumpy/                  0        2022-06
    //  frumpy/                  99       2022-06
    //  script1.py               92 KB    2022-06
    //  script_2.py              108 MB   2022-07
    //  main.py                  1.2 GB   2022-05
    //  utils.py                 985 B    2021-12
    // 
    //  :openwithvim                   /py 
    //  ___________________________________________
    
    // | <- fill -> |
    // | <- fill -> |
    // | <- fill -> | 6 | 8 |
    // | <- fill -> |
    //   | <- fill -> (highlight) | 6 (highlight) | 8 (highlight) |
    // for v_stretch - 1
    //   | <- fill -> | 6 | 8 |
    // | <- fill -> |
    // | <- fill -> | 10 (cursor sometimes) |
    // | <- fill -> |
    
    // m.mode == filter?
    let divider : &str = &"-".repeat(m.cols);
    queue!(stdout, MoveTo(1, 1), fit(&m.cwd.display().to_string(), m.cols));
    queue!(stdout, MoveTo(0, 2), Print(divider));
    queue!(stdout, MoveTo(0, 3), 
           fit(" Name ", m.cols - 8 - 10),
           fit(" Size ", 8),
           fit(" Modified ", 10),
    );
    queue!(stdout, MoveTo(0, 4), Print(divider));

    let filtered_entries = m.sorted_entries
        .clone()
        .into_iter()
        .filter(|entry| entry.name.to_lowercase().contains(&m.filter_text.to_lowercase()) )
        .collect::<Vec<StringifiedEntry>>();

    let initialRowOffset = 5;
    let endingRowOffset = 3;
    let maxRowNum = m.rows - endingRowOffset;
    for (i, entry) in filtered_entries.iter().enumerate() {
        let rowNum = i + initialRowOffset;
        if rowNum == maxRowNum { break; }
        if i == 0 { queue!(stdout, SetBackgroundColor(Color::DarkGrey)); }
        // itemhighlighted?
        queue!(stdout,
               MoveTo(1, (rowNum).try_into().unwrap()),
               fit(&entry.name, m.cols - 8 - 10),
        );
        if i == 0 { queue!(stdout, ResetColor); }
    }

    queue!(stdout, MoveTo(0, (m.rows - 2).try_into().unwrap()), Print(divider));
    queue!(stdout, MoveTo(0, (m.rows - 1).try_into().unwrap()), 
           Print(&format!(" {} {}",
                        match m.mode {
                            Mode::Filter => "(filter)",
                            Mode::Normal => "(normal)",
                            Mode::Jump => "(jump to)",
                        },
                        match m.mode {
                            Mode::Filter => format!(" /{}", m.filter_text),
                            _ => String::new(),
                        },
                        ),
               ),
    );
    match m.mode {
        Mode::Filter => queue!(stdout, crossterm::cursor::Show,),
        _ => queue!(stdout, crossterm::cursor::Hide,),
    };
}

fn fit(s: &str, final_length: usize) -> Print<String> {
    Print(fit_to_length(s, final_length))
}

fn fit_to_length(s: &str, final_length: usize) -> String {
    // assuming only ASCII input...
    match str_length(s) {
        // too short
        length if length <= final_length => {
            let padding = " ".repeat(final_length - length);
            format!("{}{}", s, padding)
        },
        // too long
        length => {
            s.chars().take(final_length).collect::<String>()
        }

    }
}

fn str_length<S: AsRef<str>> (s: S) -> usize {
    // assuming only ASCII input...
    return s.as_ref().chars().count();
}

