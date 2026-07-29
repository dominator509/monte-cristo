# EP-009 M2: Multi-stage Docker build for Monte Cristo.
#
# Builds the game and tools in a Rust builder image,
# then copies the release binary into a minimal runtime image.

# ── Builder stage ──────────────────────────────────────────────────────────────
FROM rust:1.83.0-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libgl-dev libasound2-dev libudev-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock deny.toml rust-toolchain.toml ./
COPY crates/ ./crates/
COPY content/ ./content/
COPY scripts/ ./scripts/
COPY tapes/ ./tapes/
COPY .agent/ ./.agent/

# Build release binary
RUN cargo build --release --locked -p mc_tools

# ── Runtime stage ──────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libgl1 libasound2 libudev1 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mc_tools /usr/local/bin/
COPY --from=builder /app/content /app/content
COPY --from=builder /app/tapes /app/tapes

WORKDIR /app
ENTRYPOINT ["mc_tools"]
CMD ["--help"]
