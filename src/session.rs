use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::client::DeepWikiClient;
use crate::friendly::to_friendly;

#[derive(Debug)]
pub struct Session {
    pub client: DeepWikiClient,
    pub repo: String,
    pub created_at: Instant,
    pub last_active: Instant,
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
    next_id: u64,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            next_id: 0,
        }
    }

    pub async fn get_or_create(&mut self, session_key: Option<&str>, repo: &str) -> Result<(String, &mut Session)> {
        if let Some(key) = session_key {
            if let Some(session) = self.sessions.get_mut(key) {
                session.last_active = Instant::now();
                return Ok((key.to_string(), session));
            }
            anyhow::bail!("Session '{}' not found. Start a new conversation without --session.", key);
        }

        let id = self.next_id;
        self.next_id += 1;
        let internal_key = format!("deepwiki-session-{}", id);
        let friendly_name = to_friendly(&internal_key);

        let client = DeepWikiClient::connect().await?;
        let now = Instant::now();
        let session = Session {
            client,
            repo: repo.to_string(),
            created_at: now,
            last_active: now,
        };

        self.sessions.insert(friendly_name.clone(), session);
        let session = self.sessions.get_mut(&friendly_name).unwrap();
        Ok((friendly_name, session))
    }

    pub fn get(&mut self, session_key: &str) -> Option<&mut Session> {
        if let Some(session) = self.sessions.get_mut(session_key) {
            session.last_active = Instant::now();
            Some(session)
        } else {
            None
        }
    }

    pub fn cleanup_idle(&mut self, max_idle: Duration) {
        let now = Instant::now();
        self.sessions.retain(|_, session| {
            now.duration_since(session.last_active) < max_idle
        });
    }

    pub async fn shutdown(&mut self) {
        for (_, mut session) in self.sessions.drain() {
            let _ = session.client.cancel().await;
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_new() {
        let manager = SessionManager::new();
        assert_eq!(manager.sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_get_or_create_without_key_creates_new() {
        let mut manager = SessionManager::new();
        let (name, session) = manager.get_or_create(None, "aeroxy/ast-bro").await.unwrap();
        assert!(name.contains('-'));
        assert_eq!(session.repo, "aeroxy/ast-bro");
    }

    #[tokio::test]
    async fn test_get_or_create_with_existing_key_reuses() {
        let mut manager = SessionManager::new();
        let (name1, _) = manager.get_or_create(None, "owner/repo").await.unwrap();
        let (name2, session) = manager.get_or_create(Some(&name1), "owner/repo").await.unwrap();
        assert_eq!(name1, name2);
        assert_eq!(session.repo, "owner/repo");
    }

    #[tokio::test]
    async fn test_get_or_create_with_bogus_key_errors() {
        let mut manager = SessionManager::new();
        let result = manager.get_or_create(Some("nonexistent"), "owner/repo").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_cleanup_idle() {
        let mut manager = SessionManager::new();
        let (_name, _) = manager.get_or_create(None, "owner/repo").await.unwrap();
        assert_eq!(manager.sessions.len(), 1);

        std::thread::sleep(Duration::from_millis(10));
        manager.cleanup_idle(Duration::from_millis(5));
        assert_eq!(manager.sessions.len(), 0);
    }
}