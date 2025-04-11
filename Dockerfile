FROM rust:1.86.0-slim AS builder
WORKDIR /src
COPY . .
RUN apt-get update && apt-get -y install pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo --locked install --path .

FROM debian:12-slim
RUN apt-get update && apt-get -y install libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/gh-token-gen /usr/local/bin/gh-token-gen
RUN /usr/local/bin/gh-token-gen --help
ENTRYPOINT ["/usr/local/bin/gh-token-gen"]
CMD ["--help"]
