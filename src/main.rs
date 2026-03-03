mod llm;

use axum::{
    extract::{Multipart, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tower_http::{cors::CorsLayer, services::ServeDir};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct MidiNote {
    note: u8,
    velocity: u8,
    start_time: u32,
    duration: u32,
}

#[derive(Serialize, Deserialize)]
struct MidiData {
    tempo: u32,
    time_signature: String,
    notes: Vec<MidiNote>,
}

#[derive(Serialize)]
struct ConversionResponse {
    success: bool,
    midi_file: Option<String>,
    json_data: Option<MidiData>,
    llm_analysis: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct GenerateQuery {
    prompt: String,
    api_key: Option<String>,
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn upload_handler(mut multipart: Multipart) -> impl IntoResponse {
    let upload_dir = PathBuf::from("/tmp/mp3-midi-uploads");
    std::fs::create_dir_all(&upload_dir).ok();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("upload.mp3").to_string();
            let data = field.bytes().await.unwrap_or_default();

            let file_id = Uuid::new_v4();
            let mp3_path = upload_dir.join(format!("{}.mp3", file_id));
            let midi_path = upload_dir.join(format!("{}.mid", file_id));

            // Save MP3
            if let Err(_) = std::fs::write(&mp3_path, data) {
                return Json(ConversionResponse {
                    success: false,
                    midi_file: None,
                    json_data: None,
                    error: Some("Failed to save MP3".to_string()),
                })
                .into_response();
            }

            // Convert MP3 to WAV first (MIDI conversion tools work better with WAV)
            let wav_path = upload_dir.join(format!("{}.wav", file_id));
            let output = std::process::Command::new("ffmpeg")
                .args(&[
                    "-i",
                    mp3_path.to_str().unwrap(),
                    "-ar",
                    "44100",
                    "-ac",
                    "1",
                    wav_path.to_str().unwrap(),
                ])
                .output();

            if output.is_err() || !output.unwrap().status.success() {
                return Json(ConversionResponse {
                    success: false,
                    midi_file: None,
                    json_data: None,
                    error: Some("FFmpeg conversion failed".to_string()),
                })
                .into_response();
            }

            // Use basic-pitch (free Python tool) for audio to MIDI
            // Note: This requires basic-pitch to be installed in the container
            let output = std::process::Command::new("basic-pitch")
                .args(&[
                    upload_dir.to_str().unwrap(),
                    wav_path.to_str().unwrap(),
                ])
                .output();

            // basic-pitch creates .mid file with same base name
            let expected_midi = upload_dir.join(format!("{}.mid", file_id));
            
            if !expected_midi.exists() {
                // Fallback: create a simple test MIDI file
                return Json(ConversionResponse {
                    success: false,
                    midi_file: None,
                    json_data: None,
                    error: Some("MIDI conversion not available - basic-pitch not found".to_string()),
                })
                .into_response();
            }

            // Parse MIDI to JSON
            match parse_midi_to_json(&expected_midi) {
                Ok(midi_data) => {
                    let midi_json = serde_json::to_string_pretty(&midi_data).unwrap_or_default();
                    
                    // Try LLM analysis if API key is available
                    let llm_analysis = if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
                        llm::enhance_midi_with_llm(&midi_json, &api_key, "openai")
                            .await
                            .ok()
                    } else {
                        None
                    };

                    return Json(ConversionResponse {
                        success: true,
                        midi_file: Some(file_id.to_string()),
                        json_data: Some(midi_data),
                        llm_analysis,
                        error: None,
                    })
                    .into_response();
                }
                Err(e) => {
                    return Json(ConversionResponse {
                        success: false,
                        midi_file: None,
                        json_data: None,
                        llm_analysis: None,
                        error: Some(format!("MIDI parsing failed: {}", e)),
                    })
                    .into_response();
                }
            }
        }
    }

    Json(ConversionResponse {
        success: false,
        midi_file: None,
        json_data: None,
        llm_analysis: None,
        error: Some("No file uploaded".to_string()),
    })
    .into_response()
}

async fn generate_handler(Query(params): Query<GenerateQuery>) -> impl IntoResponse {
    let api_key = params.api_key
        .or_else(|| std::env::var("OPENAI_API_KEY").ok())
        .unwrap_or_default();

    if api_key.is_empty() {
        return Json(ConversionResponse {
            success: false,
            midi_file: None,
            json_data: None,
            llm_analysis: None,
            error: Some("No API key provided".to_string()),
        })
        .into_response();
    }

    match llm::generate_midi_from_prompt(&params.prompt, &api_key).await {
        Ok(llm_response) => {
            // Try to parse the LLM response as MIDI data
            match serde_json::from_str::<MidiData>(&llm_response) {
                Ok(midi_data) => {
                    Json(ConversionResponse {
                        success: true,
                        midi_file: None,
                        json_data: Some(midi_data),
                        llm_analysis: Some(llm_response),
                        error: None,
                    })
                    .into_response()
                }
                Err(_) => {
                    // LLM didn't return valid JSON, return as analysis
                    Json(ConversionResponse {
                        success: false,
                        midi_file: None,
                        json_data: None,
                        llm_analysis: Some(llm_response),
                        error: Some("LLM response was not valid MIDI JSON".to_string()),
                    })
                    .into_response()
                }
            }
        }
        Err(e) => {
            Json(ConversionResponse {
                success: false,
                midi_file: None,
                json_data: None,
                llm_analysis: None,
                error: Some(format!("LLM generation failed: {}", e)),
            })
            .into_response()
        }
    }
}

fn parse_midi_to_json(midi_path: &PathBuf) -> Result<MidiData, String> {
    let data = std::fs::read(midi_path).map_err(|e| e.to_string())?;
    let smf = midly::Smf::parse(&data).map_err(|e| e.to_string())?;

    let mut notes = Vec::new();
    let mut tempo = 500000; // Default: 120 BPM

    for track in smf.tracks.iter() {
        let mut current_time = 0u32;
        for event in track {
            current_time += event.delta.as_int();
            
            match event.kind {
                midly::TrackEventKind::Midi { channel: _, message } => {
                    match message {
                        midly::MidiMessage::NoteOn { key, vel } => {
                            if vel.as_int() > 0 {
                                notes.push(MidiNote {
                                    note: key.as_int(),
                                    velocity: vel.as_int(),
                                    start_time: current_time,
                                    duration: 100, // Placeholder
                                });
                            }
                        }
                        _ => {}
                    }
                }
                midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(t)) => {
                    tempo = t.as_int();
                }
                _ => {}
            }
        }
    }

    Ok(MidiData {
        tempo,
        time_signature: "4/4".to_string(),
        notes,
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/upload", post(upload_handler))
        .route("/api/generate", get(generate_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    println!("🎵 mp3-midi-api running on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}
