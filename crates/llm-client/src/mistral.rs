use crate::{ImplMessage, LlmClient};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;

const DEFAULT_MISTRAL_API_URL: &str = "https://api.mistral.ai/v1/chat/completions";

/// Mistral model type.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum MistralModelType {
    #[default]
    #[serde(rename = "mistral-tiny")]
    Tiny,
    #[serde(rename = "mistral-small")]
    Small,
    #[serde(rename = "mistral-medium")]
    Medium,
}

/// Mistral chat participant role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MistralRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistralMessage {
    role: MistralRole,
    content: String,
}

impl MistralMessage {
    pub fn user(content: impl ToString) -> Self {
        Self {
            role: MistralRole::User,
            content: content.to_string(),
        }
    }

    pub fn assistant(content: impl ToString) -> Self {
        Self {
            role: MistralRole::Assistant,
            content: content.to_string(),
        }
    }

    pub fn system(content: impl ToString) -> Self {
        Self {
            role: MistralRole::System,
            content: content.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MistralClient {
    api_key: String,
    history: Vec<MistralMessage>,
    api_url: String,
    client: reqwest::Client,
    model: MistralModelType,
    temperature: f64,
    max_tokens: Option<usize>,
    random_seed: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct Response {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<ResponseChoice>,
    usage: ResponseUsage,
}

#[derive(Debug, Clone, Deserialize)]
enum FinishReason {
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "length")]
    Length,
    #[serde(rename = "model_length")]
    ModelLength,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct ResponseChoice {
    index: i64,
    message: MistralMessage,
    finish_reason: FinishReason,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct ResponseUsage {
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
}

impl MistralClient {
    pub fn new(api_key: impl ToString) -> Self {
        Self {
            api_key: api_key.to_string(),
            history: Vec::new(),
            api_url: DEFAULT_MISTRAL_API_URL.to_string(),
            client: reqwest::Client::new(),
            model: MistralModelType::Tiny,
            temperature: 0.7,
            max_tokens: None,
            random_seed: None,
        }
    }

    /// Set the temperature of the Mistral model. Default is 0.7.
    pub fn with_temperature(mut self, temperature: impl Into<f64>) -> Self {
        self.temperature = temperature.into();
        self
    }

    /// Set the maximum number of tokens to generate. Default is unlimited.
    pub fn with_max_tokens(mut self, max_tokens: impl Into<Option<usize>>) -> Self {
        self.max_tokens = max_tokens.into();
        self
    }

    /// Set the random seed for the Mistral model. Default is None.
    pub fn with_random_seed(mut self, random_seed: impl Into<Option<i64>>) -> Self {
        self.random_seed = random_seed.into();
        self
    }

    /// Adds a message to the end of the history of the client.
    pub fn with_message(mut self, message: MistralMessage) -> Self {
        self.history.push(message);
        self
    }

    /// Adds a system message to the end of the history of the client.
    pub fn with_system_message(self, message: impl ToString) -> Self {
        self.with_message(MistralMessage {
            role: MistralRole::System,
            content: message.to_string(),
        })
    }

    /// Adds a user message to the end of the history of the client.
    pub fn with_user_message(self, message: impl ToString) -> Self {
        self.with_message(MistralMessage {
            role: MistralRole::User,
            content: message.to_string(),
        })
    }

    /// Adds an assistant message to the end of the history of the client.
    pub fn with_assistant_message(self, message: impl ToString) -> Self {
        self.with_message(MistralMessage {
            role: MistralRole::Assistant,
            content: message.to_string(),
        })
    }

    /// Set history of the client.
    pub fn with_history(mut self, history: Vec<MistralMessage>) -> Self {
        self.history = history;
        self
    }

    /// Override the default Mistral API URL.
    pub fn with_api_url(mut self, api_url: impl ToString) -> Self {
        self.api_url = api_url.to_string();
        self
    }

    /// Allows to override the default Mistral model type.
    pub fn with_model(mut self, model: MistralModelType) -> Self {
        self.model = model;
        self
    }

    async fn send_message_inner(&self, message: MistralMessage) -> eyre::Result<Response> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );
        headers.insert("Accept", HeaderValue::from_static("application/json"));

        log::debug!("Sending message to Mistral: {:?}", message);

        // FIXME maybe we should clone the entire history here
        let mut history = self.history.clone();
        history.push(message.clone());

        let body = json!({
            "model": "mistral-tiny",
            "messages": self.history,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
            "random_seed": self.random_seed,
        });

        let response = self
            .client
            .post(&self.api_url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let str_resp = response.text().await?;

        log::debug!("Mistral response: {}", str_resp);

        let response: Response = serde_json::from_str(&str_resp)?;

        Ok(response)
    }
}

#[async_trait::async_trait]
impl LlmClient for MistralClient {
    async fn reset_chat(&mut self) -> eyre::Result<()> {
        self.history.clear();
        Ok(())
    }

    async fn send_message_without_history<T: ImplMessage>(
        &self,
        message: T,
    ) -> eyre::Result<String> {
        let response = self
            .send_message_inner(MistralMessage::user(message))
            .await?;

        let ResponseChoice { message, .. } = response
            .choices
            .get(0)
            .expect("choise should exist")
            .clone();

        let MistralMessage { content, .. } = message.clone();

        Ok(content)
    }

    async fn send_message<T: ImplMessage>(&mut self, user_message: T) -> eyre::Result<String> {
        let user_message = MistralMessage::user(user_message);
        let response = self.send_message_inner(user_message.clone()).await?;

        let ResponseChoice {
            message: response_message,
            ..
        } = response
            .choices
            .get(0)
            .expect("choise should exist")
            .clone();

        let content = response_message.content.clone();

        self.history.push(user_message);
        self.history.push(response_message);

        Ok(content)
    }

    fn last_response(&self) -> Option<String> {
        self.history.last().map(|item| item.content.clone())
    }
}
