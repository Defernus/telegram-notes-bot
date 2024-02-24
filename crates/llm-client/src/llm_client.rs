pub trait ImplMessage: ToString + Send + Sync {}
impl<T: ToString + Send + Sync> ImplMessage for T {}

#[async_trait::async_trait]
pub trait LlmClient {
    async fn reset_chat(&mut self) -> eyre::Result<()>;

    async fn send_message_without_history<T: ImplMessage>(
        &self,
        message: T,
    ) -> eyre::Result<String>;

    async fn send_message<T: ImplMessage>(&mut self, message: T) -> eyre::Result<String>;

    fn last_response(&self) -> Option<String>;
}
