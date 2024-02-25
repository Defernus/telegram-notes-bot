use bot::{BotArgs, MessageHandlerContext, TgBot};
use clap::Parser;
use dotenvy::dotenv;
use std::sync::Arc;
use teloxide::{
    requests::RequesterExt,
    types::{Message as TgMessage, ParseMode},
    Bot,
};

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

    let ctx = Arc::new(MessageHandlerContext::new(&args));

    teloxide::repl(bot, move |bot: TgBot, user_msg: TgMessage| {
        let ctx = ctx.clone();

        async move {
            ctx.handle_message(&bot, user_msg).await?;

            Ok(())
        }
    })
    .await;
}
