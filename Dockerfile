FROM rust:latest

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

EXPOSE 8000
CMD ["server"]