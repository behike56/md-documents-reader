CARGO_MANIFEST := md-documents-reader/Cargo.toml

.PHONY: check fmt fmt-check lint run test

check: fmt-check lint
	cargo check --manifest-path $(CARGO_MANIFEST) --all-targets --all-features

test:
	cargo test --manifest-path $(CARGO_MANIFEST) --all-targets --all-features

fmt:
	cargo fmt --manifest-path $(CARGO_MANIFEST) --all

fmt-check:
	cargo fmt --manifest-path $(CARGO_MANIFEST) --all -- --check

lint:
	cargo clippy --manifest-path $(CARGO_MANIFEST) --all-targets --all-features -- -D warnings

run:
	cargo run --manifest-path $(CARGO_MANIFEST)
