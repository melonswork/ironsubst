FROM rust:1.94@sha256:652612f07bfbbdfa3af34761c1e435094c00dde4a98036132fca28c7bb2b165c AS builder

WORKDIR /usr/src/ironsubst
COPY . .

RUN cargo build --release --locked

FROM debian:bookworm-slim

# Install any necessary runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/ironsubst/target/release/ironsubst /usr/local/bin/ironsubst

ENTRYPOINT ["ironsubst"]
