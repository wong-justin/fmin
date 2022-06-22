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

impl<I, V, U> Program<I, V, U> {
    pub fn run<Model>(self) 
    where 
        I: FnOnce() -> Model, 
        V: Fn(&Model) -> String,
        U: Fn(&mut Model, Event) -> Option<()>, 
    {
        let Self {init, view, update} = self;
        let mut stdout = stdout();
        queue!(stdout, terminal::EnterAlternateScreen);

        let mut model = init();
        queue!(stdout, Print(view(&model)),);
        stdout.flush();

        loop {
            let event = await_next_event().unwrap();
            if update(&mut model, event).is_none() {
                break;
            }
            queue!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
            queue!(stdout, MoveTo(0, 0), Print(view(&model)),);
            stdout.flush();
        }
        execute!(stdout, terminal::LeaveAlternateScreen);
    }
}
