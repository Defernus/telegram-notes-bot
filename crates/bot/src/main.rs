use bot::TagsGenerator;
use clap::{Args, Parser};
use dotenvy::dotenv;
use std::fmt::{Debug, Formatter};
use teloxide::{requests::Requester, types::Message as TgMessage, Bot};

#[derive(Args)]
struct Secrets {
    /// Mistral API token
    #[arg(short, long, env, required = true)]
    mistral_token: String,

    /// Telegram bot token
    #[arg(short, long, env, required = true)]
    telegram_token: String,
}

impl Debug for Secrets {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secrets").finish_non_exhaustive()
    }
}

#[derive(Parser, Debug)]
struct BotArgs {
    #[clap(flatten)]
    secrets: Secrets,
}

#[tokio::main]
async fn main() {
    if dotenv().is_ok() {
        log::debug!("Loaded .env file");
    }
    pretty_env_logger::init();

    let args = BotArgs::parse();

    log::info!("Starting bot with args: {:?}", args);

    let bot = Bot::new(&args.secrets.telegram_token);

    teloxide::repl(bot, move |bot: Bot, msg: TgMessage| {
        let mut tags_generator =
            TagsGenerator::new(&args.secrets.mistral_token).with_temperature(0.0);

        async move {
            let Some(text) = msg.text() else {
                bot.send_message(msg.chat.id, "Please send a text message")
                    .await?;
                return Ok(());
            };

            let tags = match tags_generator.generate_tags(text).await {
                Ok(tags) => tags,
                Err(err) => {
                    let err = err.wrap_err(format!("Failed to generate tags for text: {}", text));
                    log::error!("Error: {}", err);

                    bot.send_message(msg.chat.id, format!("Error: {}", err))
                        .await?;
                    return Ok(());
                }
            };

            bot.send_message(msg.chat.id, format!("{tags}\n\n{text}"))
                .await?;

            bot.delete_message(msg.chat.id, msg.id).await?;

            Ok(())
        }
    })
    .await;
}
