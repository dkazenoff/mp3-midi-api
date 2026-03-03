# mp3-midi-api

Rust-based web backend for MP3 → MIDI → JSON → LLM processing pipeline.

## Features

- Upload MP3 files
- Convert MP3 to MIDI using external tools (FFmpeg + basic-pitch)
- Parse MIDI to JSON format
- **LLM Integration**: Analyze MIDI with GPT-4 for musical insights
- **AI MIDI Generation**: Create MIDI from text prompts
- Minimal web frontend with drag-drop upload
- Containerized with Docker

## Tech Stack

- **Backend**: Rust (Axum web framework)
- **Frontend**: Vanilla HTML/JS
- **Conversion**: FFmpeg + external MIDI tools
- **Deployment**: Docker with restart policy

## Quick Start

```bash
# Clone repo
git clone https://github.com/dkazenoff/mp3-midi-api.git
cd mp3-midi-api

# Set up API key (optional but recommended)
cp .env.example .env
# Edit .env and add your OPENAI_API_KEY

# Build and run
docker-compose up -d
```

Access at: http://localhost:8080

### Manual Docker

```bash
docker build -t mp3-midi-api .
docker run -d --name mp3-midi-api -p 8080:8080 \
  -e OPENAI_API_KEY=sk-your-key \
  --restart always mp3-midi-api
```

## API Endpoints

### POST /api/upload
Upload MP3 file, converts to MIDI JSON, optionally analyzes with LLM

**Response:**
```json
{
  "success": true,
  "json_data": { "tempo": 120, "notes": [...] },
  "llm_analysis": "This piece shows a classical structure..."
}
```

### GET /api/generate?prompt=<text>&api_key=<optional>
Generate MIDI from natural language prompt

**Example:**
```
/api/generate?prompt=Happy+birthday+melody+in+C+major
```
