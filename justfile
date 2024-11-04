list:
  just --list

format:
  cargo fmt --all

build:
  cargo build --all --all-features

test:
  cargo test --all --all-features

miri:
  cargo +nightly miri test --manifest-path ./crates/ankha/Cargo.toml
  cargo +nightly miri test --manifest-path ./crates/auri/Cargo.toml

clippy:
  cargo clippy --all --all-features
  cargo clippy --tests --all --all-features

checks:
  just format
  just build
  just clippy
  just test
#   just miri

demo-calcite:
  cd ./demos/calcite && cargo run

clean:
  find . -name target -type d -exec rm -r {} +
  just remove-lockfiles

remove-lockfiles:
  find . -name Cargo.lock -type f -exec rm {} +

list-outdated:
  cargo outdated -R -w

update:
  cargo update --manifest-path ./crates/ankha/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/auri/Cargo.toml --aggressive

publish:
  cargo publish --no-verify --manifest-path ./crates/ankha/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./crates/auri/Cargo.toml
