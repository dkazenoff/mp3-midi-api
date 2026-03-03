FROM rust:1.75 as builder

WORKDIR /build

# Install system dependencies for audio processing
RUN apt-get update && apt-get install -y \
    ffmpeg \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Install basic-pitch for audio to MIDI conversion
RUN pip3 install --break-system-packages basic-pitch

# Copy Cargo files and build dependencies
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Copy source and build
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ffmpeg \
    python3 \
    python3-pip \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN pip3 install --break-system-packages basic-pitch

WORKDIR /app

COPY --from=builder /build/target/release/mp3-midi-api /app/
COPY static /app/static

RUN mkdir -p /tmp/mp3-midi-uploads

EXPOSE 8080

ENV OPENAI_API_KEY=""

CMD ["/app/mp3-midi-api"]
