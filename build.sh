#!/bin/bash
set -e

echo "🔨 Building mp3-midi-api Docker image..."

docker build -t mp3-midi-api .

echo "✅ Build complete!"
echo ""
echo "To run:"
echo "  docker run -d --name mp3-midi-api -p 8080:8080 --restart always mp3-midi-api"
echo ""
echo "Or use docker-compose:"
echo "  docker-compose up -d"
