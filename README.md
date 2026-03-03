# mp3-midi-api

Rust-based web backend for MP3 → MIDI → JSON → LLM processing pipeline.

## Features

- Upload MP3 files
- Convert MP3 to MIDI using external tools
- Parse MIDI to JSON format
- Send to LLM provider for analysis
- Minimal web frontend with Baroque test clips
- Containerized with Docker

## Tech Stack

- **Backend**: Rust (Axum web framework)
- **Frontend**: Vanilla HTML/JS
- **Conversion**: FFmpeg + external MIDI tools
- **Deployment**: Docker with restart policy

## Quick Start

```bash
docker build -t mp3-midi-api .
docker run -d --name mp3-midi-api -p 8080:8080 --restart always mp3-midi-api
```

Access at: http://localhost:8080
