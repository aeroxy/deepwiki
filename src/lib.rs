pub mod cli;
pub mod client;
pub mod daemon;
pub mod daemon_client;
pub mod friendly;
pub mod output;
pub mod protocol;
pub mod session;
pub mod spinner;

use anyhow::Result;
use cli::{Cli, Command};
use clap::Parser;
use client::DeepWikiClient;
use daemon_client::{send_to_daemon, spawn_daemon, wait_for_daemon};
use protocol::DaemonRequest;
use spinner::Spinner;

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    let (repo, query_type) = repo_and_query_type(&cli.command);

    // 1. Check mock env var
    if let Ok(mock_text) = std::env::var("DEEPWIKI_CLI_MOCK_TEXT") {
        println!("{}", output::format_for_claude(&mock_text, repo, query_type));
        return Ok(());
    }

    // 2. Build DaemonRequest
    let request = build_request(&cli);

    // 3. Try daemon first
    if let Ok(resp) = send_to_daemon(&request).await {
        print_response(&resp, repo, query_type);
        return Ok(());
    }

    // 4. Daemon not running -- spawn it
    spawn_daemon()?;
    if let Err(e) = wait_for_daemon().await {
        // 5. Spawn failed -- direct fallback
        return run_direct_fallback(&cli, &e).await;
    }

    // 6. Retry via daemon
    match send_to_daemon(&request).await {
        Ok(resp) => {
            print_response(&resp, repo, query_type);
            Ok(())
        }
        Err(e) => run_direct_fallback(&cli, &e).await,
    }
}

fn build_request(cli: &Cli) -> DaemonRequest {
    match &cli.command {
        Command::Ask {
            repo,
            question,
            session,
        } => DaemonRequest {
            command: "ask".to_string(),
            args: serde_json::json!({ "repo": repo, "question": question }),
            session: session.clone(),
        },
        Command::Structure { repo } => DaemonRequest {
            command: "structure".to_string(),
            args: serde_json::json!({ "repo": repo }),
            session: None,
        },
        Command::Read { repo } => DaemonRequest {
            command: "read".to_string(),
            args: serde_json::json!({ "repo": repo }),
            session: None,
        },
    }
}

fn print_response(resp: &protocol::DaemonResponse, repo: &str, query_type: &str) {
    if resp.success {
        println!("{}", output::format_for_claude(&resp.output, repo, query_type));
        if let Some(session) = &resp.session {
            eprintln!("[session: {}]", session);
        }
    } else {
        eprintln!("Error: {}", resp.error);
        std::process::exit(resp.error_code.unwrap_or(1) as i32);
    }
}

async fn run_direct_fallback(cli: &Cli, error: &anyhow::Error) -> Result<()> {
    eprintln!("Warning: daemon unavailable ({}), running directly", error);
    let (repo, query_type) = repo_and_query_type(&cli.command);

    let spinner = Spinner::start("Connecting to DeepWiki...");
    let mut client = DeepWikiClient::connect().await?;

    spinner.set_message(&command_spinner_message(&cli.command));
    let text = match &cli.command {
        Command::Ask { repo, question, .. } => client.ask_question(repo, question).await?,
        Command::Structure { repo } => client.read_wiki_structure(repo).await?,
        Command::Read { repo } => client.read_wiki_contents(repo).await?,
    };
    spinner.finish();

    println!("{}", output::format_for_claude(&text, repo, query_type));
    client.cancel().await?;
    Ok(())
}

fn command_spinner_message(command: &Command) -> String {
    match command {
        Command::Ask { repo, .. } => format!("Asking DeepWiki about {}...", repo),
        Command::Structure { repo } => format!("Fetching wiki structure for {}...", repo),
        Command::Read { repo } => format!("Reading wiki contents for {}...", repo),
    }
}

fn repo_and_query_type(command: &Command) -> (&str, &str) {
    match command {
        Command::Ask { repo, .. } => (repo, "ask"),
        Command::Structure { repo } => (repo, "structure"),
        Command::Read { repo } => (repo, "read"),
    }
}
