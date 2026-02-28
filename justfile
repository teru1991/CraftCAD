# CraftCAD bootstrap task runner

# TODO: When Qt desktop app is implemented, add desktop-specific targets
#       (e.g. cmake configure/build/test for apps/desktop).
# TODO: When Flutter mobile app is implemented, add mobile-specific targets
#       (e.g. flutter analyze/test/build for apps/mobile).

fmt:
  cargo fmt --all

lint:
  cargo clippy --all-targets --all-features -- -D warnings

test:
  cargo test --all

build:
  cargo build --all

ci: fmt lint test build
