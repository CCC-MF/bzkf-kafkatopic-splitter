FROM rust:alpine AS back-stage

RUN apk update
RUN apk add cmake make musl-dev g++

WORKDIR /usr/src/bzkf-kafkatopic-splitter
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo install --target x86_64-unknown-linux-musl --path .

# Build image from scratch
FROM scratch
LABEL org.opencontainers.image.source = "https://github.com/CCC-MF/bzkf-kafkatopic-splitter"
LABEL org.opencontainers.image.licenses=MIT
LABEL org.opencontainers.image.description="Anwendung zum Neugruppieren von Nachrichten basierend auf Angaben zum Jahr einer Meldung"

COPY --from=back-stage /usr/local/cargo/bin/bzkf-kafkatopic-splitter .
USER 65532:65532
CMD ["./bzkf-kafkatopic-splitter"]
