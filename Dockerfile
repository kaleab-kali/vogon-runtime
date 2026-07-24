# syntax=docker/dockerfile:1

FROM rust:1.97.1-bookworm AS build

WORKDIR /workspace

ENV CARGO_INCREMENTAL=0
ENV CARGO_NET_RETRY=10

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates

RUN cargo build --release --locked -p vogon-cli

FROM debian:bookworm-slim AS runtime

ARG VOGON_IMAGE_VERSION=dev
ARG VOGON_IMAGE_REVISION=unknown

LABEL org.opencontainers.image.title="Vogon Runtime" \
    org.opencontainers.image.description="Deterministic, replayable AI workflow runtime CLI." \
    org.opencontainers.image.source="https://github.com/kaleab-kali/vogon-runtime" \
    org.opencontainers.image.documentation="https://github.com/kaleab-kali/vogon-runtime#readme" \
    org.opencontainers.image.licenses="MIT" \
    org.opencontainers.image.version="${VOGON_IMAGE_VERSION}" \
    org.opencontainers.image.revision="${VOGON_IMAGE_REVISION}"

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 vogon

COPY --from=build /workspace/target/release/vogon /usr/local/bin/vogon

USER vogon
WORKDIR /work
ENTRYPOINT ["vogon"]
