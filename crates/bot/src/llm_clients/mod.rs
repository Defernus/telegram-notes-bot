mod help_generator;
mod tags_generator;
mod task_selector;

pub use help_generator::*;
pub use tags_generator::*;
pub use task_selector::*;

#[macro_export]
macro_rules! base_llm_methods {
    () => {
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

        /// Set the Mistral model type.
        pub fn with_model(mut self, model: MistralModelType) -> Self {
            self.base_client = self.base_client.with_model(model);
            self
        }
    };
}
