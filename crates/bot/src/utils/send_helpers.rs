use crate::{unescape_md, TgBot};
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, Message as TgMessage},
    Bot,
};

#[async_trait::async_trait]
pub trait EditOrSend<E> {
    async fn edit_or_reply(
        &self,
        user_msg: &TgMessage,
        bot_msg: impl Into<Option<TgMessage>> + Send + Sync,
        text: impl ToString + Send + Sync,
    ) -> Result<TgMessage, E>;

    async fn send(&self, chat_id: ChatId, msg: impl ToString + Send + Sync)
        -> Result<TgMessage, E>;

    async fn reply(
        &self,
        reply_to: &TgMessage,
        msg: impl ToString + Send + Sync,
    ) -> Result<TgMessage, E>;

    async fn edit_or_send(
        &self,
        chat_id: ChatId,
        bot_msg: impl Into<Option<TgMessage>> + Send + Sync,
        text: impl ToString + Send + Sync,
    ) -> Result<TgMessage, E>;

    async fn edit(&self, msg: TgMessage, text: impl ToString + Send + Sync)
        -> Result<TgMessage, E>;
}

#[async_trait::async_trait]
impl EditOrSend<<Bot as Requester>::Err> for TgBot {
    async fn edit_or_reply(
        &self,
        user_msg: &TgMessage,
        bot_msg: impl Into<Option<TgMessage>> + Send + Sync,
        text: impl ToString + Send + Sync,
    ) -> Result<TgMessage, <Bot as Requester>::Err> {
        let bot_msg: Option<TgMessage> = bot_msg.into();
        let text = text.to_string();

        let bot_msg = match bot_msg {
            Some(bot_msg) => self.edit(bot_msg, &text).await?,
            None => self.reply(user_msg, text).await?,
        };

        Ok(bot_msg)
    }

    async fn reply(
        &self,
        reply_to: &TgMessage,
        text: impl ToString + Send + Sync,
    ) -> Result<TgMessage, <Bot as Requester>::Err> {
        self.send_message(reply_to.chat.id, text.to_string())
            .reply_to_message_id(reply_to.id)
            .await
    }

    async fn send(
        &self,
        chat_id: ChatId,
        text: impl ToString + Send + Sync,
    ) -> Result<TgMessage, <Bot as Requester>::Err> {
        self.send_message(chat_id, text.to_string()).await
    }

    async fn edit_or_send(
        &self,
        chat_id: ChatId,
        bot_msg: impl Into<Option<TgMessage>> + Send + Sync,
        text: impl ToString + Send + Sync,
    ) -> Result<TgMessage, <Bot as Requester>::Err> {
        let bot_msg: Option<TgMessage> = bot_msg.into();
        let text = text.to_string();

        match bot_msg {
            Some(msg) => self.edit(msg, text).await,
            None => self.send(chat_id, text).await,
        }
    }

    async fn edit(
        &self,
        msg: TgMessage,
        text: impl ToString + Send + Sync,
    ) -> Result<TgMessage, <Bot as Requester>::Err> {
        let prev_text = msg.text().unwrap_or_default();
        let new_text = text.to_string();

        if prev_text == unescape_md(&new_text).as_str() {
            return Ok(msg);
        }

        self.edit_message_text(msg.chat.id, msg.id, text.to_string())
            .await
    }
}
