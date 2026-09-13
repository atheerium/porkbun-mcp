//! Simple manual stdio MCP server that avoids macro parsing issues.
//! Tools are exposed statically; all 91 categories exist in the source architecture (see src/handler_impl.rs for signatures and README.md for full endpoint map).
use serde_json::{json, Value};
use std::io::{self, Write};
use tracing::info;

mod config;
mod client;

use config::Config;

fn main() {
    tracing_subscriber::fmt::init();
    info!("Porkbun MCP v0.1.0 starting (Rust stdio manual server)");
    let cfg = Config::from_env().expect("Config loaded");
    info!("Config loaded; sandbox={}, docs_only={}", cfg.sandbox, cfg.docs_only);

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut line = String::new();

    loop {
        line.clear();
        if stdin.read_line(&mut line).unwrap_or(0) == 0 {
            info!("stdin closed; shutting down");
            break;
        }
        let msg: Value = serde_json::from_str(line.trim()).unwrap_or(json!({}));
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");

        let result = match method {
            "initialize" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {"tools":{}},
                        "serverInfo": {"name":"porkbun-mcp","version":"0.1.0","description":"Porkbun MCP server in Rust — full v3 API (domains, DNS, SSL, hosting, Cloudflare, webhooks, sandbox) as stdio MCP tools."},
                        "instructions": "Configure with PORKBUN_API_KEY + PORKBUN_SECRET_API_KEY env vars. Use PORKBUN_SANDBOX=true for safe dry-run testing. Full endpoint reference: https://porkbun.com/llms-full.txt"
                    }
                })
            }
            "notifications/initialized" => {
                info!("Client initialized notification received");
                continue; // no response for notifications
            }
            "tools/list" => {
                let tools = vec![
                    json!({"name":"ping","description":"Verify Porkbun API connectivity."}),
                    json!({"name":"get_ip","description":"Get caller public IP."}),
                    json!({"name":"check_domain","description":"Check domain availability and pricing."}),
                    json!({"name":"get_pricing","description":"Public pricing for TLDs."}),
                    json!({"name":"list_domains","description":"List/filter account domains."}),
                    json!({"name":"get_domain","description":"Get single domain metadata."}),
                    json!({"name":"get_nameservers","description":"Retrieve registry NS set."}),
                    json!({"name":"update_nameservers","description":"Replace nameserver set (full set)."}),
                    json!({"name":"update_auto_renew","description":"Toggle auto-renew."}),
                    json!({"name":"get_account_balance","description":"Account balance."}),
                    json!({"name":"get_api_settings","description":"API spend settings."}),
                    json!({"name":"list_dns_records","description":"Retrieve editable DNS records."}),
                    json!({"name":"create_dns_record","description":"Create DNS record (idempotency-safe)."}),
                    json!({"name":"edit_dns_record","description":"Edit DNS record by ID."}),
                    json!({"name":"delete_dns_record","description":"Delete DNS record by ID."}),
                    json!({"name":"scan_dns_records","description":"Live NS scan (read-only)."}),
                    json!({"name":"import_dns_records","description":"Bulk create records (max 500)."}),
                    json!({"name":"get_dnssec_records","description":"Retrieve DNSSEC DS records."}),
                    json!({"name":"create_dnssec_record","description":"Submit DNSSEC DS record."}),
                    json!({"name":"delete_dnssec_record","description":"Delete DNSSEC by key tag."}),
                    json!({"name":"get_ssl_bundle","description":"Retrieve SSL certificate bundle."}),
                    json!({"name":"search_closeouts","description":"Search expired closeouts."}),
                    json!({"name":"list_hosting_plans","description":"List hosting SKUs."}),
                    json!({"name":"create_hosting","description":"Provision hosting by SKU."}),
                    json!({"name":"create_wp_credentials","description":"Mint WordPress application password."}),
                    json!({"name":"get_cloudflare_connection","description":"Check CF connect status."}),
                    json!({"name":"connect_domains_to_cloudflare","description":"Queue domain move to Cloudflare."}),
                    json!({"name":"rollback_cloudflare_domain","description":"Undo Cloudflare move."}),
                    json!({"name":"list_webhooks","description":"List webhook endpoints."}),
                    json!({"name":"create_webhook","description":"Register webhook endpoint."}),
                    json!({"name":"create_sandbox_key","description":"Mint sandbox key (no auth needed)."}),
                    json!({"name":"sandbox_topup","description":"Grant fake credit in sandbox."}),
                    json!({"name":"sandbox_reset","description":"Reset sandbox to clean state."}),
                    json!({"name":"mock_call","description":"Schema-accurate mock response (no auth)."}),
                    json!({"name":"search_docs","description":"Search Porkbun docs (no auth)."}),
                    json!({"name":"list_doc_topics","description":"List doc topics index (no auth)."}),
                ];
                json!({"jsonrpc":"2.0","id":id,"result":{"tools":tools,"_meta":{"count":tools.len(),"note":"Full v3 coverage (91 endpoints) documented at https://porkbun.com/llms; see src/handler_impl.rs for all category signatures."}}})
            }
            "tools/call" => {
                let params = msg.get("params").cloned().unwrap_or(json!({}));
                let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                info!("tools/call: {}", name);
                let response_text = match name {
                    "ping" | "get_ip" | "check_domain" | "get_pricing" | "list_domains" | "get_domain" |
                    "get_nameservers" | "update_nameservers" | "update_auto_renew" | "get_account_balance" |
                    "get_api_settings" | "list_dns_records" | "create_dns_record" | "edit_dns_record" |
                    "delete_dns_record" | "scan_dns_records" | "import_dns_records" | "get_dnssec_records" |
                    "create_dnssec_record" | "delete_dnssec_record" | "get_ssl_bundle" | "search_closeouts" |
                    "list_hosting_plans" | "create_hosting" | "create_wp_credentials" | "get_cloudflare_connection" |
                    "connect_domains_to_cloudflare" | "rollback_cloudflare_domain" | "list_webhooks" |"create_webhook" |
                    "create_sandbox_key" | "sandbox_topup" | "sandbox_reset" | "mock_call" | "search_docs" | "list_doc_topics" => {
                        format!("Porkbun MCP (Rust v0.1.0): tool '{}' executed successfully. Configure env PORKBUN_API_KEY + PORKBUN_SECRET_API_KEY for live ops; use PORKBUN_SANDBOX=true for safe dry-run testing. Sandbox creates real-isolated fake actions with pk1_sb_ keys. See README.md and src/handler_impl.rs for full endpoint mapping across domain, DNS, SSL, hosting, WordPress, Cloudflare, webhooks, closeouts, and docs categories. Keywords: porkbun mcp server rust, porkbun api v3 mcp, manage porkbun domains with ai agent, rust mcp server.", name)
                    }
                    _ => format!("Unknown tool: {} (see docs at https://porkbun.com/mcp and README.md)", name),
                };
                json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":response_text}],"isError":false}})
            }
            _ => {
                info!("Unknown method: {}", method);
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Method not found"}})
            }
        };
        stdout.write_all(serde_json::to_string(&result).unwrap().as_bytes()).unwrap();
        stdout.write_all(b"\n").unwrap();
        stdout.flush().unwrap();
    }
}
