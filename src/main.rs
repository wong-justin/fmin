// fmin, a temrinal file manager inspired by fman + vim

#![allow(unused_variables)]
#![allow(unused_imports)]

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Display, Formatter, Error};
use std::fs::DirEntry;
use std::hash::{Hash, Hasher};
use std::io::{Write};
use std::path::{Path, PathBuf};

use byte_unit::Byte;
use chrono::{DateTime, Datelike, TimeZone, Local};
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

// width of attribute columns, assuming ascii chars, based on desired formatted output and what
// looks nice imo
const SIZE_MAX_COLS : usize = 7;
const DATE_MAX_COLS : usize = 14;
const MARGIN_COLS : usize = 2;
const MARGIN : &str = "  ";

// --- MODEL AND TYPES --- //

struct Model {
    mode: Mode,
    cwd: PathBuf,
    cwd_sort: SortBy,
    // small optimization i think: sort entries at cd time, then filter from the sorted list on
    // each keypress
    // rather than sort from filtered list every keypress
    // should calculate and store filtered and sorted entries at cd time
    // so that less work is required during render time
    // all_entries -> sort -> filter -> viewable slice of entries
    all_entries: HashSet<Entry>,
    sorted_entries: Vec<StringifiedEntry>,
    filter_text: String,
    cols: usize,
    rows: usize,
}
enum Mode {
    Filter,
    Normal,
    // CommandPalette,
}

// change state and do side effects
enum Action {
    GotoDir(PathBuf),
    SetFilterText(String),
    SelectFirstEntry,
    StartFilterMode,
    ChangeSortOrder(EntryAttribute),
    ReverseSort,
    // StartCommandPaletteMode,
    Noop,
    Quit,
}

#[derive(Copy, Clone, PartialEq)]
enum EntryAttribute {
    Name,
    Size,
    Date,
}

#[derive(Copy, Clone)]
struct SortBy {
    attribute: EntryAttribute,
    ascending: bool,
}

struct Entry {
    path: PathBuf,
    is_dir: bool,
    name: FileName,
    // concerning maybe(date) and maybe(size):
    // dirs dont have filesize - maybe count num of items in dir and display that instead? even recursively?
    // also for permission errors or special folders like $recyclebin - probably wont give you
    // any metadata about size or date modified
    size: Option<FileSize>,
    date: Option<FileDate>, 
}

struct StringifiedEntry {
    name: String,
    size: String,
    date: String,
}

impl Clone for StringifiedEntry {
    fn clone(&self) -> StringifiedEntry {
        StringifiedEntry {
            name: self.name.clone(),
            size: self.size.clone(),
            date: self.date.clone(),
        }
    }
}

impl From<&Entry> for StringifiedEntry {
    fn from(entry: &Entry) -> Self {
        StringifiedEntry {
            name : entry.name.0.clone(),
            size: match &entry.size {
                Some(size_bytes) => size_bytes.to_string(),
                None => String::new(),
            },
            date: match &entry.date {
                Some(date_modified) => date_modified.to_string(),
                None => String::new(),
            },
        }
    }
}

impl Clone for Entry {
    fn clone(&self) -> Entry {
        Entry {
            path: self.path.clone(),
            is_dir: self.is_dir.clone(),
            name: self.name.clone(),
            size: self.size.clone(),
            date: self.date.clone(),
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

#[derive(PartialEq, Clone)]
struct FileSize(u64);

#[derive(PartialEq, Clone)]
struct FileDate(DateTime<Local>);

impl Display for FileName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl Display for FileSize {
    // formatting options to explore:
    //
    // 1) two sig figs / digits, like
    // 0 - 99 B
    // 0.1 - 9.9, 10 - 99
    // KB, MB, GB
    // it has max 6 char widths
    //   4 B
    // 1.5 KB
    //  27 MB
    // 0.9 GB
    //
    // 2) or 3 sig figs, like
    // 0-999 B
    // 1.00 - 9.99
    // 10.0 - 99.9
    // 100 - 999
    // KB, MB, GB
    // it has max 7 char widths
    //    1 B
    //  514 KB
    // 87.2 MB
    // 2.31 GB
    //
    // 2.5) or just dont use decimals at all
    //
    // 3) copy how ls -lh does it
    //
    // 4) use standard behavior from bytes crate, like
    // let byte = Byte::from_bytes(self.0.into()).get_appropriate_unit(false);
    // "999.99 GB"  "1 B"
    // which is max 9 chars, min 3. not sure of the formatting alogorithm tho.
    //
    // 5) do like windows file explorer, everything just in kb...
    // 78,696 KB
    // 890 KB
    // 7 KB
    //
    // 6) always go with 1 decimal precision
    // 7 max char widths
    // 4 B
    // 1.5 K
    // 900.1 M
    // 27.9 G
    // which is good to be more consistent and therefore more readable than varying decimal places
    //
    // also note KB vs kb vs KiB vs Kb vs kB... just go with the all caps powers of 10 i think

    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        const SUFFIXES : [&str; 4] = ["G", "M", "K", "B"];
        const MAGNITUDES : [u64; 4] = [
            u64::pow(10,9),
            u64::pow(10,6),
            u64::pow(10,3),
            1
        ];

        let num_bytes = self.0;
        match num_bytes {
            b if b < u64::pow(10,3) => { 
                write!(f, "{} B", num_bytes)
            },
            b if b >= u64::pow(10,12) => {
                write!(f, "over 1 TB")
            },
            _ => {
                let mut i = 0;
                while MAGNITUDES[i] > num_bytes {
                    i += 1;
                }
                write!(f, "{:.1} {}", num_bytes as f64 / MAGNITUDES[i] as f64, SUFFIXES[i])
            },
        }
    }
}

impl Display for FileDate {
    // options for formatting last modified date:
    //
    // 1) either go midnight commander style, showing time if this year else year for prior year
    // which is 12 chars. eg. if now is july 2022, examples include
    // Jan  1 23:59
    // Jan  1 23:59
    // Dec 31  2021
    // Jun 15 12:00
    // although it hurts to lose 24h time when new year begins (eg. starting jan 2022, dec 2021
    // hours will be hidden)
    //
    // 1.5) shorten above a little bit by abbreviating year
    // and i guess abbreviating hour too
    // which is 10ch, like 
    // Dec 31 '21
    // Jan  1 +23
    // Jun 12 +12
    //
    // 2) or even like imperium, disp how long since modified, max 4 char widths
    // 8d.      8 d
    // 5hr.     5 h
    // 1min.    1 m
    // 2mo.     2 M
    // 1yr.     1 y
    // 19d.    19 d
    // aka
    // 1-59 m
    // 1 - 23 h
    // 1 - 29? d
    // 1 - 11 M
    // 1 - 99? y
    // bad tho because id like to see finer-grained differences, like 8d vs 8d12h
    //
    // 3) "10/10/10 10:10 PM"  "1/01/01 1:01 AM"
    // max 17 char widths, min 15
    // %-m/%-d/%y %-I:%M %p
    //
    // 4) "10/10/10" max 9 chars
    // %-m/%-d/%y
    
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let now = Local::now();
        let modified = self.0;
        let format = match now.year() == modified.year() {
            // https://docs.rs/chrono/latest/chrono/format/strftime/index.html
            true => "%b %e %k:%M",
            false => "%b %e  %Y",
        };
        write!(f, "{}", modified.format(format))
    }
}

impl SortBy {
    pub fn compare_entries(&self, a: &Entry, b: &Entry) -> Ordering {
        match &self.attribute {
            EntryAttribute::Name => {
                if a.is_dir && !b.is_dir {
                    return Ordering::Less;
                }
                if !a.is_dir && b.is_dir {
                    return Ordering::Greater;
                }
                return a.name.to_string().to_lowercase().cmp(&b.name.to_string().to_lowercase());
            },
            EntryAttribute::Size => {
                match (&a.size, &b.size) {
                    (None, Some(b)) => Ordering::Less,
                    (Some(a), None) => Ordering::Greater,
                    (Some(a), Some(b)) if a.0 < b.0 => Ordering::Less, 
                    (Some(a), Some(b)) if a.0 > b.0 => Ordering::Greater,
                    _ => Ordering::Equal,
                }
            }
            EntryAttribute::Date => {
                match (&a.date, &b.date) {
                    (None, Some(b)) => Ordering::Less,
                    (Some(a), None) => Ordering::Greater,
                    (Some(a), Some(b)) if a.0 < b.0 => Ordering::Less, 
                    (Some(a), Some(b)) if a.0 > b.0 => Ordering::Greater,
                    _ => Ordering::Equal,
                }
            }
        }
    }
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::convert::From<DirEntry> for Entry {
    // are there permission errors that could happen here?
    fn from(oldentry: DirEntry) -> Self {
        let path = oldentry.path();
        // TOOD: learn what filename errors are possible, then handle them
        let mut name = path.file_name().unwrap().to_str().unwrap().to_string();
        let is_dir = path.is_dir();
        let mut size_bytes = None;
        let mut date_modified = None;
        match oldentry.metadata() {
            Ok(metadata) => {
                size_bytes = Some(FileSize(metadata.len()));
                date_modified = match metadata.modified() {
                    // apparently some platforms do not have mtime / ftLastWriteTime available
                    // https://doc.rust-lang.org/std/fs/struct.Metadata.html#errors
                    Ok(system_time) => {
                        Some(FileDate(DateTime::<Local>::from(system_time)))
                    },
                    Err(err) => None,
                };
            },
            Err(err) => (),
        };
        if is_dir {
            name = format!("{}/", name);
            // dir.metadata.len just returns size of os-dir file object thingy, which is not useful
            // its not actually related to size of contents
            size_bytes = None;
        }
        Self {
            path: path,
            is_dir: is_dir,
            name: FileName(name),
            size: size_bytes,
            date: date_modified,
        }
    }
}

// fn read_directory_contents(dir, sort) -> Vec<Entry> // read into sorted list, where sorted list
// will be source of truth in model

// fn read_directory_quickly(dir, sort) -> Vec<PathBuf> // not reading metadata, but only enough to
// display names for an in-progress view. TODO - measure time to read metadata.size/date

fn read_directory_contents(dir: &PathBuf, sort: SortBy) -> HashSet<Entry> {
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
}

// --- UPDATES AND APP LOGIC --- //

fn main() {
    // println!("");
    let final_model = Program {init, view, update}.run();
    // set $FMIN_CWD = final_model.cwd.display()
    // then tell user to alias fmin(*args) { fmin *args; cd $FMIN_CWD }
    // or alias fmin = /thisrepo/script.sh
    //
    // cant pass cwd to stdout since thats where all the TUI bytes go as well

    print!("{}", final_model.cwd.display());
}

fn init() -> Model {
    let cwd = std::env::current_dir().unwrap();
    // let cwd = PathBuf::from("/mnt/c/Users");
    let sort = SortBy{ attribute: EntryAttribute::Name, ascending: true };
    let all_entries = read_directory_contents(&cwd, sort);
    let sorted_entries = sort_entries(&all_entries, sort);
    let (cols, rows) = match terminal::size() {
        Ok((cols, rows)) => (usize::from(cols), usize::from(rows)),
        // TODO: respond to error of not knowing terminal size ("give up, you dont have what fmin
        // needs" ?)
        Err(err) => (0,0)
    };

    // env var setup
    set_var("FMIN_CWD", &cwd.display().to_string());

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
    if !sort.ascending {
        entries_vec.reverse();
    }
    entries_vec
        .into_iter()
        .map(|entry| StringifiedEntry::from(entry) )
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
                        // KeyCode::Char('>') => Action::StartJumpMode, // maybe @ would work?
                        // KeyCode::Char('?') => Action::StartCommandPaletteMode,
                        KeyCode::Backspace => {
                            match m.cwd.parent() {
                                Some(path) => Action::GotoDir(path.to_owned()),
                                None => Action::Noop,
                            }
                        },
                        KeyCode::Char('n') => {
                            match m.cwd_sort.attribute {
                                EntryAttribute::Name => Action::ReverseSort,
                                _ => Action::ChangeSortOrder(EntryAttribute::Name),
                            }
                        },
                        KeyCode::Char('s') => {
                            match m.cwd_sort.attribute {
                                EntryAttribute::Size => Action::ReverseSort,
                                _ => Action::ChangeSortOrder(EntryAttribute::Size),
                            }
                        },
                        KeyCode::Char('m') => {
                            match m.cwd_sort.attribute {
                                EntryAttribute::Date => Action::ReverseSort,
                                _ => Action::ChangeSortOrder(EntryAttribute::Date),
                            }
                        },
                        KeyCode::Enter => Action::SelectFirstEntry,
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
    };
    // update state
    match action {
        Action::GotoDir(pathbuf) => {
            m.all_entries = read_directory_contents(&pathbuf, m.cwd_sort);
            m.sorted_entries = sort_entries(&m.all_entries, m.cwd_sort);
            m.mode = Mode::Filter;
            m.cwd = pathbuf;
            m.cwd_sort = SortBy { attribute: EntryAttribute::Name, ascending: true };
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
            let first = m.all_entries // sorted_entries
                .iter()
                .filter(|entry| entry.name.0.to_lowercase().contains(&m.filter_text.to_lowercase()) )
                .filter(|entry| entry.is_dir )
                .next();
            match first {
                Some(entry) => {
                    m.cwd = entry.path.clone();
                    m.all_entries = read_directory_contents(&entry.path, m.cwd_sort);
                    m.sorted_entries = sort_entries(&m.all_entries, m.cwd_sort);
                    m.mode = Mode::Filter;
                    m.filter_text = "".to_string();
                    m.cwd_sort = SortBy { attribute: EntryAttribute::Name, ascending: true };
                    Some(())
                },
                None => Some(()),
            }
        },
        Action::ChangeSortOrder(attribute) => {
            m.cwd_sort.attribute = attribute;
            m.cwd_sort.ascending = true;
            // m.cwd_sort = SortBy { attribute: attribute, ascending: true };
            m.sorted_entries = sort_entries(&m.all_entries, m.cwd_sort);
            Some(())
        },
        Action::ReverseSort => {
            m.cwd_sort.ascending = !m.cwd_sort.ascending;
            // m.cwd_sort = SortBy { attribute: m.cwd_sort.attribute, ascending: !m.cwd_sort.ascending };
            m.sorted_entries = sort_entries(&m.all_entries, m.cwd_sort);
            Some(())
        },
        Action::StartFilterMode => {
            m.mode = Mode::Filter;
            Some(())
        },
        Action::Noop => Some(()),
        Action::Quit => None,
    }
}

// --- VIEWS AND MESSY STRING HANDLING --- //

fn view(m: &Model, stderr: &mut std::io::Stderr) {
    // view must be impure function writing to mutable buf stderr
    // since crossterm lib puts control bytes in custom types like SetBackgroundColor
    // that must be used here in queue!(buf, ...) function
    // and not postponed for agnostic model/update/view loop
    //
    // maybe i can send a list of crossterm::Commands to queue...
    // but probably not worth making a whole structure of dozens of commands, 
    // only to delay writing to the same place one function later,
    // just for the sake of 'purity'

    // half-declarative view, without implementing a whole ui framework
    // hinges on having only one flex span horiz and vert - rest are static sizes

    // mockup:
    //
    //  C:\users\jkwon\desktop\programming\modenv
    //  ___________________________________________
    //   Name                v | Size   | Modified
    //  ___________________________________________
    //  loopy/                   12       2022-06
    // *droopy/                  4        2022-06
    // *grumpy/                  0        2022-06
    //  frumpy/                  99       2022-06
    // *script1.py               92 KB    2022-06
    //  script_2.py              108 MB   2022-07
    //  main.py                  1.2 GB   2022-05
    //  utils.py                 985 B    2021-12
    // 
    //  (mode) :?!@>/someinputtext        item 2 of 20, and 3 selected
    //  ___________________________________________
    //
    //
    //  consider the sentence as a UI element, as proposed in the essay magic ink (i just read it)
    //  also consider inspiration from other file manager status lines like:
    //  https://raw.githubusercontent.com/ranger/ranger-assets/master/screenshots/multipane.png
    
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
    let divider : &str = &"-".repeat(m.cols);
    #[macro_export]
    macro_rules! divider_at_row {
        ( $row:expr ) => {
            {
                queue!(stderr, MoveTo(0,$row), Print(divider));
            }
        };
    }
    
    view_cwd(m, stderr);
    divider_at_row!(2);
    view_column_headers(m, stderr);
    divider_at_row!(4);
    view_list_body(m, stderr);
    divider_at_row!( (m.rows - 2).try_into().unwrap() );
    view_footer(m, stderr);
}

fn view_cwd(m: &Model, stderr: &mut std::io::Stderr) {
    queue!(stderr, MoveTo(1, 1), fit(&m.cwd.display().to_string(), m.cols));
}

fn view_column_headers(m: &Model, stderr: &mut std::io::Stderr) {
    let name_header = format!(" Name {}", sort_indicator(EntryAttribute::Name, m.cwd_sort));
    let size_header = format!("Size {} ", sort_indicator(EntryAttribute::Size, m.cwd_sort));
    let date_header = format!("  Modified {}  ", sort_indicator(EntryAttribute::Date, m.cwd_sort));
    queue!(stderr, MoveTo(0, 3), 
           fit(&name_header, m.cols - SIZE_MAX_COLS - DATE_MAX_COLS - MARGIN_COLS),
           Print(MARGIN),
           fit(&size_header, SIZE_MAX_COLS),
           fit(&date_header, DATE_MAX_COLS),
    );
}

fn view_list_body(m: &Model, stderr: &mut std::io::Stderr) {
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
        if i == 0 { queue!(stderr, SetBackgroundColor(Color::DarkGrey)); }
        // itemhighlighted?
        queue!(stderr,
               MoveTo(0, (rowNum).try_into().unwrap()),
               Print(" "),
               fit(&entry.name, m.cols - SIZE_MAX_COLS - DATE_MAX_COLS - 2 * MARGIN_COLS),
               Print(MARGIN),
               fit( &pad_align_right(&entry.size, SIZE_MAX_COLS), SIZE_MAX_COLS),
               Print(MARGIN),
               fit(&entry.date, DATE_MAX_COLS),
        );
        if i == 0 { queue!(stderr, ResetColor); }
    }
}

fn view_footer(m: &Model, stderr: &mut std::io::Stderr) {
    queue!(stderr, MoveTo(0, (m.rows - 1).try_into().unwrap()), 
           Print(&format!(" {} {}",
                        match m.mode {
                            Mode::Filter => "(filter)",
                            Mode::Normal => "(normal)",
                        },
                        match m.mode {
                            Mode::Filter => format!(" /{}", m.filter_text),
                            _ => String::new(),
                        },
                        ),
               ),
    );
    match m.mode {
        Mode::Filter => queue!(stderr, crossterm::cursor::Show,),
        _ => queue!(stderr, crossterm::cursor::Hide,),
    };
}

// --- view helpers ---

fn sort_indicator(match_attribute: EntryAttribute, current_sort: SortBy) -> &'static str {
    if match_attribute != current_sort.attribute { return " "; }

    return match current_sort.ascending {
        true => "v",
        false => "^",
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
            pad_align_left(s, final_length)
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

fn pad_align_left(s: &str, final_length: usize) -> String {
    // pad string$0 with : spaces, left aligned <, to meet final_length$1
    format!("{0: <1$}", s, final_length) 
}

fn pad_align_right(s: &str, final_length: usize) -> String {
    format!("{0: >1$}", s, final_length) 
}
