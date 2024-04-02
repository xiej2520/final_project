FROM rust
WORKDIR /usr/src/final_project
ENV MOLD_RELEASE=https://github.com/rui314/mold/releases/download/v2.30.0/mold-2.30.0-x86_64-linux.tar.gz
RUN rustup toolchain install nightly
RUN rustup default nightly
RUN rustup component add rustc-codegen-cranelift-preview --toolchain nightly
RUN wget -O - "$MOLD_RELEASE" | sudo tar -C /usr/local --strip-components=1 --no-overwrite-dir -xzf -
RUN cargo +nightly build --profile=fast-dev-linux
RUN cargo build --release
CMD ["cargo", "+nightly", "run", "--profile=fast-dev-linux"]