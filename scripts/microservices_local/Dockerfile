FROM rust:latest

ARG FEATURES=""

WORKDIR /usr/src/app

COPY ./src ./src
COPY *.toml .

RUN cargo install ${FEATURES:+--features $FEATURES} --path .

EXPOSE 80
CMD ["server"]
