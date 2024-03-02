// mini functional framework for application lifecycle and terminal display

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

pub struct Program<Init, View, Update> {
    pub init: Init,
    pub view: View,
    pub update: Update,
}

impl<Init, View, Update> Program<Init, View, Update> {
    pub fn run<Model>(self) -> Model
    where 
        Init: FnOnce() -> Model, 
        View: Fn(&Model, &mut std::io::Stdout),
        Update: Fn(&mut Model, Event) -> Option<()>, 
    {
        let Self {init, view, update} = self;
        // if write all TUI content to stdout:
        // - nice redraws
        // - cannot print anything to stdout on app finish
        // if write all TUI content to stderr:
        // - cwd can be printed to stdout on app finish for easy scripting: `cd (fmin)`
        // - but redraws flash between frames (on windows terminal at least)
        let mut stdout = stdout();

        // disables some behavior like line wrapping and catching Enter presses
        // because i will handle those myself
        // https://docs.rs/crossterm/latest/crossterm/terminal/index.html#raw-mode
        terminal::enable_raw_mode(); 
        queue!(stdout, 
               terminal::EnterAlternateScreen,
               terminal::DisableLineWrap,
               crossterm::cursor::Hide,
               crossterm::cursor::EnableBlinking, // for indicating focus of text inputs; cursor will be hidden anyways in other modes
        );

        let mut model = init();
        view(&model, &mut stdout);
        stdout.flush();

        loop {
            let event = await_next_event().unwrap();
            if update(&mut model, event).is_none() {
                break;
            }
            queue!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
            view(&model, &mut stdout);
            stdout.flush();
        }

        // cleanup and be a good citizen so the terminal behaves normally afterwards (eg. start catching ctrl+c again, and show cursor)
        execute!(stdout, 
                 terminal::EnableLineWrap,
                 terminal::LeaveAlternateScreen,
                 crossterm::cursor::Show,
        );
        terminal::disable_raw_mode(); 
        return model;
    }
}
