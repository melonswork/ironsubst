FROM rust:latest AS builder

WORKDIR /usr/src/ironsubst
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

# Install any necessary runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/ironsubst/target/release/ironsubst /usr/local/bin/ironsubst

ENTRYPOINT ["ironsubst"]
