# CraftCAD bootstrap task runner

fmt:
  cargo fmt --all

lint:
  cargo clippy --all-targets --all-features -- -D warnings

test:
  cargo test --all

build:
  cargo build --all

desktop-build:
  ./scripts/build_desktop.sh

ci: fmt lint test build
