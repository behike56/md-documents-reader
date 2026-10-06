CARGO_MANIFEST := backend/Cargo.toml
FRONTEND_DIR := frontend
TAURI_CLI := ../frontend/node_modules/.bin/tauri

.PHONY: check frontend-check backend-check fmt fmt-check install lint run test

check: frontend-check backend-check

frontend-check:
	npm --prefix $(FRONTEND_DIR) run build

backend-check: fmt-check lint
	cargo check --manifest-path $(CARGO_MANIFEST) --all-targets --all-features

install:
	npm --prefix $(FRONTEND_DIR) ci

test:
	cargo test --manifest-path $(CARGO_MANIFEST) --all-targets --all-features

fmt:
	cargo fmt --manifest-path $(CARGO_MANIFEST) --all

fmt-check:
	cargo fmt --manifest-path $(CARGO_MANIFEST) --all -- --check

lint:
	cargo clippy --manifest-path $(CARGO_MANIFEST) --all-targets --all-features -- -D warnings

run:
	cd backend && $(TAURI_CLI) dev
