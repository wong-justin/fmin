# fmin

a terminal file manager, inspired by fman + vim

looks a little like midnight commander too

## build

rustc --version 1.65.0 or newer

cargo run or cargo build

## roadmap / todo

overall goal 1: quick, convenient directory navigator. then goal 2: file management commands like copy/paste. then stability and edge cases and fine tuning

0. bugfix: nav back and forth leads to different first item selected, seemingly random? to reproduce: /mnt/c, enter on $recycle.bin/, goes to program files, go back then enter again, this time goes to perflogs/, then repeat goes to recycle bin, then config.msi/, ... EDIT - nvm, its because im selecting first of all randomly sorted hashset of entries and not of sorted list i think. need to store sorted entries pathbug, not just stringified. 

1. move up and down in entry list, incl arrow keys in filter mode and j/k in normal mode. which means implementing flexible list view.

2. `open with...` or `open` command. let user define behavior for selecting files (ie. Enter on something that's not a dir, eg. chrome --open-html-file $FMIN_SELECTED). would have to work per file extension types. EDIT - actually would be nice to choose each time, even ignoring file extension, at least as a choice. eg. one time open a .pdf with vim and another time open it in browser. "ctrl+p -> open with... vim()" or "ctrl+p open with minbrowser()". see also windows console commands `start file` which i think does the same as `explorer file`

3. implement jumptodir feature by tracking frecency in flat file db

## other important things to do

- add license

- cd shell session to last dir when exiting fmin. see: https://github.com/dylanaraps/fff?tab=readme-ov-file#fish

- clear jumptodir history eventually.. after 90days? after ranking score is at minimum? give script to clear all with minimum possible ranking score of 1 for user to manually execute (probs as one of the .sh functions to import intop command palette?)

- consider using two env vars $FMIN_CWD and $FMIN_SELECTED that stay updated so user can shell out and use them when needed. maybe also $FMIN_OPEN=myscript.sh as a means to import that important and custom feature. note though that multiline env var values cause problmes with `env` command, so $FMIN_SELECTED cant be newline-separated. see also the terminal file manager that uses env vars for config: https://github.com/dylanaraps/fff?tab=readme-ov-file
which apparently uses a little-known young standard $CDPATH that i might be interested in. EDIT - i dont like cdpath and it seems pretty rare and nonstandard. it's pretty much a manually set home path per session
consider also: $FMIN_DELETE=rm by default, or 'mv $ /trash/'
also: $IFS internal field separator (https://unix.stackexchange.com/questions/184863/what-is-the-meaning-of-ifs-n-in-bash-scripting)

- optmizsation on large dirs, which is important bc slowness in large dirs was one of the main reasons for ditching fman:
-- read_directory_contents(dir, sortby) returns -> Vec<Entry>, creating a list and inserting in sorted order, eg.
all = new vec()
for e in entries:
  for other in vec:
    match sortby.compare(e, other)
      less => vec.insertbefore
      greater | equal => vec.insertafter/insertat

- also to help large dirs go faster: only read filenames first and be ready to display, then if that took a long time (or huersitic if dir has >1000 files) only display filenames with metadata (loading), and iterate thru entries to get metadata and finish displaying with that second step
eg. displaying newly entered directory, part 1
```
___________________________________________
 Name                v | Size   | Modified
___________________________________________
loopy/                     loading...
droopy/                    loading...
grumpy/                    loading...
frumpy/                    loading...
script1.py                 loading...
script_2.py                loading...
main.py                    loading...
utils.py                   loading...
                                            
(this dir has a lot of entries)
___________________________________________
```
second step will read metadata date and size and calc display. also note im doing a math.log() call for size formatting... probably not helping
bonus points if program is still responsive to keypresses, especially between 1st and 2nd steps while metadata is loading
MUST measure performance between both options tho - try creating perf test then git branch to test new implementation

- also consider caching large dir results, and having filewatcher processes knowing when to clear the cache id the dir is modified

- dual pane? or N-pane, with client/server architecture? where server just holds yanked filepaths... kinda overkill. maybe connect with unix pipes? also consider multiplatform... maybe cli option --pair-with-session to opt in to a dual pane? --pair-with-last, --print-all-session-ids, --start-background-server

## nice-to-haves, eventually

- make sure network filesystems work, like google drive or nas'es

- icons, like nerdfont, or custom ascii symbology, just to add redundancy to make identifying files easier (.py, directories, .md, source code, plaintext, binaries, etc)

- consider shift+m like a shift+click on windows, meaning select all from beginning mark up to cursor 

- potential cli/config options: --start-jumping, --config-at, --history-at, --logs--at
along with types like Config::starting_mode/logs_path/...
or consider also zero cli options, and all config happens in env vars

- have logs in the first place

- include sortorder in history file as UX/QOL improvement, so fmin remembers your preferred sort order in each dir

- be able to delete directories in frecency list (rather than opening flat history db file and editing / deleting lines). Note that some dirs temporarily appear and disappear, like USBs, and those should never be deleted automatically just because they arent present at a certain moment

- use docopt for cli options, and maybe just a quick and dirty custom implementation instead of the full library/dsl since that repo sounded problematic and not worth a dependency, and most complexity should live in the tui and not the cli anyways. i just like docopt

- redundant hotkeys: ctrl+j == > or something for jumptodir, ctrl+p == : for command palette, and ctrl+f == / for filter/search. bc control keys are good from any mode, whereas typical vim mode you have to escape back to normal mode before entering another. although reminder to self that many ctrl+key presses are reserved terminal shortcuts, so try not to override them

- provide option to import shell functions, like from a .sh as a config file, like plugins? eg fn unzip() {tar -xzf $FMIN_SELECTED } or whatever, and `unzip` shows up in command palette

- other plugin function ideas: print width/height for img/video files, duration of audio/video files, batch rename selected? (eg. img_01, img_02, etc), copy cwd abs path to clipboard

- for batch rename, consider opening vim/$editor buffer to let user macro their own filename pattern edits. thats what this file manager does in their demo video: https://github.com/sxyazi/yazi

- keep command palette context dependent, eg. show up/down navigation in normal mode, but hide those and show others like esc keybind for filter mode

- add filewatcher to cwd so tui live updates when files are added/removed/modified

- write --help text, full docs in readme, and consider coverting to manpage too

- support --version if it's not already free with crossterm

- do some stress tests, like dir with 1k, 100k, many files (programming/texting/data/cleaned/media is a good real example with 2300 items. also c/windows/sytem32, 5000 entries. also staged / setup test dirs). and dirs with really large files (that seems fine so far tho). then weird unicode filenames. then spamming actions like typing filter text and naving back and forth. also going to weird dirs like recycle bin. see also symlinks. then networked/virtual filesystems.

- support unicode filenames and input text? eh, only once i finish other features that i care about that matter for personal use...

- support rebinding keys? not sure how control characters and letters work on other non-american keyboards... same low priority as above tho

- drag n drop with COM objects on windows? terinal detect mouse hold event -> create COM object for windows-os-level drag n drop -> do something... that would be more of a plugin functionality, and it would take a long time to learn about and hack on COM. although this is kinda important since i do lots of drag n drop in my workflow. but less important if theres a quick `open in native os file explorer` command

- if shortening long strings, consider using unicode char (…) instead of 3 dots(...) since it takes up less space

- also consider shortening abs paths like cwd into abbreviated form, eg. /m/c/u/j/desktop for /mnt/c/users/jkwon/desktop

- also consider shortening /mnt/c/users/jkwon/Desktop to ~/Desktop, and all those home directories, since they get reptitive and dont bring important information for me personally. maybe try that on a feature branch, not master

- any max length to consider when shortening strings? some data points: 80ch historic terminal width. average filename length on my machine __ chars (todo - measure it). max filename length on my machine: __ chars (measure this too). `cal` output width, as an example of skinny output: 20ch for one month (62ch for 3 months). my clock script - ~50ch. and with smallclockchars, probably ~25ch. right now, date field is 14ch and size is 7ch, so name should be >= 14ch too. Or >= 21. so 21 + 7 + 14 + 4ch of margins = 46ch minimum in a sense. still need to shorten paths that are too long tho, and cwds over 40ch

- consider having shortened versions of date and size for tiny terminal sessions?
like use display::CompactWidth/Condensed/Comfortable if name_col is less than size + modified cols

- consider leader key + normal keypress, where user can define leader key, which works well for sxiv tool (see https://youtu.be/GYW9i_u5PY://youtu.be/GYW9i_u5PYs) 

- consider caching Format trait on Date and Size, in case it helps

- looks like windows build has screen flicker each redraw - sad. at least theres always wsl. probably fixable by rewriting view to do partical screen updates instead of redraws top to bottom. also tiny windows interesting thing - looks like terminal height returns one less row than wsl/linux terminal height - maybe windows forces an extra blank line at the end

## other thoughts

- remember to have confirmation step before perofrming action that modifies filesyystem (eg. "move 10 files to new/dir/? [y/n]")
or alternatively make it easy to undo
or alternatively harder to perform on accident, eg. not a simple keypress in normal mode, esp since its easy to think youre in filter mode by mistake
maybe keep those actions limited to command palette with no shortcut? move/copy/delete/rename

- i like how displaying more information lets you release some headspace - dont have to remember if file is big or small; dont have to remember ls -whatevercommands to format file info; seeing big file size in addition to name can help identify file quicker in your brain, same with modified date; always displaying cwd (which ive neglected in my $prompt for the sake of space); perhaps helps when burned out, freeing up mental space and energy

- also enjoy minimal keypresses, esp when coupled with minimal thinking - eg. ctrl+p one step shortcut to jump from any mode to command palette mode, rather than remembering page/modal navigation and esc -> colon:, two steps and extra keypress 

- some command ideas for the palette: move/cut, refresh, copy, delete, sort, new file, new folder, rename, count items in cwd,

- reminder to self that the UI is not as complex because no need to implement linewrapping - keep it that way 

- look at fman issues, both open and closed, to see people's most desired features: https://github.com/fman-users/fman/issues?q=is%3Aissue+sort%3Areactions-%2B1-desc
the main ones:
search (presumably recursively in cwd; low priority for me personally; could be a ls | grep command anyways; and text search is an rgrep command)
batch rename
commmand to compute directory size
feedback on file operations
remember sort order for dirs
undo for commands rename/copy/delete


## feedback; can you help by answering these questions for me?

- what are all the options for reading user-defined shell functions during runtime? see also: https://clig.dev/#configuration
im leaning towards an env var $FMIN_OPEN=script.sh, but that would need a new script for each function i think. not a good idea to parse multiple functions out of a single script - id rather just execute the script and be done with it. 
some tools use .config files, so possibly fmin.config = `open: /path/to/open.sh \nopen in browser: /path/to/browseropen.sh`.
the suckless philosophy would say fork, edit source, and recompile, but that depends on the userhaving a rust dev envirinoment, or me having github actions working properly so its easy to fork and rebuild
bonus points for solutions that dont require extra files in particular locations in particular formats
but rather a zero-file configuration, or one flexible file
maybe command-line options on start? fmin --cmd-open=/path/to/open.sh --cmd-open-with-browser=/path/to/openwithbrowser.sh
that would get ugly quick tho. and poor design because cli options are designed for config that changes often "from one invocation of the command to the next", whereas these shell functions should be reused every launch.

- crossterm lib uses u16 for functions like MoveTo(), etc. but some other funciton elsewhere uses usize. is usize::from()ing the u16s as early as possible, and usize.try_into().unwrap() the best way to go? and is it wrong to prefer the usize as the truer type represnetation of an "unlimited" unsigned int? and has anyone ever needed a terminal sized more than 65536 chars wide/rows tall?

- how do non-american keyboards use vim hotkeys and other ascii char usecases, eg. WASD for games? will those keyboards still be able to input a-z,ctrl+[a-z],shift+[a-z] easily? do power users usually have a qwerty remap layer for these kinds of programs?

- any cleanish, faster alternatives to the model-update-view application loop that avoids writing so many bytes to stdout each update frame? the current way feels a tad slow. or maybe windows terminal is just getting too slow for me personally
