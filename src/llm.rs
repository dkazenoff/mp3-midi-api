use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Serialize)]
struct LlmRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct LlmResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

pub async fn enhance_midi_with_llm(
    midi_json: &str,
    api_key: &str,
    provider: &str,
) -> Result<String, String> {
    let client = Client::new();
    
    let prompt = format!(
        r#"You are a music composition expert. Analyze the following MIDI data and suggest improvements or generate a complementary musical phrase.

MIDI Data:
{}

Please provide:
1. A brief analysis of the musical structure
2. Suggested modifications to make it more interesting
3. Additional notes or harmonies that would complement this piece

Format your response as actionable MIDI modifications (note additions, velocity changes, timing adjustments)."#,
        midi_json
    );

    let (url, model) = match provider {
        "openai" => ("https://api.openai.com/v1/chat/completions", "gpt-4"),
        "anthropic" => ("https://api.anthropic.com/v1/messages", "claude-3-sonnet-20240229"),
        _ => return Err("Unsupported provider".to_string()),
    };

    let request = LlmRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
        temperature: 0.7,
    };

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("LLM API error {}: {}", status, text));
    }

    let llm_response: LlmResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    llm_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from LLM".to_string())
}

pub async fn generate_midi_from_prompt(
    prompt: &str,
    api_key: &str,
) -> Result<String, String> {
    let client = Client::new();
    
    let system_prompt = r#"You are a MIDI composition AI. Generate MIDI note sequences in JSON format.
Output format:
{
  "tempo": 120,
  "time_signature": "4/4",
  "notes": [
    {"note": 60, "velocity": 80, "start_time": 0, "duration": 480},
    {"note": 64, "velocity": 75, "start_time": 480, "duration": 480}
  ]
}

Rules:
- note: MIDI note number (0-127, middle C = 60)
- velocity: how hard the note is played (0-127)
- start_time: ticks from start (480 ticks = quarter note at 480 PPQ)
- duration: length in ticks
"#;

    let request = LlmRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: format!("Generate a MIDI composition for: {}", prompt),
            },
        ],
        temperature: 0.9,
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("LLM API error: {}", response.status()));
    }

    let llm_response: LlmResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    llm_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from LLM".to_string())
}
