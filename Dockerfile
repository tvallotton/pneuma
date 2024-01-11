FROM rust
RUN apt update
RUN apt install valgrind -y
RUN cargo install cargo-show-asm
RUN cargo install cargo-watch
WORKDIR /home/app
COPY . .
CMD ["/usr/local/cargo/bin/cargo", "test"]
