use crate::models::{
    ConversationDetail, ConversationSummary,
};
use crate::parsers::{AgentParser, ParseError};

pub struct HermesParser;

impl HermesParser {
    pub fn new() -> Self {
        Self
    }
}

impl AgentParser for HermesParser {
    fn list_conversations(&self) -> Result<Vec<ConversationSummary>, ParseError> {
        Ok(vec![])
    }

    fn get_conversation(&self, _conversation_id: &str) -> Result<ConversationDetail, ParseError> {
        Err(ParseError::ConversationNotFound(
            "Hermes does not store conversations locally".into(),
        ))
    }
}
