// mini functional framework for application lifecycle and terminal display

#![allow(unused_variables)]
#![allow(unused_imports)]

use std::path::{Path, PathBuf};
use std::fs::DirEntry;
use std::io::{Write};

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

pub enum AppResult {
    Continue,
    Finish,
    FailWithMessage(String),
}

impl<Init, View, Update> Program<Init, View, Update> {
    pub fn run<Model>(self) -> Model
    where 
        Init: FnOnce() -> Model, 
        View: Fn(&Model, &mut std::io::Stderr),
        Update: Fn(&mut Model, Event) -> AppResult, 
    {
        let Self {init, view, update} = self;
        // write all TUI content to stderr, so on finish, stdout can pass information,
        // like `cd (fmin)`
        let mut stderr = std::io::stderr();

        // disables some behavior like line wrapping and catching Enter presses
        // because i will handle those myself
        // https://docs.rs/crossterm/latest/crossterm/terminal/index.html#raw-mode
        terminal::enable_raw_mode(); 
        queue!(stderr, 
               terminal::EnterAlternateScreen,
               terminal::DisableLineWrap,
               crossterm::cursor::Hide,
               crossterm::cursor::EnableBlinking, // for indicating focus of text inputs; cursor will be hidden anyways in other modes
        );

        let mut model = init();
        let mut result = AppResult::Continue;
        view(&model, &mut stderr);
        stderr.flush();

        loop {
            let event = await_next_event().unwrap();
            result = update(&mut model, event);
            match result {
                AppResult::Continue => (),
                AppResult::Finish => break,
                AppResult::FailWithMessage(_) => break,
            };

            view(&model, &mut stderr);
            stderr.flush();
        }

        // cleanup and be a good citizen so the terminal behaves normally afterwards (eg. start catching ctrl+c again, and show cursor)
        execute!(stderr, 
                 terminal::EnableLineWrap,
                 terminal::LeaveAlternateScreen,
                 crossterm::cursor::Show,
        );
        // display failure, if any
        match result {
            AppResult::FailWithMessage(msg) => write!(stderr, "{}", msg),
            _ => std::result::Result::Ok(()) // just to satisfy compiler
        };
        terminal::disable_raw_mode(); 
        return model;
    }
}
