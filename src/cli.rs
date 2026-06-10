use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "deepwiki", about = "Query GitHub repo wikis via DeepWiki from the terminal", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Ask an AI-powered question about a repository
    Ask {
        /// Repository name (e.g. aeroxy/ast-bro)
        repo: String,
        /// Question to ask
        question: String,
    },
    /// List wiki topics for a repository
    Structure {
        /// Repository name (e.g. aeroxy/ast-bro)
        repo: String,
    },
    /// Read full wiki contents for a repository
    Read {
        /// Repository name (e.g. aeroxy/ast-bro)
        repo: String,
    },
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use clap::Parser;

    #[test]
    fn parses_ask_command() {
        let cli = Cli::try_parse_from(["deepwiki", "ask", "aeroxy/ast-bro", "How it works?"])
            .expect("ask command should parse");
        match cli.command {
            Command::Ask { repo, question } => {
                assert_eq!(repo, "aeroxy/ast-bro");
                assert_eq!(question, "How it works?");
            }
            _ => panic!("expected ask command"),
        }
    }

    #[test]
    fn parses_structure_command() {
        let cli = Cli::try_parse_from(["deepwiki", "structure", "aeroxy/ast-bro"])
            .expect("structure command should parse");
        match cli.command {
            Command::Structure { repo } => assert_eq!(repo, "aeroxy/ast-bro"),
            _ => panic!("expected structure command"),
        }
    }

    #[test]
    fn parses_read_command() {
        let cli = Cli::try_parse_from(["deepwiki", "read", "aeroxy/ast-bro"])
            .expect("read command should parse");
        match cli.command {
            Command::Read { repo } => assert_eq!(repo, "aeroxy/ast-bro"),
            _ => panic!("expected read command"),
        }
    }

    #[test]
    fn fails_when_required_args_are_missing() {
        let result = Cli::try_parse_from(["deepwiki", "ask", "aeroxy/ast-bro"]);
        assert!(result.is_err());
    }
}
