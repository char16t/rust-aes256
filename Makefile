build:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy

check-format:
	cargo fmt --check

codecov-ci: clean
	CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test
	grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html

codecov: codecov-ci
	firefox --new-tab --url ./target/coverage/html/index.html

check-codecov: codecov-ci
	@TARGET_COVERAGE=98; \
	ACTUAL_COVERAGE=$$(cat target/coverage/html/badges/flat.svg | egrep '<title>coverage: ' | cut -d: -f 2 | cut -d% -f 1 | sed 's/ //g'); \
	echo "Code Coverage: $$ACTUAL_COVERAGE%"; \
	if [ $$ACTUAL_COVERAGE -lt $$TARGET_COVERAGE ]; then \
		echo "Error: Coverage is below target!"; \
		exit 1; \
	fi

clean:
	rm -rf target/
