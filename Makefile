build: src/*.rs
	@cargo build

.PHONY: livereload
dev:
	@find src | entr -c cargo run
