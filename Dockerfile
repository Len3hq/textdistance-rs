FROM rust:1.97-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y python3 python3-pip && rm -rf /var/lib/apt/lists/*
RUN pip3 install --break-system-packages pytest hypothesis

WORKDIR /app
COPY --from=builder /app/target/release/textdistance-rs /app/textdistance-rs
COPY adapter.py conftest.py ./
COPY tests/original/ tests/original/
COPY fuzz/ fuzz/
COPY bench/ bench/
COPY DECISIONS.md README.md .port-mortem.toml ./

ENV PATH="/app:${PATH}"

CMD ["python3", "-m", "pytest", "tests/original/", "--tb=short", "--ignore=tests/original/test_external.py"]
