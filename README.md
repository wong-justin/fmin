# fmin

a terminal file manager, inspired by fman + vim

looks a little like midnight commander too

# roadmap / todo

- main goal 1: quick, convenient directory navigator. then goal 2: file manager commands like copy/paste

- bugfix: nav back and forth leads to different first item selected, seemingly random? to reproduce: /mnt/c, enter on $recycle.bin/, goes to program files, go back then enter again, this time goes to perflogs/, then repeat goes to recycle bin, then config.msi/, ... EDIT - nvm, its because im selecting first of all randomly sorted hashset of entries and not of sorted list i think. need to store sorted entries pathbug, not just stringified. 

- align byte size output units (right align i guess)

## most useful commands from fman to replicate imo:

- move up and down in entry list, incl arrow keys in filter mode and j/k in normal mode

- open with... or open. let user define behavior for selecting files (ie. Enter on something that's not a dir, eg. chrome --open-html-file $FMIN_SELECTED). would have to work per file extension types. EDIT - actually would be nice to choose each time, even ignoring file extension, at least as a choice. eg. one time open a .pdf with vim and another open it in browser. "ctrl+p -> open with... vim()" or "ctrl+p open with minbrowser()". see also windows console commands `start file` which i think does the same as `explorer file`

- implement jumptodir feature by tracking frecency in flat file db

- dual pane? or N-pane, with client/server architecture? where server just holds yanked filepaths... kinda overkill. maybe connect with unix pipes? also consider multiplatform... maybe cli option --pair-with-session to opt in to a dual pane? --pair-with-last, --print-all-session-ids

## other important things to do

- add license

- cd shell session to last dir when exiting fmin

- clear jumptodir history eventually.. after 90days? after ranking score is at minimum? give script to clear all with minimum possible ranking score of 1 for user to manually execute (probs as one of the .sh functions to import intop command palette?)

- consider using two env vars $FMIN_CWD and $FMIN_SELECTED that stay updated so user can shell out and use them when needed. maybe also $FMIN_OPEN=myscript.sh as a means to import that important and custom feature.

## nice-to-haves

- make sure network filesystems work, like wsl or google drive or nas'es

- icons, like nerdfont, or custom ascii symbology, just to add redundancy to make identifying files easier (.py, directories, .md, source code, plaintext, binaries, etc)

- consider shift+m like a shift+click on windows, meaning select all from beginning mark up to cursor 

- potential cli/config options: --start-jumping, --config-at, --history-at, --logs--at
along with types like Config::starting_mode/logs_path/...

- use docopt for cli options, and maybe just a quick and dirty custom implementation instead of the full library/dsl since that repo sounded problematic and not worth a dependency, and most complexity should live in the tui and not the cli anyways

- redundant hotkeys: ctrl+j == > or something for jumptodir, ctrl+p == : for command palette, and ctrl+f == / for filter/search. bc control keys are good from any mode, whereas typical vim mode you have to escape back to normal mode before entering another

- normal mode sorting keybinds: N to sort by name, S to sort by size, D to sort by modified...date? shift+n/s/d? or normal n/s/d, and hope people forget that d means delete in vim, and just use x for delete

- provide option to import shell functions, like from a .sh as a config file, like plugins? eg fn unzip() {tar -xzf $FMIN_SELECTED } or whatever, and `unzip` shows up in command palette

- other plugin function ideas: print width/height for img/video files, duration of audio/video files, batch rename selected? (eg. img_01, img_02, etc), copy cwd abs path to clipboard

- keep command palette context dependent, eg. show up/down navigation in normal mode, but hide those and show others like esc keybind for filter mode

- add filewatcher to cwd so tui live updates when files are added/removed/modified

- support unicode filenames and input text? eh, only once i finish other features that i care about that matter for personal use...

- support rebinding keys? not sure how control characters and letters work on other non-american keyboards... same low priority as above tho

- drag n drop with COM objects on windows? terinal detect mouse hold event -> create COM object for windows-os-level drag n drop -> do something... that would be more of a plugin functionality, and it would take a long time to learn about and hack on COM. although this is kinda important since i do lots of drag n drop in my workflow. but less important if theres a quick `open in native os file explorer` command

- if shortening long strings, consider using unicode char (…) instead of 3 dots(...) since it takes up less space

- also consider shortening abs paths like cwd into abbreviated form, eg. /m/c/u/j/desktop for /mnt/c/users/jkwon/desktop

- also consider shortening /mnt/c/users/jkwon/Desktop to ~/Desktop, and all those home directories, since they get reptitive and dont bring important information for me personally. maybe make that my customized branch

- any max length to consider when shortening strings? some data points: 80ch historic terminal width. average filename length on my machine __ chars (todo - script and measure it). max filename length on my machine: __ chars. `cal` output width, as an example of skinny output: 20ch for one month (62ch for 3 months). my clock script - ~50ch. and with smallclockchars, probably ~25ch. right now, date field is 12ch and size is 8ch, so name should be >= 12ch too. so 12 + 12 + 8 + 4ch of margins = 42ch minimum in a sense. still need to shorten paths that are over 12ch tho, and cwds over 40ch

- consider having shortened versions of date and size for tiny terminal sessions?

- reminder to self that the UI is not as complex because no need to implement linewrapping - keep it that way 

- consider leader key + normal keypress, where user can define leader key, which works well for sxiv tool (see https://youtu.be/GYW9i_u5PY://youtu.be/GYW9i_u5PYs) 

# other thoughts

- i like how displaying more information lets you release some headspace - dont have to remember if file is big or small; dont have to remember ls -whatevercommands to format file info; seeing big file size in addition to name can help identify file quicker in your brain, same with modified date; always displaying cwd (which ive neglected in my $prompt for the sake of space); perhaps helps when burned out, freeing up mental space and energy

- also enjoy minimal keypresses, esp when coupled with minimal thinking - eg. ctrl+p one step shortcut to jump from any mode to command palette mode, rather than remembering page/modal navigation and esc -> colon:, two steps and extra keypress 

- some command ideas for the palette: move/cut, refresh, copy, delete, sort, new file, new folder, rename

# feedback; can you help by answering these questions for me?

- crossterm lib uses u16 for functions like MoveTo(), etc. but some other funciton elsewhere uses usize. is usize::from()ing the u16s as early as possible, and usize.try_into().unwrap() the best way to go? and is it wrong to prefer the usize as the truer type represnetation of an "unlimited" unsigned int? and has anyone ever needed a terminal sized more than 65536 chars wide/rows tall?

- how do non-american keyboards use vim hotkeys and other ascii char usecases like WASD for games? will those keyboards still be able to input a-z,ctrl+[a-z],shift+[a-z] easily? do the power users that care usually have a remap layer to use these kinds of programs?

- any cleanish, faster alternatives to the model-update-view application loop that avoids writing so many bytes to stdout each update frame? the current way feels a tad slow. or maybe windows terminal is just getting too slow for me personally
