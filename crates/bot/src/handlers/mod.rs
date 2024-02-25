use crate::{BotArgs, EditOrSend, HelpGenerator, TagsGenerator, TaskSelector, TaskType};
use teloxide::{
    adaptors::DefaultParseMode, requests::Requester, types::Message as TgMessage, Bot, RequestError,
};

pub use help::*;
pub use note::*;

mod help;
mod note;

pub type TgBot = DefaultParseMode<Bot>;

pub struct MessageHandlerContext {
    pub tags_generator: TagsGenerator,
    pub task_selector: TaskSelector,
    pub help_generator: HelpGenerator,
}

impl MessageHandlerContext {
    pub fn new(args: &BotArgs) -> Self {
        let tags_generator = TagsGenerator::new(&args.secrets.mistral_token)
            .with_temperature(args.default_temperature)
            .with_random_seed(args.random_seed);
        let task_selector = TaskSelector::new(&args.secrets.mistral_token)
            .with_temperature(args.default_temperature)
            .with_random_seed(args.random_seed);
        let help_generator = HelpGenerator::new(&args.secrets.mistral_token)
            .with_temperature(args.default_temperature)
            .with_random_seed(args.random_seed);

        Self {
            tags_generator,
            task_selector,
            help_generator,
        }
    }

    pub async fn handle_message(
        &self,
        bot: &TgBot,
        user_msg: TgMessage,
    ) -> Result<(), RequestError> {
        let chat_id = user_msg.chat.id;
        let Some(text) = user_msg.text() else {
            bot.send_message(chat_id, "I can only process text messages")
                .await?;
            return Ok(());
        };

        log::debug!("Received message: {:?}", user_msg);
        let loading_message = bot.reply(&user_msg, r"*Processing message\.\.\.* ").await?;

        let task_type = match self.task_selector.select_task(text).await {
            Ok(task_type) => task_type,
            Err(e) => {
                log::warn!("Failed to select task: {}", e);
                bot.edit(loading_message, r"Something went wrong, sorry\.\.\.")
                    .await?;
                return Ok(());
            }
        };

        match task_type {
            TaskType::Note => {
                handle_note(self, bot, &user_msg, text, loading_message).await?;
            }
            TaskType::Help => {
                handle_help(self, bot, &user_msg, text, loading_message).await?;
            }
            TaskType::Unknown => {
                handle_help(self, bot, &user_msg, None, loading_message).await?;
            }
        }

        Ok(())
    }
}
