FROM rust:1.82-bookworm@sha256:d9c3c6f1264a547d84560e06ffd79ed7a799ce0bff0980b26cf10d29af888377 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p aidoc-cli --locked

FROM rust:1.82-bookworm@sha256:d9c3c6f1264a547d84560e06ffd79ed7a799ce0bff0980b26cf10d29af888377

RUN useradd --uid 10001 --create-home --shell /usr/sbin/nologin aidoc

WORKDIR /home/aidoc

COPY --from=builder /app/target/release/aidoc /usr/local/bin/aidoc

USER aidoc

ENV AIDOC_TIMEOUT_MS=5000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD aidoc explain >/dev/null 2>&1 || exit 1

ENTRYPOINT ["aidoc"]
CMD ["explain"]
