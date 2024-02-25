use crate::{base_llm_methods, parse_prompt};
use core::fmt;
use enum_iterator::{all, Sequence};
use llm_client::{ImplMessage, LlmClient, MistralClient, MistralModelType, MistralRole};
use std::fmt::{Display, Formatter};

const MAX_TOKENS: usize = 10;

const PROMPT: &str = r####"
You are the task selector manager bot. Your goal to select exact task that user want to do.

# Tags
{{tags}}
"####;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum TaskType {
    Note,
    Help,
    Unknown,
}

impl Display for TaskType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Note => write!(f, "[note]"),
            Self::Help => write!(f, "[help]"),
            Self::Unknown => write!(f, "[unknown]"),
        }
    }
}

impl TaskType {
    pub fn description(self) -> String {
        let msg = self.to_string();
        match self {
            Self::Note => format!("`{msg}`: User typed anything that look like a note: shopping list, idea, some movie to watch etc. Probably any sentence that doesn't fit other categories and make at least remote sense."),
            Self::Help => format!("`{msg}`: User directly asked how to use bot, any specific or general question about available commands, bot's features etc."),
            Self::Unknown => format!("`{msg}`: User typed something that bot can't understand: gibberish, random letters, etc."),
        }
    }
}

const HISTORY: &[(MistralRole, &str)] = &[
    (MistralRole::User, "Watch titanic"),
    (MistralRole::Assistant, "[note]"),
    (MistralRole::User, "start"),
    (MistralRole::Assistant, "[help]"),
    (MistralRole::User, "Platformer game about a cat"),
    (MistralRole::Assistant, "[note]"),
    (MistralRole::User, "zerxtcvbhjkm"),
    (MistralRole::Assistant, "[unknown]"),
    (MistralRole::User, "How to use this bot?"),
    (MistralRole::Assistant, "[help]"),
];

#[derive(Debug, Clone)]
pub struct TaskSelector {
    base_client: MistralClient,
}

impl TaskSelector {
    pub fn new(token: impl ToString) -> Self {
        let tags_types = all::<TaskType>()
            .map(|t| format!("- {}", t.description()))
            .collect::<Vec<_>>()
            .join("\n");

        Self {
            base_client: MistralClient::new(token)
                .with_model(MistralModelType::Tiny)
                .with_max_tokens(MAX_TOKENS)
                .with_history(HISTORY)
                .with_system_message(parse_prompt!(PROMPT, tags = tags_types)),
        }
    }

    base_llm_methods! {}

    pub async fn select_task(&self, text: impl ImplMessage) -> eyre::Result<TaskType> {
        let text = text.to_string();
        let text = text.trim();

        let response = self.base_client.send_message_without_history(text).await?;

        log::debug!("select_task response: {}", response);

        let task_type = all::<TaskType>()
            .find(|t| response.contains(&t.to_string()))
            .unwrap_or(TaskType::Unknown);

        Ok(task_type)
    }
}
