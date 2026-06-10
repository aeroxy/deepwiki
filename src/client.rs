use anyhow::{Context, Result};
use rmcp::model::CallToolRequestParams;
#[cfg(not(test))]
use rmcp::{
    model::{CallToolResult, RawContent},
    serve_client,
    service::RunningService,
    RoleClient,
    transport::streamable_http_client::StreamableHttpClientTransport,
};
use serde_json::json;
use std::future::Future;

pub const MCP_ENDPOINT: &str = "https://mcp.deepwiki.com/mcp";

pub(crate) struct ToolCallSpec {
    pub(crate) name: &'static str,
    pub(crate) arguments: serde_json::Value,
    pub(crate) error_context: &'static str,
}

#[derive(Debug)]
pub struct DeepWikiClient {
    #[cfg(not(test))]
    service: Option<RunningService<RoleClient, ()>>,
}

impl DeepWikiClient {
    #[cfg(not(test))]
    pub async fn connect() -> Result<Self> {
        let reqwest_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(!tls_verification_enabled())
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to build reqwest client")?;

        let config = rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig::with_uri(MCP_ENDPOINT);
        let transport = StreamableHttpClientTransport::with_client(reqwest_client, config);
        let service = serve_client((), transport)
            .await
            .context("Failed to connect to mcp.deepwiki.com")?;
        Ok(Self { service: Some(service) })
    }

    #[cfg(test)]
    pub async fn connect() -> Result<Self> {
        Ok(Self {})
    }

    pub async fn ask_question(&self, repo: &str, question: &str) -> Result<String> {
        let spec = ToolCallSpec {
            name: "ask_question",
            arguments: json!({ "repoName": repo, "question": question }),
            error_context: "Failed to call ask_question",
        };
        self.call_tool_text(spec).await
    }

    pub async fn read_wiki_structure(&self, repo: &str) -> Result<String> {
        let spec = ToolCallSpec {
            name: "read_wiki_structure",
            arguments: json!({ "repoName": repo }),
            error_context: "Failed to call read_wiki_structure",
        };
        self.call_tool_text(spec).await
    }

    pub async fn read_wiki_contents(&self, repo: &str) -> Result<String> {
        let spec = ToolCallSpec {
            name: "read_wiki_contents",
            arguments: json!({ "repoName": repo }),
            error_context: "Failed to call read_wiki_contents",
        };
        self.call_tool_text(spec).await
    }

    pub async fn cancel(&mut self) -> Result<()> {
        #[cfg(not(test))]
        if let Some(svc) = self.service.take() {
            svc.cancel().await?;
        }
        Ok(())
    }

    #[cfg(not(test))]
    fn service(&self) -> &RunningService<RoleClient, ()> {
        self.service.as_ref().expect("service should be present")
    }

    #[cfg_attr(test, allow(unused_variables))]
    pub(crate) async fn call_tool_text(&self, spec: ToolCallSpec) -> Result<String> {
        #[cfg(test)]
        {
            if let Ok(mock) = std::env::var("DEEPWIKI_CLI_MOCK_TEXT") {
                return Ok(mock);
            }
            Ok("mock response".to_string())
        }

        #[cfg(not(test))]
        {
            call_tool_text_with(spec, |params| async {
                let result = self.service().peer().call_tool(params).await?;
                Ok(extract_text_segments(result))
            })
            .await
        }
    }
}

async fn call_tool_text_with<TCall, TFut>(spec: ToolCallSpec, caller: TCall) -> Result<String>
where
    TCall: FnOnce(CallToolRequestParams) -> TFut,
    TFut: Future<Output = Result<Vec<String>>>,
{
    let params = build_call_tool_request_params(&spec)?;
    let text_segments = caller(params)
        .await
        .with_context(|| spec.error_context.to_string())?;
    Ok(join_text_segments(text_segments))
}

fn build_call_tool_request_params(spec: &ToolCallSpec) -> Result<CallToolRequestParams> {
    let arguments = spec
        .arguments
        .as_object()
        .cloned()
        .context("Tool arguments must be a JSON object")?;
    Ok(CallToolRequestParams::new(spec.name).with_arguments(arguments))
}

#[cfg(not(test))]
fn extract_text_segments(result: CallToolResult) -> Vec<String> {
    result
        .content
        .into_iter()
        .filter_map(|c| {
            if let RawContent::Text(t) = c.raw {
                Some(t.text)
            } else {
                None
            }
        })
        .collect()
}

fn join_text_segments(text_segments: Vec<String>) -> String {
    text_segments.join("\n")
}

/// TLS certificate verification is **disabled by default**. These agents
/// usually run inside monitored sandboxes whose TLS-intercepting proxies
/// present certificates that don't chain to a trusted root, which would
/// otherwise make every request fail with an opaque cert error the agent
/// can't fix. Set `DEEPWIKI_TLS_VERIFY` to `1`/`true`/`yes` to restore
/// strict verification.
fn tls_verification_enabled() -> bool {
    matches!(
        std::env::var("DEEPWIKI_TLS_VERIFY")
            .ok()
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

#[cfg(test)]
mod tls_tests {
    use super::tls_verification_enabled;
    use std::env;

    #[test]
    fn disabled_by_default() {
        env::remove_var("DEEPWIKI_TLS_VERIFY");
        assert!(!tls_verification_enabled());
    }

    #[test]
    fn enabled_by_truthy_values() {
        for value in ["1", "true", "yes", "TRUE", "  yes  "] {
            env::set_var("DEEPWIKI_TLS_VERIFY", value);
            assert!(
                tls_verification_enabled(),
                "expected {:?} to enable TLS",
                value
            );
        }
    }

    #[test]
    fn disabled_by_other_values() {
        for value in ["0", "false", "no", "off", ""] {
            env::set_var("DEEPWIKI_TLS_VERIFY", value);
            assert!(
                !tls_verification_enabled(),
                "expected {:?} to disable TLS",
                value
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn build_params_for_ask_question() {
        let spec = ToolCallSpec {
            name: "ask_question",
            arguments: json!({ "repoName": "aeroxy/ast-bro", "question": "How?" }),
            error_context: "Failed",
        };
        let params = build_call_tool_request_params(&spec).expect("params should be built");

        assert_eq!(params.name, "ask_question");
        let arguments = params.arguments.expect("arguments should exist");
        assert_eq!(
            arguments.get("repoName"),
            Some(&serde_json::Value::String("aeroxy/ast-bro".to_string()))
        );
        assert_eq!(
            arguments.get("question"),
            Some(&serde_json::Value::String("How?".to_string()))
        );
    }

    #[test]
    fn build_params_for_repo_only_tools() {
        for tool_name in ["read_wiki_structure", "read_wiki_contents"] {
            let spec = ToolCallSpec {
                name: tool_name,
                arguments: json!({ "repoName": "owner/repo" }),
                error_context: "Failed",
            };
            let params = build_call_tool_request_params(&spec).expect("params should be built");
            assert_eq!(params.name, tool_name);
            let arguments = params.arguments.expect("arguments should exist");
            assert_eq!(
                arguments.get("repoName"),
                Some(&serde_json::Value::String("owner/repo".to_string()))
            );
        }
    }

    #[tokio::test]
    async fn call_tool_text_with_mocked_caller_joins_lines() {
        let spec = ToolCallSpec {
            name: "ask_question",
            arguments: json!({ "repoName": "owner/repo", "question": "Q" }),
            error_context: "Failed to call ask_question",
        };
        let text = call_tool_text_with(spec, |_params| async {
            Ok(vec!["line1".to_string(), "line2".to_string()])
        })
        .await
        .expect("call should succeed");
        assert_eq!(text, "line1\nline2");
    }

    #[tokio::test]
    async fn call_tool_text_with_mocked_caller_wraps_error_context() {
        let spec = ToolCallSpec {
            name: "read_wiki_structure",
            arguments: json!({ "repoName": "owner/repo" }),
            error_context: "Failed to call read_wiki_structure",
        };
        let err = call_tool_text_with(spec, |_params| async { Err(anyhow!("boom")) })
            .await
            .expect_err("call should fail");
        let err_text = format!("{:#}", err);
        assert!(err_text.contains("Failed to call read_wiki_structure"));
        assert!(err_text.contains("boom"));
    }
}
