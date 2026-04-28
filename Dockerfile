FROM rust:1.95@sha256:a9cfb755b33f5bb872610cbdb25da61f527416b28fc9c052bbce4bef93e7799a AS builder

WORKDIR /usr/src/ironsubst
COPY . .

RUN cargo build --release --locked

FROM debian:trixie-slim@sha256:cedb1ef40439206b673ee8b33a46a03a0c9fa90bf3732f54704f99cb061d2c5a

# Install any necessary runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/ironsubst/target/release/ironsubst /usr/local/bin/ironsubst

ENTRYPOINT ["ironsubst"]
