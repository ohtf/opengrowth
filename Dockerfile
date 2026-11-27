# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# The opengrowth server as a container image: `opengrowth serve` on port 8000,
# as a non-root user, with the reference tables compiled in.
#
#   docker build --build-arg OPENGROWTH_COMMIT="$(git rev-parse HEAD)" -t opengrowth .
#   docker run --rm -p 8000:8000 -e AUTHORIZATION_KEY="..." opengrowth
#
# OPENGROWTH_COMMIT records the full commit in every result's provenance;
# without it, results say "unknown". The builder's Rust version should match
# rust-toolchain.toml, which the build follows either way. Both base images are
# pinned by digest, and Dependabot keeps them current.

FROM rust:1.100.0-slim-trixie@sha256:3b199fd6db7d709274df56565b8f4003f9e30d7bfcff9d53148c86053b7855e2 AS build
WORKDIR /src

# The toolchain the repository pins, in a layer of its own.
COPY rust-toolchain.toml ./
RUN rustup toolchain install

COPY . .
ARG OPENGROWTH_COMMIT
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    if [ -z "${OPENGROWTH_COMMIT:-}" ]; then unset OPENGROWTH_COMMIT; fi \
    && cargo build --release --locked --bin opengrowth \
    && cp target/release/opengrowth /usr/local/bin/opengrowth

# glibc, matching the builder's Debian 13, and nothing else: no shell.
FROM gcr.io/distroless/cc-debian13:nonroot@sha256:29ee70762214fe8190cea04ef1c0a84a8365879e047ce8f911ae5a5ab2355fa2
ARG OPENGROWTH_COMMIT
LABEL org.opencontainers.image.title="opengrowth" \
      org.opencontainers.image.description="Growth calculations for children over HTTP. Pre-alpha: not for clinical use." \
      org.opencontainers.image.source="https://github.com/ohtf/opengrowth" \
      org.opencontainers.image.documentation="https://ohtf.github.io/opengrowth/" \
      org.opencontainers.image.licenses="AGPL-3.0-or-later" \
      org.opencontainers.image.revision="${OPENGROWTH_COMMIT}"
COPY --from=build /usr/local/bin/opengrowth /usr/local/bin/opengrowth
COPY LICENSE NOTICE.md /usr/share/doc/opengrowth/
COPY LICENSES /usr/share/doc/opengrowth/LICENSES
ENV OPENGROWTH_BIND=0.0.0.0:8000
EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/opengrowth"]
CMD ["serve"]
