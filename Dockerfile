# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:1.50 as cargo-build

RUN apt-get update
RUN apt-get install libclang-dev clang cmake libxxhash-dev -y

WORKDIR /usr/src/sfss

COPY Cargo.toml Cargo.toml

RUN mkdir src/

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

RUN cargo build --release

RUN rm -f target/release/deps/sfss*

COPY . .

RUN cargo build --release

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM debian:buster-slim

RUN useradd --shell /bin/sh --uid 1024 sfss

RUN apt-get update
RUN apt-get install libclang-dev libxxhash-dev -y

WORKDIR /home/sfss/bin/
RUN mkdir -p /var/sfss && chown sfss:sfss /var/sfss
RUN mkdir -p /tmp/sfss && chown sfss:sfss /tmp/sfss
COPY --from=cargo-build /usr/src/sfss/target/release/sfss .

RUN chown sfss:sfss sfss

EXPOSE 8000
USER sfss

CMD ["./sfss"]
