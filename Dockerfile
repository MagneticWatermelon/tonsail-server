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
