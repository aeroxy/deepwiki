use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::sleep;

use crate::protocol::{read_msg, write_msg, DaemonRequest, DaemonResponse};

pub async fn send_to_daemon(request: &DaemonRequest) -> Result<DaemonResponse> {
    let mut stream = connect_daemon().await?;
    send_to_stream(&mut stream, request).await
}

async fn send_to_stream<S>(stream: &mut S, request: &DaemonRequest) -> Result<DaemonResponse>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let data = serde_json::to_vec(request)?;
    write_msg(stream, &data).await?;

    let response_data = read_msg(stream).await?;
    let response: DaemonResponse = serde_json::from_slice(&response_data)?;
    Ok(response)
}

async fn connect_daemon() -> Result<UnixStream> {
    let socket_path = socket_path();
    UnixStream::connect(&socket_path)
        .await
        .with_context(|| format!("Failed to connect to daemon at {:?}", socket_path))
}

fn socket_path() -> PathBuf {
    std::env::temp_dir().join("deepwiki-daemon.sock")
}

pub fn spawn_daemon() -> Result<()> {
    let exe = std::env::current_exe()?;
    Command::new(&exe)
        .arg("__daemon__")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    Ok(())
}

pub async fn wait_for_daemon() -> Result<()> {
    let mut delay = Duration::from_millis(50);
    let max_delay = Duration::from_millis(500);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);

    loop {
        if tokio::time::Instant::now() > deadline {
            anyhow::bail!("Daemon did not start within 5 seconds");
        }

        if connect_daemon().await.is_ok() {
            return Ok(());
        }

        sleep(delay).await;
        delay = (delay * 2).min(max_delay);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{DaemonRequest, DaemonResponse};
    use tokio::io::duplex;

    #[tokio::test]
    async fn test_send_to_daemon_protocol() {
        let (mut server, mut client) = duplex(1024);
        let request = DaemonRequest {
            command: "ask".to_string(),
            args: serde_json::json!({ "repo": "owner/repo", "question": "test" }),
            session: None,
        };

        let server_fut = async {
            let received = read_msg(&mut server).await.unwrap();
            let req: DaemonRequest = serde_json::from_slice(&received).unwrap();
            assert_eq!(req.command, "ask");
            let response = DaemonResponse {
                success: true,
                output: "test response".to_string(),
                error: "".to_string(),
                session: Some("bold-fox".to_string()),
                error_code: None,
            };
            let response_data = serde_json::to_vec(&response).unwrap();
            write_msg(&mut server, &response_data).await.unwrap();
        };

        let client_fut = async {
            let response = send_to_stream(&mut client, &request).await.unwrap();
            assert!(response.success);
            assert_eq!(response.output, "test response");
            assert_eq!(response.session, Some("bold-fox".to_string()));
        };

        tokio::join!(server_fut, client_fut);
    }
}