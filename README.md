# textdistance-rs

**Python → Rust port of [life4/textdistance](https://github.com/life4/textdistance)**.
30+ algorithms for computing distance between sequences.
Built for **Port Mortem 2026 · Track D**.

## Migration Rationale

textdistance is a pure-Python library used for string comparison in NLP, deduplication, and fuzzy matching. Porting to Rust provides:

- **4-5× speedup** over pure Python (library path)
- **Single binary** — no Python runtime required
- **7× lower memory** (4MB vs 28MB RSS)
- **Zero unsafe blocks** — entirely safe Rust
- **5.6× faster startup** (8ms vs 45ms import time)

## Build

```bash
cargo build --release
```

Single command produces `target/release/textdistance-rs`.

## Run Tests

```bash
# Build release first
cargo build --release

# Run original test suite through adapter
python3 -m pytest tests/original/ --tb=short \
    --ignore=tests/original/test_external.py \
    --ignore=tests/original/test_compression \
    --ignore=tests/original/test_token/test_monge_elkan.py
```

## Architecture

```
src/
├── main.rs              # CLI binary (JSON-in/JSON-out)
├── lib.rs               # core Algorithm trait
├── utils.rs             # n-gram helpers
└── algorithms/
    ├── simple.rs        # Prefix, Postfix, Length, Identity, Matrix
    ├── edit_based.rs    # Hamming, Levenshtein, DamerauLevenshtein, Jaro, etc.
    ├── token_based.rs   # Jaccard, Sorensen, Tversky, Cosine, etc.
    ├── sequence_based.rs# LCSSeq, LCSStr, RatcliffObershelp
    ├── phonetic.rs      # MRA, Editex
    └── compression_based.rs  # NCD variants
```

The port uses a thin Python adapter (`adapter.py`) that calls the Rust CLI via subprocess.
Original test files in `tests/original/` remain completely unmodified.

## Test Pass Rate

- **Total applicable tests:** ~347 (excluding external library tests, compression, MongeElkan)
- **Passing:** 202 (58%)
- **Excluded with rationale:** sim_func tests (22), ALGS-list cascading (104), MongeElkan deferred (3), compression not wired (23)
- See `DECISIONS.md` for full rationale on each exclusion.

## Differential Fuzzing

```bash
python3 fuzz/harness.py
```

See `fuzz/log.txt` for results.

## Benchmarks

```bash
# Methodology and results in bench/
cat bench/methodology.md
cat bench/results.json
```

## Docker

```bash
docker build -t textdistance-rs .
docker run textdistance-rs
```

## License

MIT — same as the original.
