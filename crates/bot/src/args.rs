use clap::{Args, Parser};
use std::fmt::{Debug, Formatter};

#[derive(Args)]
pub struct Secrets {
    /// Mistral API token
    #[arg(short, long, env, required = true)]
    pub mistral_token: String,

    /// Telegram bot token
    #[arg(short, long, env, required = true)]
    pub telegram_token: String,
}

impl Debug for Secrets {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secrets").finish_non_exhaustive()
    }
}

#[derive(Parser, Debug)]
pub struct BotArgs {
    #[clap(flatten)]
    pub secrets: Secrets,

    #[clap(short, long, env, default_value = "123")]
    pub random_seed: i64,

    #[clap(short, long, env, default_value = "0.5")]
    pub default_temperature: f32,
}
