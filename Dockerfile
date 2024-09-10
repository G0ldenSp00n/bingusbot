FROM rust:1.75

WORKDIR /bingus-bot
COPY . .

RUN cargo build --release

CMD ["./target/release/bingusbot"]
