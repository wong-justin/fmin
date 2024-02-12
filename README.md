# fmin

a terminal file manager, inspired by fman + vim

# roadmap / todo

- main goal 1: quick, convenient directory navigator. then goal 2: file manager commands like copy/paste

- move up and down in entry list

- implement jumptodir feature by tracking frecency in flat file db

- bugfix: nav back and forth leads to different first item selected, seemingly random? to reproduce: /mnt/c, enter on $recycle.bin/, goes to program files, go back then enter again, this time goes to perflogs/, then repeat goes to recycle bin, then config.msi/, ... EDIT - nvm, its because im selecting first of all randomly sorted hashset of entries and not of sorted list i think. need to store sorted entries pathbug, not just stringified. 

- add license

- cd shell session to last dir when exiting fmin

- make sure network filesystems work, like wsl or google drive or nas'es

- consider shift+m like a shift+click on windows, meaning select all from beginning mark up to cursor 

- potential cli/config options: --start-jumping, --config-at, --history-at, --logs--at
along with types like Config::starting_mode/logs_path/...

- let user define behavior for selecting files (ie. Enter on something that's not a dir, eg. chrome --open-html-file $FMIN_SELECTED). would have to work per file extension types. EDIT - actually would be nice to choose each time, even ignoring file extension, at least as a choice. eg. one time open a .pdf with vim and another open it in browser. "ctrl+p -> open with... vim()" or "ctrl+p open with minbrowser()"

- use docopt for cli options, and maybe just a quick and dirty custom implementation instead of the full library/dsl since that repo sounded problematic and not worth a dependency, and most complexity should live in the tui and not the cli anyways

- redundant hotkeys: ctrl+j == > or something for jumptodir, ctrl+p == : for command palette, and ctrl+f == / for filter/search. bc control keys are good from any mode, whereas typical vim mode you have to escape back to normal mode before entering another

- dual pane? or N-pane, with client/server architecture? where server just holds yanked filepaths... kinda overkill. maybe connect with unix pipes? also consider multiplatform... maybe cli option --pair-with-session to opt in to a dual pane? --pair-with-last, --print-all-session-ids

- normal mode sorting keybinds: N to sort by name, S to sort by size, D to sort by modified...date? shift+n/s/d? or normal n/s/d, and hope people forget that d means delete in vim, and just use x for delete

- consider using two env vars $FMIN_CWD and $FMIN_SELECTED that stay updated so user can shell out and use them when needed

- provide option to import shell functions, like from a .sh as a config file, like plugins? eg fn unzip() {tar -xzf $FMIN_SELECTED } or whatever, and `unzip` shows up in command palette

- other plugin function ideas: print width/height for img/video files, duration of audio/video files, batch rename selected? (eg. img_01, img_02, etc), copy cwd abs path to clipboard

- keep command palette context dependent, eg. show up/down navigation in normal mode, but hide those and show others like esc keybind for filter mode

- add filewatcher to cwd so tui live updates when files are added/removed/modified

- support unicode filenames and input text? eh, only once i finish other features that i care about that matter for personal use...

- support rebinding keys? not sure how control characters and letters work on other non-american keyboards... same low priority as above tho

- drag n drop with COM objects on windows? terinal detect mouse hold event -> create COM object for windows-os-level drag n drop -> do something... that would be more of a plugin functionality, and it would take a long time to learn about and hack on COM

- if shortening long strings, consider using unicode char (…) instead of 3 dots(...) since it takes up less space

- also consider shortening abs paths like cwd into abbreviated form, eg. /m/c/u/j/desktop for /mnt/c/users/jkwon/desktop

- any max length to consider when shortening strings? some data points: 80ch historic terminal width. average filename length on my machine __ chars (todo - script and measure it). max filename length on my machine: __ chars. `cal` output width, as an example of skinny output: 20ch for one month (62ch for 3 months). my clock script - ~50ch. and with smallclockchars, probably ~25ch. right now, date field is 12ch and size is 8ch, so name should be >= 12ch too. so 12 + 12 + 8 + 4ch of margins = 42ch minimum in a sense. still need to shorten paths that are over 12ch tho, and cwds over 40ch

- consider having shortened versions of date and size for tiny terminal sessions?

- reminder to self that the UI is not as complex because no need to implement linewrapping - keep it that way 

- consider leader key + normal keypress, where user can define leader key, which works well for sxiv tool (see https://youtu.be/GYW9i_u5PY://youtu.be/GYW9i_u5PYs) 
