use std::{collections::{BTreeMap, HashMap}, sync::Arc};

use tokio::sync::RwLock;

use crate::models::{AgentMessage, MemorySnapshot};

#[derive(Clone, Debug)]
pub struct MemoryStore {
    sessions: Arc<RwLock<HashMap<String, SessionMemory>>>,
    max_messages: usize,
}

impl MemoryStore {
    pub fn new(max_messages: usize) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            max_messages,
        }
    }

    pub async fn recent_messages(&self, session_id: &str) -> Vec<AgentMessage> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|memory| memory.messages.clone())
            .unwrap_or_default()
    }

    pub async fn append_exchange(
        &self,
        session_id: &str,
        mut request_messages: Vec<AgentMessage>,
        assistant_message: AgentMessage,
    ) {
        let mut sessions = self.sessions.write().await;
        let session = sessions.entry(session_id.to_owned()).or_default();
        session.messages.append(&mut request_messages);
        session.messages.push(assistant_message);

        if session.messages.len() > self.max_messages {
            let keep_from = session.messages.len().saturating_sub(self.max_messages);
            session.messages = session.messages.split_off(keep_from);
        }
    }

    pub async fn remember_fact(&self, session_id: &str, key: String, value: String) {
        let mut sessions = self.sessions.write().await;
        let session = sessions.entry(session_id.to_owned()).or_default();
        session.facts.insert(key, value);
    }

    pub async fn recall_fact(&self, session_id: &str, key: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .and_then(|session| session.facts.get(key).cloned())
    }

    pub async fn list_facts(&self, session_id: &str) -> BTreeMap<String, String> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|session| session.facts.clone())
            .unwrap_or_default()
    }

    pub async fn snapshot(&self, session_id: &str) -> MemorySnapshot {
        let sessions = self.sessions.read().await;
        match sessions.get(session_id) {
            Some(session) => MemorySnapshot {
                session_id: session_id.to_owned(),
                messages: session.messages.clone(),
                facts: session.facts.clone(),
            },
            None => MemorySnapshot {
                session_id: session_id.to_owned(),
                messages: Vec::new(),
                facts: BTreeMap::new(),
            },
        }
    }

    pub async fn clear(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
    }
}

#[derive(Clone, Debug, Default)]
struct SessionMemory {
    messages: Vec<AgentMessage>,
    facts: BTreeMap<String, String>,
}
