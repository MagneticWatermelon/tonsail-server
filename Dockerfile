FROM rust:1.66.0 AS builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
RUN cargo prisma generate
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tonsail-server tonsail-server
COPY configuration configuration
ENTRYPOINT [ "./tonsail-server" ]

# FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
# WORKDIR /app
#
# FROM chef AS planner
# COPY . .
# RUN cargo chef prepare --recipe-path recipe.json
#
# FROM chef AS builder 
# COPY --from=planner /app/recipe.json recipe.json
# # Build dependencies - this is the caching Docker layer!
# RUN cargo chef cook --release --recipe-path recipe.json
# # Build application
# COPY . .
# RUN cargo prisma generate
# RUN cargo build --release
#
# # We do not need the Rust toolchain to run the binary!
# FROM debian:buster-slim AS runtime
# WORKDIR /app
# COPY --from=builder /app/target/release/tonsail-server tonsail-server
# COPY configuration configuration
# ENTRYPOINT ["./tonsail-server"]
