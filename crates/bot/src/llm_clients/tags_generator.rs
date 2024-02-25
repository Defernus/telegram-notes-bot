use crate::{base_llm_methods, escape_md, unescape_md};
use llm_client::{ImplMessage, LlmClient, MistralClient, MistralModelType, MistralRole};
use std::{fmt::Display, ops::Deref};

const PROMPT: &str = r####"
You are notes tags generator. Your goal to help with tags generation fot notes.

# Rules

First tag is a category tag. Example of category tags:
- #idea
- #shopping_list
- #recipe
- #must_watch / #must_read / #must_play
- #credentials
- #project

Second tag is a subcategory tag or regular tag. Example of subcategory tags:
- #startup
- #movie / #book / #game
- #grocery / #clothes / #electronics / #furniture ...
- #bank / #email / #social_media / #website

Other tags should be regular tags related to the note content.

Each not should have from 2 to 5 tags.

Tags should contain only lowercase latin letters, numbers and underscores.

Tags should be separated by spaces.

DO NOT GENERATE MORE THAN 5 TAGS!

YOU SHOULD RETURN ONLY A TAG LIST! DO NOT ADD ANYTHING ELSE!

# Examples

## Input
`Liste de courses Ikea:
- Table basse (la petite, pas trop chère)
- Étagère pour le salon (tu sais, celle qu'on a vu la dernière fois)
- Coussins colorés (prends des motifs sympas)
- Lampe de bureau (IMPORTANT, celle avec variateur de lumière si possible)
- Plantes artificielles (2 ou 3 pour égayer la cuisine)
- Cadres photo (tailles variées, choisis jolis)
- Boîtes de rangement (pour mes trucs de couture)
- Rideaux pour la chambre (couleur neutre, style cosy)`

## Response
`#shopping_list #ikea #furniture #home_decor #lighting`

## Input
`Home: 6, Jalan Taman Seputeh, Taman Seputeh, 58000 Kuala Lumpur, Wilayah Persekutuan Kuala Lumpur, Malaysia`

## Response
`#address #home #malasia #kuala_lumpur`

## Input
`Add feature: Dark mode`

## Response
`#idea #project #feature #dark_mode`
"####;

const HISTORY: &[(MistralRole, &str)] = &[
    (MistralRole::User, "Platformer game about a cat"),
    (MistralRole::Assistant, "#idea #game #platformer #cat"),
    (MistralRole::User, "Silicon Valley"),
    (MistralRole::Assistant, "#must_watch #tv_show #comedy #geek"),
];

#[derive(Debug, Clone)]
pub struct TagsGenerator {
    base_client: MistralClient,
    max_tags_amount: usize,
}

impl TagsGenerator {
    pub fn new(token: impl ToString) -> Self {
        let max_tags_amount = 6;
        Self {
            base_client: MistralClient::new(token)
                .with_model(MistralModelType::Tiny)
                .with_max_tokens(calc_mx_tokens(max_tags_amount))
                .with_history(HISTORY)
                .with_system_message(PROMPT),
            max_tags_amount,
        }
    }

    base_llm_methods! {}

    /// Set the maximum amount of tags that can be generated. Default is 6.
    pub fn with_max_tags_amount(mut self, max_tags_amount: usize) -> Self {
        // TODO modify the prompt to include the new max_tags_amount
        self.max_tags_amount = max_tags_amount;
        self.base_client = self
            .base_client
            .with_max_tokens(calc_mx_tokens(max_tags_amount));
        self
    }

    /// Generate tags for a text.
    pub async fn generate_tags(
        &self,
        text: impl ImplMessage,
        // TODO make something with these nested Results
    ) -> eyre::Result<Result<Tags, String>> {
        let text = text.to_string();
        let text = text.trim();
        let response = self.base_client.send_message_without_history(text).await?;

        Ok(Tags::from_str(&response, self.max_tags_amount).ok_or(response))
    }

    /// Generate tags for a text and format them as a markdown string.
    /// If model output is not a valid tag list, return it as a message.
    pub async fn generate_tags_md(
        &self,
        text: impl ImplMessage,
        // TODO make something with these nested Results
    ) -> eyre::Result<String> {
        let tags = self
            .generate_tags(format!("Note:\n{}", text.to_string()))
            .await?;

        match tags {
            Ok(tags) => Ok(tags.to_escaped_md()),
            Err(message) => Ok(escape_md(&message)),
        }
    }
}

fn calc_mx_tokens(tags_amount: usize) -> usize {
    // 10 tokens per tag on average
    tags_amount * 10
}

#[derive(Debug, Clone)]
pub struct Tags {
    tags: Vec<String>,
}

impl Tags {
    /// Create a new `Tags` from a string.
    ///
    /// `max_tags_amount` is the maximum amount of tags that can be generated.
    /// Use `0` for unlimited.
    pub fn from_str(tags: impl ToString, max_tags_amount: usize) -> Option<Self> {
        let tags = tags.to_string();
        let tags_regex = regex::Regex::new(r#"^(\#[a-z_\\]+ )*(\#[a-z_\\]+)"#).unwrap();
        let tags_match = tags_regex.find(tags.as_str())?;

        if tags_match.is_empty() {
            return None;
        }

        let tags = unescape_md(&tags);
        let tags = tags.split("#").filter_map(|tag| {
            let tag = tag.trim();
            let tag = tag.split_once(' ').unwrap_or((tag, "")).0;

            if tag.is_empty() {
                None
            } else {
                Some(tag.to_string())
            }
        });

        let tags = if max_tags_amount == 0 {
            tags.collect::<Vec<_>>()
        } else {
            tags.take(max_tags_amount).collect::<Vec<_>>()
        };

        Some(Self { tags })
    }

    pub fn to_escaped_md(&self) -> String {
        let resp = self
            .iter()
            .map(|tag| format!("#{}", escape_md(tag)))
            .collect::<Vec<_>>()
            .join(" ");

        escape_md(&resp)
    }
}

impl Deref for Tags {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.tags
    }
}

impl From<Vec<String>> for Tags {
    fn from(tags: Vec<String>) -> Self {
        Self { tags }
    }
}

impl Display for Tags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, tag) in self.tags.iter().enumerate() {
            write!(f, "#{}", tag)?;
            if i != self.tags.len() - 1 {
                write!(f, " ")?;
            }
        }
        Ok(())
    }
}
