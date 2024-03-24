build: src/*.rs
	@cargo build

# open tmux pane showing logs, and run reloading cargo app in original pane
.PHONY: livereload
dev:
	@tmux split-window -v
	@tmux select-pane -l
	@tmux send-keys -t ':.!' 'tail -f /tmp/fmin_log' Enter
	@find src | entr -c cargo run
