FROM rust:alpine AS back-stage

RUN apk update
RUN apk add cmake make musl-dev g++

WORKDIR /usr/src/bzkf-kafkatopic-splitter
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo install --target x86_64-unknown-linux-musl --path .

# Build image from scratch
FROM scratch
COPY --from=back-stage /usr/local/cargo/bin/bzkf-kafkatopic-splitter .
USER 1000
CMD ["./bzkf-kafkatopic-splitter"]
