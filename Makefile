build:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy

check-format:
	cargo fmt --check

clean:
	rm -rf target/
