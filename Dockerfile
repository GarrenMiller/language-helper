FROM rust:1.97-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN useradd -r -s /bin/false appuser
USER appuser
COPY --from=builder /app/target/release/language-helper /app/language-helper
COPY --from=builder /app/src/morphology/dog.hfstol /app/src/morphology/dog.hfstol
CMD ["./language-helper"]

