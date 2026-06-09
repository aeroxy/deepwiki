use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DaemonRequest {
    pub command: String,
    pub args: serde_json::Value,
    pub session: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DaemonResponse {
    pub success: bool,
    pub output: String,
    pub error: String,
    pub session: Option<String>,
    pub error_code: Option<u32>,
}

pub fn socket_path() -> PathBuf {
    std::env::temp_dir().join("deepwiki-daemon.sock")
}

pub fn pid_path() -> PathBuf {
    std::env::temp_dir().join("deepwiki-daemon.pid")
}

pub async fn write_msg<W>(writer: &mut W, data: &[u8]) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let len = data.len() as u32;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(data).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn read_msg<R>(reader: &mut R) -> Result<Vec<u8>>
where
    R: AsyncReadExt + Unpin,
{
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    if len > MAX_MESSAGE_SIZE {
        anyhow::bail!("Message too large: {} bytes (max {})", len, MAX_MESSAGE_SIZE);
    }

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn test_write_read_roundtrip() {
        let (mut a, mut b) = duplex(1024);
        let msg = DaemonRequest {
            command: "ask".to_string(),
            args: serde_json::json!({ "repo": "owner/repo", "question": "test" }),
            session: None,
        };
        let data = serde_json::to_vec(&msg).unwrap();

        let write_fut = async {
            write_msg(&mut a, &data).await.unwrap();
        };
        let read_fut = async {
            let received = read_msg(&mut b).await.unwrap();
            serde_json::from_slice::<DaemonRequest>(&received).unwrap()
        };

        let (_, received) = tokio::join!(write_fut, read_fut);

        assert_eq!(received.command, "ask");
        assert_eq!(received.args["repo"], "owner/repo");
        assert!(received.session.is_none());
    }

    #[tokio::test]
    async fn test_oversized_message_rejected() {
        let (mut a, mut b) = duplex(1024);
        
        let write_fut = async {
            let len = (MAX_MESSAGE_SIZE + 1) as u32;
            a.write_all(&len.to_be_bytes()).await.unwrap();
        };
        let read_fut = async {
            read_msg(&mut b).await
        };

        let (_, result) = tokio::join!(write_fut, read_fut);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_daemon_request_serialization() {
        let req = DaemonRequest {
            command: "structure".to_string(),
            args: serde_json::json!({ "repo": "aeroxy/ast-bro" }),
            session: Some("bold-fox".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("structure"));
        assert!(json.contains("bold-fox"));
    }

    #[test]
    fn test_daemon_response_serialization() {
        let resp = DaemonResponse {
            success: true,
            output: "Hello".to_string(),
            error: "".to_string(),
            session: Some("clever-owl".to_string()),
            error_code: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("clever-owl"));
    }
}