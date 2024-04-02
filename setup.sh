#!/bin/bash

sudo apt update

# Install rust
sudo apt install build-essential libssl-dev pkg-config -y

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y # install unattended
source "$HOME/.cargo/env"

rustup toolchain install nightly
rustup default nightly
rustup component add rustc-codegen-cranelift-preview --toolchain nightly
MOLD_RELEASE=https://github.com/rui314/mold/releases/download/v2.30.0/mold-2.30.0-x86_64-linux.tar.gz
wget -O - "$MOLD_RELEASE" | sudo tar -C /usr/local --strip-components=1 --no-overwrite-dir -xzf -

# build in background
cargo +nightly build --profile=fast-dev-linux &
cargo build --release &

echo Run 'source \"\$HOME/.cargo/env\"'
echo Add a config.toml to run the server
