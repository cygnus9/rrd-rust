ARG ubuntu_version
FROM ubuntu:${ubuntu_version}

RUN apt-get update
RUN DEBIAN_FRONTEND=noninteractive TZ=Etc/UTC \
    apt-get install -qy librrd-dev curl build-essential pkg-config libclang-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal -y
ENV PATH="$PATH:/root/.cargo/bin"
RUN rustup component add clippy rustfmt

RUN mkdir -p /rrd/ && cd rrd && mkdir src librrd-sys
WORKDIR /rrd

CMD ./scripts/quick-check.sh

RUN touch src/lib.rs
COPY librrd-sys librrd-sys

# get dependencies during initial build
COPY Cargo.toml .
RUN cargo fetch -q
RUN cargo build -q
# then copy the actual source, keeping dependencies
COPY . .
