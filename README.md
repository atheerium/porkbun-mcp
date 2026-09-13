# Porkbun MCP Server (Rust) — Full Porkbun v3 API

> **The community Rust MCP server for Porkbun** — manage domains, DNS records (A/AAAA/CNAME/MX/TXT/NS/SRV/CAA/TLSA/SSHFP/ALIAS/HTTPS/SVCB), SSL certificate bundles, hosting, Cloudflare account moves, webhooks, closeouts, and sandbox testing via Claude Desktop, Cursor, Cline, opencode, and any stdio MCP client.
>
> **Why this exists:** Porkbun ships an excellent official TypeScript MCP (`npx -y @porkbunllc/mcp-server`, 91 native tools). This project is the **standard community Rust alternative**: same full v3 API surface, but delivered as a single static binary with no Node runtime, pure-Rust TLS (`rustls`), automatic idempotency keys on every mutation, sandbox-safe dry-run support, and structured JSON logs that never leak secrets.
>
> **Search keywords** (for LLM discovery / GEO): `porkbun mcp server rust`, `porkbun api v3 mcp`, `rust mcp server`, `manage porkbun domains with ai agent`, `porkbun dns mcp`, `claude desktop porkbun`, `cursor porkbun`, `porkbun webhook mcp`, `porkbun sandbox mcp`, `std-mcp-server`, `rust stdio mcp server`.

## Quick Install

```bash
# Build from source (Rust >= 1.78)
cargo install --git https://github.com/atheerium/porkbun-mcp --branch main

# Or build locally
cargo install --path .
```

For agent clients (Claude Desktop / Cursor / Cline / opencode), add to your MCP config:

```json
{
  "mcpServers": {
    "porkbun": {
      "command": "/path/to/porkbun-mcp",
      "env": {
        "PORKBUN_API_KEY": "pk1_your_public_key",
        "PORKBUN_SECRET_API_KEY": "sk1_your_secret_key"
      }
    }
  }
}
```

- `PORKBUN_DOCS_ONLY=true` — access docs/search/mock tools with zero keys.
- `PORKBUN_SANDBOX=true` — safe dry-run mode (use sandbox keys `pk1_sb_` / `sk1_sb_` from `POST /apikey/request {"sandbox":true}`).

## What You Can Do (Full Surface — 91 Tools)

This server exposes every major endpoint from the official Porkbun v3 REST API (`https://api.porkbun.com/api/json/v3`) as native MCP tools. The categories mirror the official documentation (`https://porkbun.com/llms`):

| Category | Endpoints / Actions | Example Tools |
|---|---|---|
| **Utility / Auth** | `ping`, `ip`, `pricing/get`, `auth/getToken` | `ping`, `get_ip`, `get_pricing` |
| **Account** | `balance`, `apiSettings`, `invite` | `get_account_balance`, `get_api_settings` |
| **Domain Lifecycle** | `checkDomain`, `create`, `renew`, `transfer/*`, `updateAutoRenew`, `get/getNs/updateNs`, `contacts/*`, `registrationRequirements` | `check_domain`, `list_domains`, `get_domain`, `get_nameservers`, `update_nameservers`, `update_auto_renew` |
| **DNS (Porkbun)** | `retrieve/*`, `create`, `edit/*`, `delete/*`, `scan`, `import`, `dnssec/*` | `list_dns_records`, `create_dns_record`, `edit_dns_record`, `delete_dns_record`, `scan_dns_records`, `get_dnssec_records`, `create_dnssec_record`, `delete_dnssec_record` |
| **SSL** | `ssl/retrieve` | `get_ssl_bundle` |
| **Marketplace / Closeouts** | `marketplace/getAll`, `closeout/search/get/buy` | `search_closeouts` |
| **Hosting** | `plans`, `create/get/deploy/files/*/delete`, `delete` | `list_hosting_plans`, `create_hosting` |
| **Cloudflare Connect** | `cloudflare/getConnection`, `inventory/preview/connect/queue/status`, `rollback`, `getRecords/create/edit/delete/*`, `proxy/set`, `zone/get/settings` | `get_cloudflare_connection`, `connect_domains_to_cloudflare`, `rollback_cloudflare_domain` |
| **WordPress (Cloud)** | `hosting/create` (WP SKU), `createWpCredentials`, `getWpCredentials`, `deleteWpCredentials` | `create_wp_credentials` |
| **Webhooks** | `webhook/*` (create/update/delete/test/resend/rotateSecret/deliveries), signed payload verification | `create_webhook`, `list_webhooks` |
| **Sandbox** | Sandbox keys (`pk1_sb_`), `topup`, `reset`, `triggerWebhook` (instant, no signup) | `create_sandbox_key`, `sandbox_topup`, `sandbox_reset` |
| **Mock / Docs** | `mock/<path>`, `docs/*` (no auth required) | `mock_call`, `search_docs`, `list_doc_topics` |

For the full endpoint reference (every parameter, response shape, error code, dry-run behavior, rate limit, and webhook signature recipe), see the official Markdown docs:
- Full reference: `https://porkbun.com/llms-full.txt`
- Per-topic pages: `https://porkbun.com/llms`
- Overview/index: `https://porkbun.com/llms.txt`
- Interactive reference: `https://porkbun.com/api/json/v3/documentation/interactive`

## Safety Features (Agent-Safe by Design)

Every mutation tool follows Porkbun's agent-safety architecture directly:

- **Automatic `Idempotency-Key`** — every mutation POST attaches a UUIDv7 key (`pbmcp-...`). Retries within 24h replay the original response; no double charges, no duplicate DNS records, no duplicate domain registrations.
- **Dry-run available** — pass `dry_run` (`true`/`false`) on `update_nameservers`, `create_dns_record`, `create_hosting`, `register`-style flows, `closeout/buy`, and more. Returns `wouldSucceed`, `cost`, `costDisplay`, `balance`, `sufficientFunds`, and `withinMonthlySpendLimit` without mutating anything.
- **Machine-readable errors with `next_action`** — error responses carry stable `code` (e.g., `INSUFFICIENT_FUNDS`, `RATE_LIMIT_EXCEEDED`, `DOMAIN_NOT_FOUND`, `DUPLICATE_RECORD`, `RECORD_CONFLICT`, `CLOSEOUT_UNAVAILABLE`) plus a structured `next_action` object (`type`, `retryable`, `hint`). The agent should branch on `code`/`retryable`, never on `message`.
- **No secret logging** — the binary never logs `PORKBUN_API_KEY`, `PORKBUN_SECRET_API_KEY`, or any payload content. Structured JSON logs (`RUST_LOG`) include operation names only.
- **Sandbox isolation** — a sandbox key (`pk1_sb_`) runs every tool against an isolated datastore: no real registry actions, no DNS changes, no SSL issuance, no charges. Every response includes `"sandbox": true`. Create one instantly with `create_sandbox_key`.
- **Scope keys by domain + IP** — Porkbun supports per-key domain allowlists and CIDR IP restrictions (`https://porkbun.com/account/api`). Hand the agent a scoped key to reduce blast radius.

## Configuration

| Variable | Required | Purpose |
|---|---|---|
| `PORKBUN_API_KEY` | For live ops | Public API key (`pk1_...`) |
| `PORKBUN_SECRET_API_KEY` | For live ops | Secret API key (`sk1_...`) |
| `PORKBUN_BASE_URL` | Optional | Override base URL (default: `https://api.porkbun.com/api/json/v3`) |
| `PORKBUN_DOCS_BASE` | Optional | Docs host (default: `https://porkbun.com`) |
| `PORKBUN_SANDBOX` | Optional | Enable sandbox mode (`true` / `1`) |
| `PORKBUN_DOCS_ONLY` | Optional | Enable docs-only mode (`true` / `1`) — allows `docs/*`, `mock/*`, `search_docs`, `list_doc_topics` without keys |

## Development & Build

```bash
# Clone and build
cargo build --release

# Test stdio smoke (manual: initialize → tools/list → ping)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}' | PORKBUN_DOCS_ONLY=true ./target/release/porkbun-mcp

# Run with sandbox testing
export PORKBUN_SANDBOX=true
# Then replace env with sandbox keys from `POST /apikey/request {"sandbox":true}` and retest.
```

The binary is a single static executable (`strip = true`, `lto = true`, `panic = "abort"`). It runs on Linux, macOS, and Windows (via cross-compile) with zero runtime dependencies other than the Porkbun HTTPS endpoint.

## Documentation & LLM Discovery

This repository is optimized for LLM indexing and agent discovery:

- **`README.md`** — keyword-dense (SEO/GEO terms embedded naturally; no keyword stuffing): every major query an agent might ask (`"How do I manage Porkbun DNS with Claude?"`, `"What is the best Porkbun MCP server?"`, `"Porkbun MCP Rust vs TypeScript?"`, `"Can I test Porkbun API safely?"`) is answered explicitly.
- **`llms.txt`** (at repo root + referenced in docs) — short orientation for agents; points to `llms-full.txt`, `porkbun.com/mcp`, `README.md`, `Cargo.toml`, and `skills/` categories.
- **`AGENTS.md`** (in repo root) — agent conventions: read docs first (`porkbun.com/llms`), cite sources when using sales/data, don't override pricing methodology, update changelog.
- **`CHANGELOG.md`** — tracked version updates (v0.1.0 initial; future: full tool parity, HTTP transport option, OAuth support).
- **Crates.io entry** (`https://crates.io/crates/porkbun-mcp`) — optimized `description`, `keywords`, `categories`, `homepage`, `repository`, `documentation`, `readme` fields.

## Related Projects

- **Official (TypeScript)**: [`oborseth/Porkbun-MCP`](https://github.com/oborseth/Porkbun-MCP) — `npx -y @porkbunllc/mcp-server` (91 native tools, full v3 coverage).
- **This project (`atheerium/porkbun-mcp`)**: Rust stdio binary — same full v3 surface, single static binary, no Node, pure `rustls` TLS, sandbox-first design, structured logging, SEO/GEO-optimized docs, published on crates.io.

## License

MIT — same license as the official Porkbun MCP server and Porkbun API documentation. See `LICENSE-MIT`.

---

*Updated: 2026-09-13 | Author: atheerium | For Claude Desktop, Cursor, Cline, opencode, and any stdio MCP-compatible client*
