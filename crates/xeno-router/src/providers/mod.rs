//! Provider backends and adapter implementations.

pub mod anthropic;
pub mod deepseek;
pub mod gemini;
pub mod groq;
pub mod local_openai;
pub mod mock;
pub mod openai;
pub mod sse;

pub use anthropic::AnthropicProvider;
pub use deepseek::DeepSeekProvider;
pub use gemini::GeminiProvider;
pub use groq::GroqProvider;
pub use local_openai::LocalOpenAIProvider;
pub use mock::{MockConfig, MockProvider};
pub use openai::OpenAIProvider;
pub use sse::SseEvent;
