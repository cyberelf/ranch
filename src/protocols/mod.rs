pub mod a2a;
pub mod openai;

use crate::agent::ProtocolType;
use crate::protocol::ProtocolAdapter;
use std::sync::Arc;

pub fn create_protocol_adapter(
    protocol_type: &ProtocolType,
    api_key: Option<String>,
) -> ProtocolAdapter {
    match protocol_type {
        ProtocolType::OpenAI => Arc::new(openai::OpenAIProtocol::new(api_key)),
        ProtocolType::A2A => Arc::new(a2a::A2AProtocol::new(api_key)),
    }
}