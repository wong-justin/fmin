build: src/*.rs
	@cargo build

.PHONY: livereload
dev:
	@find src | entr -c cargo run
	# tmux open pane below, and tail -f /tmp/fmin_log
