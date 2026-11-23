FROM rust:1.100.0-trixie AS build-env
LABEL maintainer="yanorei32"

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

WORKDIR /usr/src
RUN cargo new konomitv-proxy
COPY LICENSE Cargo.toml Cargo.lock /usr/src/konomitv-proxy/
WORKDIR /usr/src/konomitv-proxy
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN	cargo install cargo-license && cargo license \
	--authors \
	--do-not-bundle \
	--avoid-dev-deps \
	--avoid-build-deps \
	--filter-platform "$(rustc -vV | sed -n 's|host: ||p')" \
	> CREDITS

RUN cargo build --release
COPY src/ /usr/src/konomitv-proxy/src/
COPY assets/ /usr/src/konomitv-proxy/assets/
RUN touch  assets/* src/* && cargo build --release

FROM debian:trixie-slim

RUN apt-get update -y && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /

COPY --chown=root:root --from=build-env \
	/usr/src/konomitv-proxy/CREDITS \
	/usr/src/konomitv-proxy/LICENSE \
	/usr/share/licenses/konomitv-proxy/

COPY --chown=root:root --from=build-env \
	/usr/src/konomitv-proxy/target/release/konomitv-proxy \
	/usr/bin/konomitv-proxy

CMD ["/usr/bin/konomitv-proxy"]
