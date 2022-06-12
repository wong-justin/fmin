#![allow(unused_imports)]

use std::io::{stdout, Write};
use std::fmt;

use crossterm::{
    // macro commands, sending data to stdout or some other buffer
    execute,
    queue,
    // groups of commands that return data, oft special ansi terminal sequences
    cursor,
    terminal,
    event::{
        // block thread until next terminal interaction
        read as awaitNextEvent, 
        // terminal interaction types + data
        Event
    },
    // main() needs this since the macros/functions return a result type
    Result,
};


pub trait Model {
    // Return None to quit the program, else Some(())
    fn update(&mut self, ev: Event) -> Option<()>;
    // Write to stdout
    fn view(&self, buf: &mut impl Write);
}

pub fn start_event_loop(mut model : impl Model) {

    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen).unwrap();

    // optimization idea: view() -> WriteBuf
    // compare new buf to prev buf, iter lines by "\n"
    // only clear lines and write lines that are diff

    model.view(&mut stdout);
    stdout.flush().unwrap();
    loop {
        let ev : Event = awaitNextEvent().unwrap();

        if model.update(ev).is_none() {
            break;
        }

        queue!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
        model.view(&mut stdout);
        stdout.flush().unwrap();
    }
    
    execute!(stdout, terminal::LeaveAlternateScreen);
}


// curr understanding:
//
// printing special chars/strs/escapesequences/data to the terminal makes it do particular actions
//
// one common source of issues: variables must be objects of known size at compile time
// and trait implementers could be objects of any size,
// so you can't have eg. var x = impl MyTrait
// solution:
// use pointers/references instead of objects themselves
// since pointers are of known size
// just be more careful with them i guess?
// and giving a pointer/reference means borrowing memory, and so the lifetime of that
// pointer/reference dies when it's done being borrowed
