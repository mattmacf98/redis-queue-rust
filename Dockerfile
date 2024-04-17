FROM rust:1.74

RUN apt-get update -yqq && apt-get install -yqq cmake g++

COPY . .
WORKDIR .

RUN cargo clean
RUN cargo build --release

RUN cp ./target/release/redis-queue-rust ./redis-queue-rust
RUN rm -rf ./target
RUN rm -rf ./src
RUN chmod +x ./redis-queue-rust

EXPOSE 3000
ENTRYPOINT ["./redis-queue-rust"]
