# fmin

a terminal file manager, inspired by fman + vim

# roadmap / todo

- potential cli/config options: --start-jumping, --config-at, --history-at, --logs--at
along with types like Config::starting_mode/logs_path/...

- use docopt for cli options, and maybe just a quick and dirty custom implementation instead of the full library/dsl since that repo sounded problematic and not worth a dependency, and most complexity should live in the tui and not the cli anyways

- redundant hotkeys: ctrl+j == > or something for jumptodir, ctrl+p == : for command palette, and ctrl+f == / for filter/search. bc control keys are good from any mode, whereas typical vim mode you have to escape back to normal mode before entering another

- implement jumptodir feature by tracking frecency in flat file db

- dual pane? or N-pane, with client/server architecture? where server just holds yanked filepaths... kinda overkill. maybe connect with unix pipes? also consider multiplatform... maybe cli option --pair-with-session to opt in to a dual pane?

- normal mode sorting keybinds: N to sort by name, S to sort by size, D to sort by modified...date? shift+n/s/d? or normal n/s/d, and hope people forget that d means delete in vim, and just use x for delete

- consider using two env vars $FMIN_CWD and $FMIN_SELECTED that stay updated so user can shell out and use them when needed

- provide option to import shell functions, like from a .sh as a config file, like plugins? eg fn unzip() {tar -xzf $FMIN_SELECTED } or whatever, and `unzip` shows up in command palette

- other plugin function ideas: print width/height for img/video files, duration of audio/video files, batch rename selected? (eg. img_01, img_02, etc), copy cwd abs path to clipboard

- keep command palette context dependent, eg. show up/down navigation in normal mode, but hide those and show others like esc keybind for filter mode

- support unicode filenames and input text? eh, only once i finish other features that i care about that matter for personal use...

- support rebinding keys? not sure how control characters and letters work on other non-american keyboards... same low priority as above tho

- drag n drop with COM objects on windows? more of a plugin functionality, and it would take a long time to learn about and hack on COM

- if shortening long strings, consider using unicode char (…) instead of 3 dots(...) since it takes up less space

- reminder to self that the UI is not as complex because no need to implement linewrapping - keep it that way 

- cd shell session to last dir when exiting fmin

- consider leader key + normal keypress, where user can define leader key, which works well for sxiv tool (see https://youtu.be/GYW9i_u5PY://youtu.be/GYW9i_u5PYs) 
