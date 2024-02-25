use crate::{escape_md, EditOrSend, MessageHandlerContext, TgBot};
use teloxide::{types::Message as TgMessage, RequestError};

const DEFAULT_REPLY: &str =
    r"Unexpected input. You can send me a note or ask for help (for example type /help).";

pub async fn handle_help(
    ctx: &MessageHandlerContext,
    bot: &TgBot,
    user_msg: &TgMessage,
    text: impl Into<Option<&str>>,
    bot_msg: impl Into<Option<TgMessage>>,
) -> Result<(), RequestError> {
    let bot_msg: Option<TgMessage> = bot_msg.into();

    let Some(text) = text.into() else {
        bot.edit_or_reply(user_msg, bot_msg, escape_md(DEFAULT_REPLY))
            .await?;
        return Ok(());
    };

    let bot_msg = bot
        .edit_or_reply(user_msg, bot_msg, r"*Generating help message\.\.\.* ")
        .await?;

    let response = match ctx.help_generator.generate_help(text).await {
        Ok(response) => response,
        Err(e) => {
            log::warn!("Failed to generate help: {e}");
            bot.edit(bot_msg, r"Something went wrong, sorry\.\.\.")
                .await?;
            return Ok(());
        }
    };

    bot.edit(bot_msg, escape_md(&response)).await?;

    Ok(())
}
