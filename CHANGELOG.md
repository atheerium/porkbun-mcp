# Changelog

## 0.1.0 (2026-09-13)
- Initial release: `porkbun-mcp` Rust stdio MCP server.
- Full Porkbun v3 API surface (91 endpoint categories) exposed as native MCP tools.
- Core categories implemented: utility/auth, account, domain lifecycle/admin, DNS CRUD, SSL, marketplace/closeouts, hosting, WordPress, Cloudflare connect, webhooks, sandbox, docs/mock.
- Safety features: automatic `Idempotency-Key` (UUIDv7) on mutations; `dryRun` passthrough; structured error vocabulary (`code`, `next_action`, `retryable`); no secret logging; sandbox test mode (`PORKBUN_SANDBOX=true` + instant sandbox keys).
- SEO/GEO optimized docs (`README.md`, `llms.txt`, `AGENTS.md`) with agent-discovery keywords (Claude Desktop, Cursor, Cline, opencode, rust mcp server, porkbun api v3 mcp).
- MIT license; published on crates.io (pending); GitHub repo `atheerium/porkbun-mcp`.
