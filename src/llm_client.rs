use crate::pipeline::LlmConfig;
use crate::types::GateError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LlmClient {
    api_key: String,
    config: LlmConfig,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    max_tokens: usize,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: String,
}

impl LlmClient {
    pub fn load(config: &LlmConfig) -> Result<Self, GateError> {
        let api_key = if config.api_key.is_empty() {
            std::env::var("DEEPSEEK_API_KEY").unwrap_or_default()
        } else {
            config.api_key.clone()
        };

        if api_key.is_empty() {
            tracing::warn!("DEEPSEEK_API_KEY not set, LLM stage will be skipped");
            return Err("no API key configured".into());
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("failed to create HTTP client: {}", e))?;

        tracing::info!(
            model = %config.model_name,
            url = %config.api_url,
            "LLM client configured"
        );

        Ok(Self {
            api_key,
            config: config.clone(),
            client,
        })
    }

    pub fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    pub async fn infer(&self, prompt: &str) -> Result<String, GateError> {
        let request = ChatRequest {
            model: self.config.model_name.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: "You are a security gatekeeper. Respond in the exact format requested. Be concise.".into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: prompt.to_string(),
                },
            ],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };

        let response = self
            .client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("failed to read response: {}", e))?;

        if !status.is_success() {
            tracing::error!(status = %status, body = %body, "LLM API error");
            return Err(format!("API error {}: {}", status.as_u16(), body).into());
        }

        let chat_response: ChatResponse =
            serde_json::from_str(&body).map_err(|e| format!("failed to parse API response: {}", e))?;

        let content = chat_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(content)
    }
}
