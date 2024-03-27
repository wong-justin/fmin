// fmin, a terminal file manager inspired by fman + vim

#![allow(unused_variables)]
#![allow(unused_imports)]

use std::cmp::Ordering;
use std::collections::{HashSet, HashMap};
use std::fmt::{Display, Formatter, Error};
use std::fs::{DirEntry, File};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use binary_heap_plus::BinaryHeap;
use chrono::{DateTime, Datelike, TimeZone, Local};
use crossterm::{
    terminal,
    queue,
    execute,
    cursor::{MoveTo, MoveToColumn, MoveToRow, MoveToNextLine, MoveToPreviousLine},
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

use crate::tui_program::{Program, UpdateResult};

mod tui_program;

// --- for debugging

struct LoggerToFile<'a> {
    path: &'a str,
}

impl log::Log for LoggerToFile<'_> {
    fn flush(&self) {}
    fn enabled(&self, metadata: &log::Metadata) -> bool { true }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&self.path)
                // not sure what to do when log file breaks
                // either silently fail or crash. i choose crash
                .expect("failure to open or read log file");
            writeln!(file, "{}", record.args());
        }
    }
}

static LOGGER : LoggerToFile = LoggerToFile { path: "/tmp/fmin_log" };

// --- MODEL, and other data structures --- //

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
    // all_entries: HashSet<Entry>,
    sorted_entries: Vec<Entry>,
    filter_text: String,
    cols: usize,
    rows: usize,
    list_view: ListViewData,
    // model.history_filepath file contents should be like:
    // n,1,my/path
    // S,20,other/path
    // m,999,path/can have/spaces/and,weirdchars!too.
    // where n/N/s/S/m/M is sort order, and int is frequency
    //
    // prepend new fields to beginning, and read from end, eg:
    // chunks = line.split(,)
    // path = chunks[-1]
    // freq = chunks[-2]
    // sort = chunks[-3]
    history_filepath: PathBuf,
    // model.history should be like
    // HashMap<String=path, (usize=frequency, FileAttribute=sortoder)>
    // Maybe read history_filepath later to keep startup quick?
    //
    history: HashMap<HistoryPath, HistoryFrequency>,
}

struct Entry {
    path: PathBuf,
    is_dir: bool,
    name: FileName,
    // justification for maybe(date) and maybe(size):
    // dirs dont have filesize - maybe count num of items in dir and display that instead? even recursively? but for now, dir has filesize None
    // also for permission errors or special folders like $recyclebin - probably wont give you
    // any metadata about size or date modified
    // so entries might have filedate None
    size: Option<FileSize>,
    date: Option<FileDate>, 
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
    SelectEntryUnderCursor,
    StartFilterMode,
    ChangeSortOrder(EntryAttribute),
    ReverseSort,
    // StartCommandPaletteMode,
    TryCursorMoveUp,
    TryCursorMoveDown,
    Noop,
    Quit,
}

#[derive(Eq, Hash, PartialEq, Debug)]
struct HistoryRecord {
    path: String,
    frequency: usize,
    // preferred_sort, or last_sort: SortBy,
}

struct FailedToReadHistory;
struct FailedToWriteHistory;

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

#[derive(PartialEq, Eq, Clone)]
struct FileName(String);

#[derive(PartialEq, Clone)]
struct FileSize(u64);

#[derive(PartialEq, Clone)]
struct FileDate(DateTime<Local>);

struct ListViewData {
    items: Vec<Entry>, // Vec<String> row of Entry, row of Command; could be anything stringified
                       // maybe try Vec<ListItem> where ListItem impls display() and on_enter()
    first_viewable_index: usize,
    cursor_index: usize,
    max_items_visible: usize,
    // last_viewable_index = math.min (items.length - 1) , (max_items_visible - first_index)
    // for later: attrs like marked_indexes:Set, 
}

// --- associated behavior for data structures --- //

impl Ord for FileName {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = &self.0;
        let b = &other.0;
        // bad: assumes .chars() will work with unicode filenames
        // and assumes filename has at least one character
        let a_is_dir = a.chars().last().unwrap() == '/';
        let b_is_dir = b.chars().last().unwrap() == '/';

        match (a_is_dir, b_is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.to_string().to_lowercase().cmp(&b.to_string().to_lowercase())
        }
    }
}

impl PartialOrd for FileName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
    // let byte = byte_unit::Byte::from_bytes(self.0.into()).get_appropriate_unit(false);
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

// width of attribute columns, assuming ascii chars
// based on desired formatted output and what looks nice imo.
// used in view functions later on
const SIZE_COLUMN_WIDTH : usize = 7;
const DATE_COLUMN_WIDTH : usize = 14;
const MARGIN_WIDTH : usize = 2;
const MARGIN : &str = "  ";
const NUM_ROWS_OUTSIDE_LISTVIEW : usize = 6;

impl ListViewData {
    // fn new(items: Vec<Entry>) {
    //     return Self {
    //         items: items,
    //         first_viewable_index: 0,
    //         cursor_index: 0,
    //         max_items_visible
    //     }
    // }
    fn reset_with_items(&mut self, items: Vec<Entry>) {
        self.items = items;
        self.cursor_index = 0;
        self.first_viewable_index = 0;
    }
    // fn increment_cursor(&self) -> ListViewData {
    // return Self {
    //  ..self,
    //  cursor_index: new_value
    // }
    fn increment_cursor(&mut self) {
        let last_viewable_index = self.max_items_visible + self.first_viewable_index - 1;

        // at last index, no movement possible
        if self.cursor_index == self.items.len() - 1 {
            // noop
        }
        // at bottom of list, and you can scroll down
        else if self.cursor_index == last_viewable_index {
            self.cursor_index += 1;
            self.first_viewable_index += 1;
        }
        // in middle of list, no need to scroll yet
        else {
            self.cursor_index += 1;
        }
    }
    fn decrement_cursor(&mut self) {
        // at first index, no movement possible
        if self.cursor_index == 0 {
            // noop
        }
        // at top of list, when scrolling is possible
        else if self.cursor_index == self.first_viewable_index {
            self.cursor_index -= 1;
            self.first_viewable_index -= 1;
        }
        // middle of list, no need to scroll yet
        else {
            self.cursor_index -= 1;
        }
    }
    // to listen for cursor move up:
    //   if cursor == 0 then no op
    //   elif cursor == first index then first index --, and last index --, and cursor --
    //   else cursor --
    // and listen for cursor move down:
    //   if cursor == list length then no op
    //   elif cursor == last index then first index++, last index ++, and cursor ++
    //   else cursor ++

    fn set_max_height(&self, num_rows: usize) {
    }
    // for later: fn toggle_mark_under_cursor() {
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

impl std::convert::From<std::io::Error> for FailedToReadHistory {
    fn from(err: std::io::Error) -> Self { FailedToReadHistory }
}

impl std::convert::From<std::num::ParseIntError> for FailedToReadHistory {
   fn from(err: std::num::ParseIntError) -> Self { FailedToReadHistory }
}

impl std::convert::From<std::io::Error> for FailedToWriteHistory {
    fn from(err: std::io::Error) -> Self { FailedToWriteHistory }
}


impl Display for HistoryRecord {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        // example: "1,my/path"
        write!(f, "{},{}", self.frequency, self.path)
    }
}

impl std::convert::TryFrom<String> for HistoryRecord {
    type Error = FailedToReadHistory;

    fn try_from(record_string: String) -> Result<Self, Self::Error> {
        // example of expected record_string: "1,my/path"
        let mut chars = record_string.chars();
        let frequency : usize = chars
            .by_ref()
            .take_while(|&c| c != ',')
            .collect::<String>()
            .parse()?;

        let path : String = chars
            .by_ref()
            .collect::<String>();
        Ok( Self { path: path, frequency: frequency} )
    }
}

fn read_directory_contents_into_sorted(dir: &PathBuf, sort: SortBy) -> Vec<Entry> {
    // optimization idea: replace this fn with
    // read_directory_quickly(dir, sort) -> Vec<PathBuf> 
    // only gets name of entries for an in-progress view, and avoids reading metadata
    // TODO - measure time to read metadata.size/date
    // also TODO - measure to confirm binary heap is actually faster than simple vec

    let mut name_sorted_heap = BinaryHeap::new_by(|a: &Entry, b: &Entry| sort.compare_entries(a, b) );
    
    match dir.read_dir() {
        Ok(dir_entries) => {
            for direntry in dir_entries {
                match direntry {
                    Ok(e) => name_sorted_heap.push(Entry::from(e)),
                    // todo: give better msg if io err happens getting dir entry
                    Err(err) => (),
                };
            }
        },
        // todo: give better msg if io err reading dir
        Err(err) => (),
    };

    // TODO - reverse this if !sort.ascending
    return name_sorted_heap.into_sorted_vec();
}

fn sort_entries(entries: &Vec<Entry>, sort: SortBy) -> Vec<Entry> {
    // let mut new_entries = entries.into_iter().collect::<Vec<&Entry>>();
    let mut new_entries = entries.clone();
    new_entries.sort_by(|a,b| sort.compare_entries(&a, &b));
    if !sort.ascending {
        new_entries.reverse();
    }
    return new_entries;
}

// type HistoryRecord = (String, usize);
type HistoryPath = String;
type HistoryFrequency = usize;

fn read_history_file(filename: &PathBuf) -> Result<HashMap<HistoryPath, HistoryFrequency>, FailedToReadHistory> {
    // input:
    // 123,/my/path
    // 45,/another/path
    // let mut history = HashSet::<HistoryRecord>::new();
    let mut history = HashMap::<HistoryPath, HistoryFrequency>::new();

    let file = File::open(filename)?;

    // line reading taken from:
    // https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html 
    for line in BufReader::new(file).lines() {
        // hope reading line is Ok,
        // and hope parsing line in HistoryRecord::from is Ok
        // else return Err type of this function
        let record = HistoryRecord::try_from(line?)?; 
        history.insert(record.path, record.frequency);
    }
    Ok(history)
}

fn write_history_file(history: HashMap<HistoryPath, HistoryFrequency>, filepath: PathBuf) -> Result<(), FailedToWriteHistory> {
    // output:
    // 123,/my/path
    // 45,/another/path
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&filepath)?;

    let line_separated_records = history
        .iter()
        .map( |(path, freq)| 
            HistoryRecord {
                path: path.to_string(),
                frequency: *freq // from borrowed primitive usize to copied/cloned usize
            }.to_string()
        )
        .collect::<Vec<String>>()
        .join("\n");

    file.write(line_separated_records.as_bytes())?;
    Ok(())
}

fn increment_history(history: &mut HashMap<HistoryPath, HistoryFrequency>, path: HistoryPath) {
    history
        .entry(path)
        .and_modify( |freq| *freq += 1 )
        .or_insert(1);
}

// --- UPDATES AND APP LOGIC --- //

fn main() {
    let program_result = Program {init, view, update}.run();
    match program_result {
        Ok(model) => {
            write!(std::io::stdout(), "{}", model.cwd.display());
            write_history_file(model.history, model.history_filepath);
            // maybe should write_history_file on every cwd update?
        },
        Err(msg) => {
            write!(std::io::stderr(), "Error: {}", msg.to_string());
        }
        // TODO - on err, still write cwd to stdout so parent script won't fail
    };
}

fn init() -> Result<Model, String> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Info));

    log::info!("---\nnew session");

    let cwd = std::env::current_dir().unwrap();
    let sort = SortBy{ attribute: EntryAttribute::Name, ascending: true };
    let sorted_entries = read_directory_contents_into_sorted(&cwd, sort);
    // let (cols, rows) = terminal::size()?;
    let (cols, rows) = match terminal::size() {
        Ok((cols, rows)) => (usize::from(cols), usize::from(rows)),
        Err(_) => return Err("can't read terminal size".to_string()),
    };
    let list_view = ListViewData {
        items: sorted_entries.clone(),
        first_viewable_index: 0,
        cursor_index: 0,
        max_items_visible: rows - NUM_ROWS_OUTSIDE_LISTVIEW,
    };


    // Thoughts on dotfiles, env vars, and related conventions:
    //
    // I personally don't like the XDG basedir spec (https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
    //
    // 1) the XDG spec defines several directories, including:
    //    $XDG_DATA_HOME, XDG_CONFIG_HOME, STATE_HOME
    //    According to the spec, fmin should use most of those dirs
    //    (write selected entries to STATE_HOME/.fmin_selected_filepaths,
    //     keep history in DATA_HOME/.fmin_history, 
    //     config file at CONFIG_HOME/.fminrc,
    //     and use CACHE_HOME if I ever get that serious)
    //    Too many directories for a tiny tool like fmin, imo.
    //    Twould be like having `ls` vomit four files all over your system.
    //    Not to mention my personal preference for portable, self-contained app directories.
    //    But that's just me.
    //
    // 2) the XDG spec also defines fallbacks for each env var, eg. 
    //    "if $XDG_CONFIG_HOME is not set, a default equal to $HOME/.config should be used."
    //    Which means the spec defines what to do if you don't follow the spec...
    //    Or in other words, you can't fully support the XDG spec and non-XDG fallbacks 
    //      at the same time.
    //    That's a less-important feature of the spec, tho. Easy to ignore.
    //
    // 3) Some big old command-line tools have publicly stated they will never 
    //    break compatibility to support XDG; take $HOME/.bashrc, for example.
    //    Other tools half-heartedly support the XDG spec, 
    //      and just dump everything to $XDG_CONFIG_HOME if it's set,
    //        and hope that's enough to satisfy XDG-enjoyers.
    //    Still other tools don't even bother with the XDG spec.
    //    Very few implement the spec 100%.
    //
    // I think users mainly care about keeping $HOME clean
    // and keeping config files in one place / easy to backup.
    //
    // But supporting the XDG spec is only a few extra LOC. So I might as well do it.
    // For now, that means putting .fmin_history in $XDG_DATA_HOME if it's set.
    //
    // - fmin provides an alternative data home var like $FMIN_HOME,
    //   meant for the users that want neither XDG nor naked $HOME.
    //   I'm not sure if every tool should have a custom env var home,
    //   but it feels reasonable to me for now.
    //   Everything goes in one folder, which is at worst easy to understand and control
    //
    // - if those env vars are unset, then do the typical $HOME/.dotfile behavior,
    //   because everyone is used to that,
    //   and picky people should have developed workarounds by now.
    //
    // Ultimately, there will always be tools with dotfiles in $HOME,
    // and there will always be tools half-heartedly implementing the XDG spec,
    // whether or not anyone likes it, and whether or not it's a good spec.
    //
    // So I think the best behavior is educating users,
    // embracing long-running conventions instead of breaking backwards-compatiblity,
    // giving users an escape hatch for customization and control over any dotfile
    //   (see symlinks,
    //    and symlink management tools like gnu stow,
    //    and other config-mangement options like
    //    leaving ~/ for config while using another dir like ~/mycleanhome/ for personal use,
    //    or perhaps using cli options --config if possible)
    // and minimizing the amount of config needed in the first place.

    let data_dir = match(
        std::env::var("FMIN_HOME"),
        std::env::var("XDG_DATA_HOME"),
        std::env::var("HOME"), // TODO - replace with windows env var for home? %USERHOME% or whatver?
    ) {
        ( Ok(fmin_home), _, _) => PathBuf::from(fmin_home),
        ( Err(_), Ok(xdg_config_home), _ ) => PathBuf::from(xdg_config_home),
        ( Err(_), Err(_), Ok(home) ) => PathBuf::from(home).join(".fmin/"),
        _ => return Err("need to set directory: either $FMIN_HOME, $XDG_DATA_HOME, or $HOME".to_string()),
    };

    const HISTORY_FILENAME : &str = ".fmin_history";
    let history_filepath = data_dir.join(HISTORY_FILENAME);
    let mut history = match read_history_file(&history_filepath) {
        Ok(records) => records,
        Err(FailedToReadHistory) => HashMap::<HistoryPath, HistoryFrequency>::new(),
    };
    // or later, in Action::GotoMode:
    // let history = match m.history {
    //     NotInitialized => read_history_file()
    //     Loaded(records) => 
    //     Failed => ...
    // }
    log::info!("{:?}", history);

    increment_history(&mut history, cwd.display().to_string());
 
    Ok(Model {
        cwd: cwd,
        cwd_sort: sort,
        sorted_entries: sorted_entries,
        filter_text: "".to_string(),
        mode: Mode::Filter,
        cols: cols,
        rows: rows,
        list_view: list_view,
        history_filepath: history_filepath,
        history: history,
    })
}

fn update(m: &mut Model, terminal_event: Event) -> UpdateResult {
    // exit early if ctrl+c, no matter what
    // returning None means to quit the program
    // TODO - have a better return type than None/Some(())

    match terminal_event {
        Event::Key(keyevent) => {
            if
                keyevent.modifiers == KeyModifiers::CONTROL &&
                keyevent.code == KeyCode::Char('c')
            {
                return UpdateResult::Finish;
            }
        },
        Event::Resize(cols, rows) => {
            m.cols = usize::from(cols);
            m.rows = usize::from(rows);
            m.list_view.max_items_visible = m.rows - NUM_ROWS_OUTSIDE_LISTVIEW;
        },
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
                            // Action::ChangeSortOrder(SortBy{ 
                            //     attribute: EntryAttribute::Name,
                            //     ascending: match m.cwd_sort.attribute == EntryAttribute::Name {
                            //         EntryAttribute::Name => !m.cwd_sort.ascending,
                            //         _ => true,
                            //     }
                            // }),
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
                        KeyCode::Char('k') | KeyCode::Up => Action::TryCursorMoveUp,
                        KeyCode::Char('j') | KeyCode::Down => Action::TryCursorMoveDown,
                        KeyCode::Enter => Action::SelectEntryUnderCursor,
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
                    match keyevent.modifiers {
                        KeyModifiers::SHIFT => match keyevent.code {
                            KeyCode::Char('K') => Action::TryCursorMoveUp,
                            KeyCode::Char('J') => Action::TryCursorMoveDown,
                            KeyCode::Char('N') => match m.cwd_sort.attribute {
                                EntryAttribute::Name => Action::ReverseSort,
                                _ => Action::ChangeSortOrder(EntryAttribute::Name),
                            },
                            KeyCode::Char('S') => match m.cwd_sort.attribute {
                                EntryAttribute::Size => Action::ReverseSort,
                                _ => Action::ChangeSortOrder(EntryAttribute::Size),
                            },
                            KeyCode::Char('M') => match m.cwd_sort.attribute {
                                EntryAttribute::Date => Action::ReverseSort,
                                _ => Action::ChangeSortOrder(EntryAttribute::Date),
                            },
                            // KeyCode::Char('O') => Action::StartJumpMode,
                            // KeyCode::Char('P') => Action::StartCommandPaletteMode,
                            KeyCode::Char('Q') => Action::Quit,
                            _ => Action::Noop,
                        },
                        _ => match keyevent.code {
                            // todo: listen for end-of-input ctrl+d, arrow keys left and right, paste
                            // ideally a readline lib is used to listen for input here, but
                            // - dont need history or multiline editing, and probably not vim/emacs shortcuts
                            // - not sure how to include static .so external dependency
                            // so probably not worth including that extra dependency
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
                            KeyCode::Up => {
                                Action::TryCursorMoveUp
                            },
                            KeyCode::Down => {
                                Action::TryCursorMoveDown
                            },
                            KeyCode::Enter => Action::SelectEntryUnderCursor,
                            _ => Action::Noop,
                        }
                    }
                },
                _ => Action::Noop,
            }
        },
    };
    // update state
    match action {
        Action::GotoDir(pathbuf) => {
            m.cwd_sort = SortBy { attribute: EntryAttribute::Name, ascending: true };
            m.sorted_entries = read_directory_contents_into_sorted(&pathbuf, m.cwd_sort);
            m.cwd = pathbuf;
            m.mode = Mode::Filter;
            m.list_view.reset_with_items(m.sorted_entries.clone());
            increment_history(&mut m.history, m.cwd.display().to_string());
            // m.history = increment_history(m.history, pathbuf);
            UpdateResult::Continue
        },
        Action::SetFilterText(text) => {
            // m.mode = match text.is_empty() {
            //     true => Mode::Normal,
            //     false => Mode::Filter,
            // };
            m.filter_text = text;

            let filtered_entries =  m.sorted_entries
                .clone()
                .into_iter()
                .filter(|entry| entry.name.0.to_lowercase().contains(&m.filter_text.to_lowercase()) )
                .collect::<Vec<Entry>>();

            m.list_view.reset_with_items(filtered_entries);
            UpdateResult::Continue
        },
        Action::SelectEntryUnderCursor => {
            // if no cursor, cant do anything
            if m.list_view.items.len() == 0 { return UpdateResult::Continue; }

            let entry = &m.list_view.items[m.list_view.cursor_index];

            if entry.is_dir {
                m.cwd = entry.path.clone();
                m.sorted_entries = read_directory_contents_into_sorted(&entry.path, m.cwd_sort);
                m.mode = Mode::Filter;
                m.filter_text = "".to_string();
                m.cwd_sort = SortBy { attribute: EntryAttribute::Name, ascending: true };
                increment_history(&mut m.history, m.cwd.display().to_string());
                m.list_view.reset_with_items(m.sorted_entries.clone());
            }
            UpdateResult::Continue
        },
        Action::ChangeSortOrder(attribute) => {
            // TODO - consider preserving hovered entry by finding it in the new sorted vec,
            // and updating cursor_index accordingly,
            // instead of just resetting to top
            m.cwd_sort.attribute = attribute;
            m.cwd_sort.ascending = true;
            m.sorted_entries = sort_entries(&m.sorted_entries, m.cwd_sort);
            m.list_view.reset_with_items(m.sorted_entries.clone());
            UpdateResult::Continue
        },
        Action::ReverseSort => {
            m.cwd_sort.ascending = !m.cwd_sort.ascending;
            m.sorted_entries = sort_entries(&m.sorted_entries, m.cwd_sort);
            m.list_view.reset_with_items(m.sorted_entries.clone());
            UpdateResult::Continue
        },
        Action::TryCursorMoveUp => {
            m.list_view.decrement_cursor();
            UpdateResult::Continue
        },
        Action::TryCursorMoveDown => {
            m.list_view.increment_cursor();
            UpdateResult::Continue
        },
        Action::StartFilterMode => {
            m.mode = Mode::Filter;
            UpdateResult::Continue
        },
        Action::Noop => UpdateResult::Continue,
        Action::Quit => UpdateResult::Finish,
    }
}

// --- VIEWS AND MESSY STRING HANDLING --- //

fn view(m: &Model, stderr: &mut std::io::Stderr) {
    // half-declarative view, without implementing a whole ui framework
    // hinges on having only one flex span horiz and vert - rest are static sizes
    //
    // view must be impure function writing to mutable buf stderr
    // since crossterm lib puts control bytes in custom types like SetBackgroundColor
    // so view happens with mutable in queue!(buf, ...) function
    // and not postponed for agnostic model/update/view loop
    //
    // maybe i can send a list of crossterm::Commands to queue...
    // but probably not worth making a whole structure of dozens of commands, 
    // only to delay writing to the same place one function later,
    // just for the sake of 'purity'

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
    
    let divider : &str = &"-".repeat(m.cols);
    let spacer : &str = &" ".repeat(m.cols);
    #[macro_export]
    macro_rules! divider {
        () => {
            queue!(stderr, Print(divider), MoveToNextLine(1));
        };
    }
    #[macro_export]
    macro_rules! empty_line {
        () => {
            queue!(stderr, Print(spacer), MoveToNextLine(1));
        };
    }
    queue!(stderr, crossterm::cursor::Hide);

    view_cwd(m, stderr);            // height = 1 // 2
    divider!();                     // height = 1
    view_column_headers(m, stderr); // height = 1
    divider!();                     // height = 1
    view_list_body(m, stderr);      // height = m.rows - 8
    // empty_line!();                  // height = 1
    divider!();                     // height = 1
    view_footer(m, stderr);         // height = 1
    // empty_line!();                  // height = 1
}

fn view_cwd(m: &Model, stderr: &mut std::io::Stderr) {
    queue!(stderr,
           MoveTo(0,0),
           // Print(" ".repeat(m.cols)), 
           // MoveToNextLine(1),
           fit(&format!(" {}", m.cwd.display()), m.cols),
           MoveToNextLine(1)
    );
}

fn view_column_headers(m: &Model, stderr: &mut std::io::Stderr) {
    let name_header = format!(" Name {}", sort_indicator(EntryAttribute::Name, m.cwd_sort));
    let size_header = format!("Size {} ", sort_indicator(EntryAttribute::Size, m.cwd_sort));
    let date_header = format!("  Modified {}  ", sort_indicator(EntryAttribute::Date, m.cwd_sort));
    queue!(stderr, 
           fit(&name_header, m.cols - SIZE_COLUMN_WIDTH - DATE_COLUMN_WIDTH - MARGIN_WIDTH),
           Print(MARGIN),
           fit(&size_header, SIZE_COLUMN_WIDTH),
           fit(&date_header, DATE_COLUMN_WIDTH),
           MoveToNextLine(1)
    );
}

fn view_list_body(m: &Model, stderr: &mut std::io::Stderr) {
    // example of displaying list_view.items and indexes:
    //
    // all items indexes  
    // on left
    //                    
    // out of   0 
    // view     -----    viewable indexes
    //          1   0    on right           
    //          2   1           
    //          3   2  
    //          4   3
    //          -----
    // out of   5
    // view
    //
    // first_viewable_index = 1
    // max_items_visible = 4

    let viewable_entries = m.list_view.items.iter()
        .skip(m.list_view.first_viewable_index)
        .take(m.list_view.max_items_visible); 

    for (visible_index, entry) in viewable_entries.enumerate() {
        let name = &entry.name.0.clone();
        let size = match &entry.size {
            Some(size_bytes) => size_bytes.to_string(),
            None => String::new(),
        };
        let date = match &entry.date {
            Some(date_modified) => date_modified.to_string(),
            None => String::new(),
        };

        let at_cursor = m.list_view.cursor_index == visible_index + m.list_view.first_viewable_index;
        if at_cursor { queue!(stderr, SetBackgroundColor(Color::DarkGrey)); }

        queue!(stderr,
               Print(" "),
               fit(&name, m.cols - SIZE_COLUMN_WIDTH - DATE_COLUMN_WIDTH - 2 * MARGIN_WIDTH),
               Print(MARGIN),
               fit( &pad_align_right(&size, SIZE_COLUMN_WIDTH), SIZE_COLUMN_WIDTH),
               Print(MARGIN),
               fit(&date, DATE_COLUMN_WIDTH),
               MoveToNextLine(1),
        );

        if at_cursor { queue!(stderr, ResetColor); }
    }

    // draw over any empty rows
    if m.list_view.max_items_visible > m.list_view.items.len() {
        let empty_rows = m.list_view.max_items_visible - m.list_view.items.len();

        for _ in 0..empty_rows {
            queue!(stderr, Print(" ".repeat(m.cols)), MoveToNextLine(1));
        }
    }
}

fn view_footer(m: &Model, stderr: &mut std::io::Stderr) {
    queue!(stderr, 
           // clear any artifacts from previous draw
           Print(" ".repeat(m.cols)),
           MoveToColumn(1),
           // display filter field
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

// --- view helpers --- //

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
    // bad: assuming only ASCII input...
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
    // bad: assuming only ASCII input...
    return s.as_ref().chars().count();
}

fn pad_align_left(s: &str, final_length: usize) -> String {
    // pad string$0 with : spaces, left aligned <, to meet final_length$1
    format!("{0: <1$}", s, final_length) 
}

fn pad_align_right(s: &str, final_length: usize) -> String {
    format!("{0: >1$}", s, final_length) 
}
