#!/bin/bash
export DEBIAN_FRONTEND=noninteractive

# Install dependencies
sudo apt update
sudo apt install build-essential libssl-dev pkg-config

# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Install cranelift
rustup toolchain install nightly
rustup default nightly
rustup component add rustc-codegen-cranelift-preview --toolchain nightly

# Install mold
MOLD_RELEASE=https://github.com/rui314/mold/releases/download/v2.30.0/mold-2.30.0-x86_64-linux.tar.gz
wget -O - "$MOLD_RELEASE" | sudo tar -C /usr/local --strip-components=1 --no-overwrite-dir -xzf -

# Build in background
# cargo +nightly build --profile=fast-dev-linux &
# cargo build --release &