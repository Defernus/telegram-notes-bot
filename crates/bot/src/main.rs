use bot::TagsGenerator;
use clap::{Args, Parser};
use dotenvy::dotenv;
use std::fmt::{Debug, Formatter};
use teloxide::{
    adaptors::DefaultParseMode,
    payloads::SendMessageSetters,
    requests::{Requester, RequesterExt},
    types::{MediaKind, Message as TgMessage, MessageCommon, MessageKind, ParseMode},
    Bot,
};

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
    let bot = bot.parse_mode(ParseMode::MarkdownV2);

    teloxide::repl(
        bot,
        move |bot: DefaultParseMode<Bot>, user_msg: TgMessage| {
            let chat_id = user_msg.chat.id;
            let tags_generator =
                TagsGenerator::new(&args.secrets.mistral_token).with_temperature(0.0);

            async move {
                let Some(text) = user_msg.text() else {
                    bot.send_message(chat_id, "I can only process text messages")
                        .await?;
                    return Ok(());
                };

                log::debug!("Received message: {:?}", user_msg);
                let loading_message_id = bot
                    .send_message(chat_id, "**Loading** ")
                    .reply_to_message_id(user_msg.id)
                    .await?
                    .id;

                let tags = match tags_generator.generate_tags(text).await {
                    Ok(tags) => tags,
                    Err(e) => {
                        log::error!("Failed to generate tags: {}", e);
                        bot.edit_message_text(
                            chat_id,
                            loading_message_id,
                            "Failed to generate tags",
                        )
                        .await?;
                        return Ok(());
                    }
                };

                let reply_text = tags.to_escaped_md();
                println!("Replying with: {}", reply_text);

                bot.edit_message_text(chat_id, loading_message_id, reply_text)
                    .await?;

                Ok(())
            }
        },
    )
    .await;
}
