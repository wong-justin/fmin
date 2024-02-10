// mini framework for application lifecycle and terminal display

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
    pub fn run<Model>(self) 
    where 
        Init: FnOnce() -> Model, 
        View: Fn(&Model, &mut std::io::Stdout),
        Update: Fn(&mut Model, Event) -> Option<()>, 
    {
        let Self {init, view, update} = self;
        let mut stdout = stdout();

        // disables some behavior like line wrapping and catching Enter presses: https://docs.rs/crossterm/latest/crossterm/terminal/index.html#raw-mode
        terminal::enable_raw_mode(); 
        queue!(stdout, 
               terminal::EnterAlternateScreen,
               terminal::DisableLineWrap,
               crossterm::cursor::Hide,
               crossterm::cursor::EnableBlinking, // to signify focus in filter mode; will be hidden anyways in other modes
        );

        let mut model = init();
        view(&model, &mut stdout);
        // display(&mut stdout, view(&model));
        stdout.flush();

        loop {
            let event = await_next_event().unwrap();
            if update(&mut model, event).is_none() {
                break;
            }
            queue!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
            view(&model, &mut stdout);
            // display(&mut stdout, view(&model));
            stdout.flush();
        }

        // cleanup and be a good citizen so the terminal behaves normally afterwards (eg. start catching ctrl+c again, and show cursor)
        execute!(stdout, 
                 terminal::LeaveAlternateScreen,
                 crossterm::cursor::Show,
        );
        terminal::disable_raw_mode(); 
    }
}

// fn display(stdout: &mut std::io::Stdout, content: String) {
//     // queue!(stdout, MoveTo(0, 0), Print(view(&model)),);
// 
//     let mut i = 0;
//     let lines = content.split("\n");
//     for line in lines {
//         queue!(stdout, MoveTo(0, i), Print(line),);
//         i += 1;
//     }
// }
