.PHONY: run-example
run-example:
	cargo install --path runner --locked
	cd example && cargo test