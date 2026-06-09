# deepwiki

[![crates.io](https://img.shields.io/crates/v/deepwiki.svg)](https://crates.io/crates/deepwiki)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

Query GitHub repository wikis via [DeepWiki](https://deepwiki.com/) — without opening a browser.

Built for LLM coding agents and humans. Speaks [MCP](https://modelcontextprotocol.io/) (Model Context Protocol) to DeepWiki's backend to provide high-quality wiki structure, content, and AI-powered Q&A.

## Install

### Homebrew (macOS arm64)

```bash
brew install aeroxy/tap/deepwiki
```

### Cargo

```bash
cargo install deepwiki
```

## Usage

```bash
deepwiki structure aeroxy/ast-bro           # list section titles
deepwiki read aeroxy/ast-bro                # full wiki as Markdown
deepwiki ask aeroxy/ast-bro "How does the call graph resolution work?"
```

### Multi-turn Sessions

`deepwiki` supports persistent conversation sessions. When you ask a question, it returns a session ID that you can use for follow-ups:

```bash
$ deepwiki ask aeroxy/ast-bro "What does ast-bro do?"
...
[session: bold-fox]

$ deepwiki ask --session bold-fox "Tell me more about the dependency graph features."
```

Sessions are managed by a background daemon that keeps the MCP connection alive. The daemon auto-exits after 5 minutes of inactivity.

## How it works

`deepwiki` connects to `mcp.deepwiki.com` using the Model Context Protocol over HTTPS.

A background daemon handles session management, allowing multi-turn conversations by maintaining persistent stateful connections.

No authentication required for public repositories.

### TLS

TLS certificate verification is **disabled by default** to support running inside monitored agent sandboxes with intercepting proxies. Set `DEEPWIKI_TLS_VERIFY=1` to restore strict certificate checking.

## Output format

Every command prints a header line followed by the result:

```
## DeepWiki: <owner>/<repo> (<command>)

<content>
```

The session ID (if any) is printed to stderr.

## Claude Code skill

A ready-to-install Claude Code skill is available in `skill/deepwiki/`.

## Testing

```bash
cargo test
```

## License

MIT
