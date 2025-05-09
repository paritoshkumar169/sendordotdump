# Builder stage
FROM rust:1.86.0-slim-bullseye AS builder

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev llvm clang make curl && \
    rm -rf /var/lib/apt/lists/*

RUN rustup install nightly && \
    rustup default nightly

RUN cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
ENV PATH="/root/.cargo/bin:$PATH"
RUN avm install 0.31.0 && avm use 0.31.0

WORKDIR /workspace
COPY . .

RUN anchor build

# Runtime stage
FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y libssl-dev curl bzip2 && \
    rm -rf /var/lib/apt/lists/* && \
    curl -sSfL https://release.anza.xyz/stable/solana-release-x86_64-unknown-linux-gnu.tar.bz2 | \
    tar -xj -C /usr/local/bin --strip-components=1

WORKDIR /app
COPY --from=builder /workspace/target/deploy ./target/deploy

ENTRYPOINT ["bash"]
