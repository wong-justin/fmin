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

pub enum UpdateResult {
    Continue,
    Finish,
    Failed(String)
}


impl<Init, View, Update> Program<Init, View, Update> {
    pub fn run<Model>(self) -> Result<Model, String>
    where 
        Init: FnOnce() -> Result<Model, String>,
        View: Fn(&Model, &mut std::io::Stderr),
        // update() mutates the model bc I think it's a bit easier and more performant
        //   than creating a new Model in memory on each update
        Update: Fn(&mut Model, Event) -> UpdateResult,
    {
        let Self {init, view, update} = self;
        // write all TUI content to stderr, so on finish, stdout can pass information,
        // like `cd (fmin)`
        let mut stderr = std::io::stderr();

        let mut model = init()?; // quit early here if init fails

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

        view(&model, &mut stderr);
        stderr.flush();

        loop {
            let event = await_next_event().unwrap();
            match update(&mut model, event) {
                UpdateResult::Continue => (),
                UpdateResult::Finish => break,
                UpdateResult::Failed(msg) => {
                    return Err(msg);
                    () // to satisfy compiler return type
                }
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
        terminal::disable_raw_mode(); 
        Ok(model)
    }
}
