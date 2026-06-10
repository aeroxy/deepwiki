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

Each command is a single, stateless query — there is no conversation state to preserve across invocations. If you want to ask a follow-up, run another `ask` with the same `<repo>`.

```bash
deepwiki structure aeroxy/ast-bro           # list section titles
deepwiki read aeroxy/ast-bro                # full wiki as Markdown
deepwiki ask aeroxy/ast-bro "How does the call graph resolution work?"
```

## How it works

`deepwiki` connects to `mcp.deepwiki.com` using the Model Context Protocol over HTTPS. Each invocation opens a fresh MCP connection, sends a single tool call (`ask_question`, `read_wiki_structure`, or `read_wiki_contents`), prints the result, and exits.

No authentication required for public repositories.

### TLS

TLS certificate verification is **disabled by default** to support running inside monitored agent sandboxes with intercepting proxies. Set `DEEPWIKI_TLS_VERIFY=1` to restore strict certificate checking.

## Output format

Every command prints a header line followed by the result:

```markdown
## DeepWiki: <owner>/<repo> (<command>)

<content>
```

## Claude Code skill

A ready-to-install Claude Code skill is available in `skill/deepwiki/`.

## Testing

```bash
cargo test
```

## License

MIT
