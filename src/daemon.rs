use anyhow::{Context, Result};
use std::fs;
use std::time::Duration;
use tokio::net::{UnixListener, UnixStream};
use tokio::time::timeout;

use crate::client::{DeepWikiClient, ToolCallSpec};
use crate::protocol::{read_msg, socket_path, write_msg, DaemonRequest, DaemonResponse, pid_path};
use crate::session::SessionManager;

pub async fn run_daemon() -> Result<()> {
    let pid = std::process::id();
    fs::write(pid_path(), pid.to_string())?;

    let socket_path = socket_path();
    if socket_path.exists() {
        let _ = fs::remove_file(&socket_path);
    }

    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("Failed to bind to socket {:?}", socket_path))?;

    let mut sessions = SessionManager::new();
    let idle_timeout = Duration::from_secs(
        std::env::var("DEEPWIKI_DAEMON_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300),
    );

    loop {
        match timeout(idle_timeout, listener.accept()).await {
            Ok(Ok((stream, _))) => {
                if let Err(e) = handle_connection(stream, &mut sessions).await {
                    eprintln!("Error handling connection: {:#}", e);
                }
            }
            Ok(Err(e)) => {
                eprintln!("Error accepting connection: {:#}", e);
            }
            Err(_) => {
                // Idle timeout
                break;
            }
        }
    }

    sessions.shutdown().await;
    let _ = fs::remove_file(&socket_path);
    let _ = fs::remove_file(pid_path());

    Ok(())
}

async fn handle_connection(mut stream: UnixStream, sessions: &mut SessionManager) -> Result<()> {
    let data = read_msg(&mut stream).await?;
    let request: DaemonRequest = serde_json::from_slice(&data)?;

    let response = handle_request(sessions, request).await?;
    let response_data = serde_json::to_vec(&response)?;
    write_msg(&mut stream, &response_data).await?;

    Ok(())
}

async fn handle_request(sessions: &mut SessionManager, request: DaemonRequest) -> Result<DaemonResponse> {
    let repo = request.args["repo"].as_str().unwrap_or("").to_string();
    if repo.is_empty() {
        return Ok(DaemonResponse {
            success: false,
            output: "".to_string(),
            error: "Repository name is required".to_string(),
            session: None,
            error_code: Some(1),
        });
    }

    match request.command.as_str() {
        "ask" => {
            let question = request.args["question"].as_str().unwrap_or("").to_string();
            match sessions.get_or_create(request.session.as_deref(), &repo).await {
                Ok((session_name, session)) => {
                    let spec = ToolCallSpec {
                        name: "ask_question",
                        arguments: serde_json::json!({ "repoName": repo, "question": question }),
                        error_context: "Failed to call ask_question",
                    };
                    match session.client.call_tool_text(spec).await {
                        Ok(output) => Ok(DaemonResponse {
                            success: true,
                            output,
                            error: "".to_string(),
                            session: Some(session_name),
                            error_code: None,
                        }),
                        Err(e) => Ok(DaemonResponse {
                            success: false,
                            output: "".to_string(),
                            error: format!("{:#}", e),
                            session: Some(session_name),
                            error_code: Some(2),
                        }),
                    }
                }
                Err(e) => Ok(DaemonResponse {
                    success: false,
                    output: "".to_string(),
                    error: format!("{:#}", e),
                    session: None,
                    error_code: Some(3),
                }),
            }
        }
        "structure" => {
            // Ephemeral client for one-shot if no session provided
            if let Some(sess_key) = request.session {
                if let Some(session) = sessions.get(&sess_key) {
                    match session.client.read_wiki_structure(&repo).await {
                        Ok(output) => Ok(DaemonResponse {
                            success: true,
                            output,
                            error: "".to_string(),
                            session: Some(sess_key),
                            error_code: None,
                        }),
                        Err(e) => Ok(DaemonResponse {
                            success: false,
                            output: "".to_string(),
                            error: format!("{:#}", e),
                            session: Some(sess_key),
                            error_code: Some(2),
                        }),
                    }
                } else {
                    Ok(DaemonResponse {
                        success: false,
                        output: "".to_string(),
                        error: format!("Session '{}' not found", sess_key),
                        session: None,
                        error_code: Some(3),
                    })
                }
            } else {
                let mut client = DeepWikiClient::connect().await?;
                let res = client.read_wiki_structure(&repo).await;
                let _ = client.cancel().await;
                match res {
                    Ok(output) => Ok(DaemonResponse {
                        success: true,
                        output,
                        error: "".to_string(),
                        session: None,
                        error_code: None,
                    }),
                    Err(e) => Ok(DaemonResponse {
                        success: false,
                        output: "".to_string(),
                        error: format!("{:#}", e),
                        session: None,
                        error_code: Some(2),
                    }),
                }
            }
        }
        "read" => {
            if let Some(sess_key) = request.session {
                if let Some(session) = sessions.get(&sess_key) {
                    match session.client.read_wiki_contents(&repo).await {
                        Ok(output) => Ok(DaemonResponse {
                            success: true,
                            output,
                            error: "".to_string(),
                            session: Some(sess_key),
                            error_code: None,
                        }),
                        Err(e) => Ok(DaemonResponse {
                            success: false,
                            output: "".to_string(),
                            error: format!("{:#}", e),
                            session: Some(sess_key),
                            error_code: Some(2),
                        }),
                    }
                } else {
                    Ok(DaemonResponse {
                        success: false,
                        output: "".to_string(),
                        error: format!("Session '{}' not found", sess_key),
                        session: None,
                        error_code: Some(3),
                    })
                }
            } else {
                let mut client = DeepWikiClient::connect().await?;
                let res = client.read_wiki_contents(&repo).await;
                let _ = client.cancel().await;
                match res {
                    Ok(output) => Ok(DaemonResponse {
                        success: true,
                        output,
                        error: "".to_string(),
                        session: None,
                        error_code: None,
                    }),
                    Err(e) => Ok(DaemonResponse {
                        success: false,
                        output: "".to_string(),
                        error: format!("{:#}", e),
                        session: None,
                        error_code: Some(2),
                    }),
                }
            }
        }
        _ => Ok(DaemonResponse {
            success: false,
            output: "".to_string(),
            error: format!("Unknown command: {}", request.command),
            session: None,
            error_code: Some(4),
        }),
    }
}
