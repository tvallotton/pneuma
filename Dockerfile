FROM rust
RUN cargo install cargo-show-asm
WORKDIR /home/app
COPY . .
CMD ["/bin/sh"]
