use std::{fmt::Display, ops::Deref};

use llm_client::{ImplMessage, LlmClient, MistralClient, MistralMessage, MistralModelType};

pub const TAG_GENERATOR_PROMPT: &str = r####"
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

"####;

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
                .with_history(vec![
                    MistralMessage::system(TAG_GENERATOR_PROMPT),
                    MistralMessage::user("Platformer game about a cat"),
                    MistralMessage::assistant("#idea #game #platformer #cat"),
                    MistralMessage::user("Silicon Valley"),
                    MistralMessage::assistant("#must_watch #tv_show #comedy #geek"),
                ]),
            max_tags_amount,
        }
    }

    /// Set the temperature of the Mistral model. Default is 0.7.
    pub fn with_temperature(mut self, temperature: impl Into<f64>) -> Self {
        self.base_client = self.base_client.with_temperature(temperature);
        self
    }

    /// Set the random seed for the Mistral model. Default is None.
    pub fn with_random_seed(mut self, random_seed: impl Into<Option<i64>>) -> Self {
        self.base_client = self.base_client.with_random_seed(random_seed);
        self
    }

    /// Set the maximum amount of tags that can be generated. Default is 6.
    pub fn with_max_tags_amount(mut self, max_tags_amount: usize) -> Self {
        // TODO modify the prompt to include the new max_tags_amount
        self.max_tags_amount = max_tags_amount;
        self.base_client = self
            .base_client
            .with_max_tokens(calc_mx_tokens(max_tags_amount));
        self
    }

    /// Set the Mistral model type.
    pub fn with_model(mut self, model: MistralModelType) -> Self {
        self.base_client = self.base_client.with_model(model);
        self
    }

    /// Generate tags for a text.
    pub async fn generate_tags(&self, text: impl ImplMessage) -> eyre::Result<Tags> {
        let text = text.to_string();
        let text = text.trim();
        let response = self.base_client.send_message_without_history(text).await?;

        let response = response.trim();

        // Safety: Split always returns at least one element
        let response = unsafe { response.split("\n").next().unwrap_unchecked() };

        let response = response.trim().to_string();

        let response = response.replace(r"\_", "_").replace(r"\#", "#");

        // TODO check response format using `(#[_a-z0-9]+ )*#[_a-z0-9]+` or something similar

        let response: Vec<_> = response
            .split("#")
            .filter_map(|tag| {
                let tag = tag.trim().to_lowercase();
                if tag.is_empty() {
                    None
                } else {
                    Some(tag)
                }
            })
            .take(self.max_tags_amount)
            .collect();

        if response.is_empty() {
            eyre::bail!("No tags were generated")
        }

        Ok(response.into())
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
    pub fn to_escaped_md(&self) -> String {
        self.to_string().replace("#", r"\#").replace("_", r"\_")
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
