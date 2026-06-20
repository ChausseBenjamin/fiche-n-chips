# ----------------------------------------
# Stage 1: Build the musl Rust binary
# ----------------------------------------
FROM rust:slim-bookworm AS build

# Install build dependencies:
#   gcc       - C compiler needed by simsimd crate's build script (for musl target)
#   clang     - Rust linker (per .cargo/config.toml)
#   mold      - Fast linker (per .cargo/config.toml)
#   upx       - Binary compression
RUN echo "deb http://deb.debian.org/debian bookworm-backports main" >> /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y --no-install-recommends -t bookworm-backports \
      upx-ucl \
      gcc \
      clang \
      mold && \
    rm -rf /var/lib/apt/lists/*

# Add the musl cross-compilation target (for the nightly toolchain from rust-toolchain.toml)
RUN rustup target add x86_64-unknown-linux-musl --toolchain nightly-2026-05-17

WORKDIR /build

# ── Layer 1: Build-time dependencies ──
# Everything that rarely changes: manifests, build script, assets.
COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY app/Cargo.toml app/build.rs ./app/
COPY app/assets ./app/assets/
COPY rslog/Cargo.toml ./rslog/
COPY .cargo/ ./.cargo/

# ── Layer 2: Dummy sources to pre-build all dependencies ──
# Use an empty build.rs so cargo registers the build script and creates an
# OUT_DIR.  Without this the real build.rs will fail with
# "OUT_DIR not defined at compile time" because the build-script runner
# never ran during the dummy- source phase.
RUN mkdir -p app/src rslog/src && \
    echo "fn main() {}" > app/src/main.rs && \
    echo "" > rslog/src/lib.rs && \
    echo "fn main() {}" > app/build.rs && \
    RUSTC_WRAPPER="" cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null; true

# ── Layer 3: Real source code ──
# Docker COPY preserves host mtimes which may be older than the stored cargo
# fingerprint, causing cargo to skip recompilation.  Remove the dummy sources
# first, then COPY, then touch everything so cargo sees the changes.
RUN rm -rf app/src rslog/src
COPY app/src ./app/src
COPY rslog/src ./rslog/src
# Restore the real build.rs (overwrites the empty one from the dummy step)
COPY app/build.rs ./app/build.rs
RUN find app/src rslog/src app/build.rs -type f -exec touch {} +

# Build a fully static musl release binary
RUN RUSTC_WRAPPER="" cargo build --release --target x86_64-unknown-linux-musl && \
    cp target/x86_64-unknown-linux-musl/release/fiche-n-chips /app

# Strip unnecessary exception-handling sections first.
# With panic=abort the eh_frame tables are dead weight (~600KB).
# This must run before UPX because UPX-compressed binaries have no sections.
RUN objcopy --remove-section=.eh_frame --remove-section=.eh_frame_hdr /app /app.stripped && \
    mv /app.stripped /app

# Compress the binary with UPX level 7
RUN upx -7 /app

# Sanity check: verify the binary produces help output
RUN /app --help 2>&1 | grep -q "Usage:" || (echo "ERROR: built binary is broken! Help output missing."; exit 1)

# ----------------------------------------
# Stage 2: Minimal scratch image
# ----------------------------------------
FROM scratch AS package

COPY --from=build /app /app

ENTRYPOINT ["/app"]
