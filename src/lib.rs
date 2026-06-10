pub mod cli;
pub mod client;
pub mod output;
pub mod spinner;

use anyhow::Result;
use cli::{Cli, Command};
use clap::Parser;
use client::DeepWikiClient;
use spinner::Spinner;

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let (repo, query_type) = repo_and_query_type(&cli.command);

    if let Ok(mock_text) = std::env::var("DEEPWIKI_CLI_MOCK_TEXT") {
        println!("{}", output::format_for_claude(&mock_text, repo, query_type));
        return Ok(());
    }

    let spinner = Spinner::start("Connecting to DeepWiki...");
    let mut client = DeepWikiClient::connect().await?;

    spinner.set_message(&command_spinner_message(&cli.command));
    let text = match &cli.command {
        Command::Ask { repo, question } => client.ask_question(repo, question).await?,
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
