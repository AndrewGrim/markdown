debug:
	cargo build && cargo test -- --show-output
release:
	cargo build --release && strip target/release/md.exe