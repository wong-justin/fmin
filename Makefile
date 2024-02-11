build: src/*.rs

.PHONY: livereload
dev:
	@find src | entr -c cargo run
