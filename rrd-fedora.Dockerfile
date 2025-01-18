ARG fedora_version
FROM fedora:${fedora_version}

RUN dnf install -qy --refresh rrdtool-devel curl clang \
    && dnf group install -qy c-development

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal -y
ENV PATH="$PATH:/root/.cargo/bin"
RUN rustup component add clippy

RUN mkdir -p /rrd/ && cd rrd && mkdir src librrd-sys
WORKDIR /rrd

CMD cargo build --all-targets -q && cargo test -q && cargo clippy --all-targets -- -Dwarnings

RUN touch src/lib.rs
COPY librrd-sys librrd-sys

# get dependencies during initial build
COPY Cargo.toml .
RUN cargo fetch -q
RUN cargo build -q
# then copy the actual source, keeping dependencies
COPY . .
