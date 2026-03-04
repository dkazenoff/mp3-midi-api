# Use Bullseye to avoid Debian Bookworm apt signature issues on older Docker/libseccomp
FROM rust:1-bullseye as builder

WORKDIR /build

# Apt options for hosts where seccomp blocks GPG (invalid signature); uses official Debian mirrors only
ENV APT_OPTS="-o Acquire::AllowInsecureRepositories=true -o Acquire::AllowDowngradeToInsecureRepositories=true -o APT::Get::AllowUnauthenticated=true"

# Install system dependencies for Rust build only (basic-pitch lives in runtime stage)
RUN apt-get update $APT_OPTS && apt-get install -y --no-install-recommends $APT_OPTS \
    ffmpeg \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*

# Copy Cargo files and build dependencies
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Copy source and build
COPY src ./src
RUN cargo build --release

# Runtime stage (Bullseye for same apt compatibility as builder)
FROM debian:bullseye-slim

ENV APT_OPTS="-o Acquire::AllowInsecureRepositories=true -o Acquire::AllowDowngradeToInsecureRepositories=true -o APT::Get::AllowUnauthenticated=true"

RUN apt-get update $APT_OPTS && apt-get install -y --no-install-recommends $APT_OPTS \
    ffmpeg \
    python3 \
    python3-pip \
    python3-venv \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*

# Install basic-pitch in a venv (Bullseye's pip lacks --break-system-packages)
ENV VENV=/opt/venv
RUN python3 -m venv $VENV && $VENV/bin/pip install --no-cache-dir basic-pitch
ENV PATH="$VENV/bin:$PATH"

WORKDIR /app

COPY --from=builder /build/target/release/mp3-midi-api /app/
COPY static /app/static

RUN mkdir -p /tmp/mp3-midi-uploads

EXPOSE 8080

ENV OPENAI_API_KEY=""

# Wrap so we see in docker logs when the process exits (helps debug restart loops)
CMD ["/bin/sh", "-c", "echo 'Starting mp3-midi-api...'; exec /app/mp3-midi-api"]
