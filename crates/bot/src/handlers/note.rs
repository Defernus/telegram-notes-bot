use crate::{EditOrSend, MessageHandlerContext, TgBot};
use teloxide::{types::Message as TgMessage, RequestError};

pub async fn handle_note(
    ctx: &MessageHandlerContext,
    bot: &TgBot,
    user_msg: &TgMessage,
    text: &str,
    bot_msg: impl Into<Option<TgMessage>>,
) -> Result<(), RequestError> {
    let bot_msg: Option<TgMessage> = bot_msg.into();

    let bot_msg = bot
        .edit_or_reply(user_msg, bot_msg, r"*Generating tags\.\.\.* ")
        .await?;

    log::debug!("processing tags");
    let tags = match ctx.tags_generator.generate_tags_md(text).await {
        Ok(tags) => tags,
        Err(e) => {
            log::warn!("Failed to generate tags: {}", e);
            bot.edit(bot_msg, r"Something went wrong, sorry\.\.\.")
                .await?;
            return Ok(());
        }
    };

    bot.edit(bot_msg, tags).await?;

    Ok(())
}
