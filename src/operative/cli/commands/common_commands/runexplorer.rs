use crate::constructive::bitcoiny::batch_record::batch_record::BatchRecord;
use crate::constructive::entry::entry::entry::Entry;
use crate::inscriptive::archival_manager::archival_manager::ARCHIVAL_MANAGER;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
use crate::inscriptive::privileges_manager::privileges_manager::PRIVILEGES_MANAGER;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::operative::run_args::chain::Chain;
use crate::transmutative::key::{FromNostrKeyStr, ToNostrKeyStr};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Router,
};
use bitcoin::hashes::Hash;
use bitcoin::Txid;
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::Deserialize;
use serde_json::Value;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
struct ExplorerState {
    chain: Chain,
    archival: ARCHIVAL_MANAGER,
    registery: REGISTERY,
    privileges_manager: Option<PRIVILEGES_MANAGER>,
    flame_manager: FLAME_MANAGER,
}

/// Serves a small block-explorer-style UI for archived batches (requires archival mode).
pub async fn runexplorer_command(
    chain: Chain,
    port: u16,
    archival: Option<&ARCHIVAL_MANAGER>,
    registery: &REGISTERY,
    privileges_manager: Option<&PRIVILEGES_MANAGER>,
    _coin_manager: &COIN_MANAGER,
    flame_manager: &FLAME_MANAGER,
) {
    let Some(archival) = archival else {
        eprintln!(
            "{}",
            "runexplorer requires archival resource mode (cube started with `archival`).".yellow()
        );
        return;
    };

    let state = ExplorerState {
        chain,
        archival: Arc::clone(archival),
        registery: Arc::clone(registery),
        privileges_manager: privileges_manager.map(Arc::clone),
        flame_manager: Arc::clone(flame_manager),
    };

    let app = Router::new()
        .route("/", get(|| async { Redirect::to("/batches") }))
        .route("/batches", get(page_batches))
        .route("/accounts", get(page_accounts))
        .route("/contracts", get(page_contracts))
        .route("/search", get(search_batch))
        .route("/batch/height/:height", get(page_batch_by_height))
        .route("/batch/tx/:txid", get(page_batch_by_txid))
        .route("/entry/:entry_id", get(page_entry_by_id))
        .route("/account/:account_id", get(page_account_by_id))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            if e.kind() == ErrorKind::AddrInUse {
                eprintln!(
                    "{}",
                    format!(
                        "runexplorer: port {} is already in use ({}).\n\
                         If CUBE_EXPLORER_PORT={0} is set, the explorer already started automatically — skip `runexplorer {0}`.\n\
                         Otherwise stop whatever owns the port, unset CUBE_EXPLORER_PORT, or use e.g. `runexplorer 8081`.",
                        port, e
                    )
                    .yellow()
                );
            } else {
                eprintln!("{} {}", format!("runexplorer: failed to bind {}:", addr).yellow(), e);
            }
            return;
        }
    };

    println!(
        "{}",
        format!(
            "Batch explorer listening on http://127.0.0.1:{}/batches (bound on {})",
            port, addr
        )
        .green()
    );

    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
}

fn parse_entry_id_hex(hex_str: &str) -> Option<[u8; 32]> {
    let s = hex_str.trim().trim_start_matches("0x");
    if s.len() != 64 {
        return None;
    }
    let bytes = hex::decode(s).ok()?;
    bytes.try_into().ok()
}

fn parse_account_key(input: &str) -> Option<[u8; 32]> {
    let trimmed = input.trim();
    if let Some(key) = trimmed.from_npub() {
        return Some(key);
    }
    parse_entry_id_hex(trimmed)
}

fn account_url(account_key: [u8; 32]) -> String {
    format!("/account/{}", hex::encode(account_key))
}

fn account_link(account_key: [u8; 32]) -> String {
    let npub = account_key.to_npub().unwrap_or_else(|| "n/a".to_string());
    format!(
        r#"<a class="row-link" href="{}"><code class="mono">{}</code></a>"#,
        html_escape(&account_url(account_key)),
        html_escape(&npub)
    )
}

fn entry_involved_account_keys(entry: &Entry) -> Vec<[u8; 32]> {
    match entry {
        Entry::Move(move_entry) => {
            let from_key = move_entry.from.account_key();
            let to_key = move_entry.to.account_key();
            if from_key == to_key {
                vec![from_key]
            } else {
                vec![from_key, to_key]
            }
        }
        Entry::Liftup(liftup) => vec![liftup.root_account.account_key()],
        Entry::Call(call) => vec![call.account.account_key()],
    }
}

fn mempool_tx_url(chain: Chain, txid: &str) -> Option<String> {
    match chain {
        Chain::Mainnet => Some(format!("https://mempool.space/tx/{}", txid)),
        Chain::Signet => Some(format!("https://mempool.space/signet/tx/{}", txid)),
        Chain::Testbed => None,
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Renders `ts` as Unix seconds: RFC3339 `datetime` attribute and human-readable label only.
fn explorer_timestamp_html(ts: u64) -> String {
    let Ok(secs) = i64::try_from(ts) else {
        return r#"<span class="muted">—</span>"#.to_string();
    };
    match DateTime::<Utc>::from_timestamp(secs, 0) {
        Some(dt) => {
            let human = dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
            let iso = dt.to_rfc3339();
            format!(
                r#"<span class="mono"><time datetime="{}">{}</time></span>"#,
                html_escape(&iso),
                html_escape(&human)
            )
        }
        None => r#"<span class="muted">—</span>"#.to_string(),
    }
}

/// Relative time for the batches list column (`just now` / `N seconds ago` / …).
fn batch_table_relative_time_html(ts: u64) -> String {
    let Ok(secs_i) = i64::try_from(ts) else {
        return r#"<span class="muted">—</span>"#.to_string();
    };
    let Some(then) = DateTime::<Utc>::from_timestamp(secs_i, 0) else {
        return r#"<span class="muted">—</span>"#.to_string();
    };
    let now = Utc::now();
    let title_abs = then.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let iso = then.to_rfc3339();
    let diff = now.signed_duration_since(then);

    let label = if diff.num_seconds() < 0 {
        then.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else {
        let s = diff.num_seconds();
        if s < 10 {
            "just now".to_string()
        } else if s < 60 {
            if s == 1 {
                "1 second ago".to_string()
            } else {
                format!("{} seconds ago", s)
            }
        } else {
            let m = diff.num_minutes();
            if m < 60 {
                if m == 1 {
                    "1 minute ago".to_string()
                } else {
                    format!("{} minutes ago", m)
                }
            } else {
                let h = diff.num_hours();
                if h < 24 {
                    if h == 1 {
                        "1 hour ago".to_string()
                    } else {
                        format!("{} hours ago", h)
                    }
                } else {
                    let d = diff.num_days();
                    if d == 1 {
                        "1 day ago".to_string()
                    } else {
                        format!("{} days ago", d)
                    }
                }
            }
        }
    };

    format!(
        r#"<span class="mono"><time datetime="{}" title="{}">{}</time></span>"#,
        html_escape(&iso),
        html_escape(&title_abs),
        html_escape(&label)
    )
}

/// Header logo (GitHub avatar).
const EXPLORER_LOGO_URL: &str = "https://avatars.githubusercontent.com/u/173623209?s=200&v=4";

fn explorer_css() -> &'static str {
    r#"<style>
:root { color-scheme: light; }
html { font-size: 110%; }
body { font-family: ui-sans-serif, system-ui, sans-serif; background: #faf7ef; color: #1f2328; margin: 0; line-height: 1.55; min-height: 100vh; display: flex; flex-direction: column; }
.site-header { position: sticky; top: 0; z-index: 20; background: #fffdf8; border-bottom: 1px solid #e8e2d4; padding: 0.55rem 1.25rem; display: flex; justify-content: space-between; align-items: center; gap: 1rem 1.5rem; flex-wrap: wrap; box-shadow: 0 1px 0 rgba(0,0,0,0.04); }
.site-header .header-left { display: flex; align-items: center; flex-wrap: wrap; gap: 0.5rem 1.5rem; }
.site-header .brand { display: inline-flex; align-items: center; gap: 0.5rem; text-decoration: none; color: #1f2328; }
.site-header .brand:hover { color: #0550ae; }
.site-header .brand-logo { height: 38px; width: auto; display: block; object-fit: contain; }
.site-header .brand-text { font-weight: 700; font-size: 1.22rem; letter-spacing: 0.02em; padding-top: 0.18em; }
.site-header nav {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.08rem;
  padding: 0.15rem;
  background: linear-gradient(180deg, rgba(255,255,255,0.65) 0%, rgba(245,240,228,0.9) 100%);
  border: 1px solid #e4dcc8;
  border-radius: 9px;
  box-shadow: 0 1px 2px rgba(31,35,40,0.06);
}
.site-header nav a {
  color: #3d444d;
  text-decoration: none;
  font-size: 0.88rem;
  font-weight: 500;
  letter-spacing: 0.01em;
  padding: 0.3rem 0.65rem;
  border-radius: 7px;
  transition: background-color 0.16s ease, color 0.16s ease, box-shadow 0.16s ease;
}
.site-header nav a:hover {
  background: #fffefb;
  color: #0550ae;
  box-shadow: 0 1px 0 rgba(5,80,174,0.12);
}
.site-header nav a:focus-visible {
  outline: 2px solid #0550ae;
  outline-offset: 2px;
}
.site-header nav a:active {
  background: #efe9dc;
  color: #033d8a;
}
.search-form { display: flex; gap: 0.4rem; align-items: center; flex-shrink: 0; }
.search-form input { background: #fffefb; border: 1px solid #d8d0c0; border-radius: 6px; color: #1f2328; padding: 0.45rem 0.7rem; min-width: 16.5rem; font-family: ui-monospace, monospace; font-size: 0.85rem; }
.search-form input::placeholder { color: #6e7781; }
.search-form input:focus { outline: 2px solid #bfae88; outline-offset: 1px; border-color: #c4b896; }
.search-form button { background: #f0ebe0; border: 1px solid #d8d0c0; color: #1f2328; border-radius: 6px; padding: 0.45rem 0.85rem; cursor: pointer; font-size: 0.85rem; }
.search-form button:hover { background: #e8e2d4; }
main { flex: 1; padding: 1.5rem; max-width: 1100px; margin: 0 auto; width: 100%; box-sizing: border-box; }
.site-footer { background: #fffdf8; border-top: 1px solid #e8e2d4; padding: 1rem 1.5rem; text-align: center; }
.site-footer .footer-tagline { margin: 0 auto; color: #5c6670; font-size: 0.88rem; line-height: 1.55; max-width: 48rem; text-align: center; }
h1 { font-size: 1.52rem; font-weight: 650; margin: 0 0 1rem; color: #1f2328; }
table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
th, td { text-align: left; padding: 0.55rem 0.65rem; border-bottom: 1px solid #e8e2d4; }
th { color: #5c6670; font-weight: 600; }
td.mono, th.mono { font-family: ui-monospace, monospace; font-size: 0.82rem; }
th.num, td.num { text-align: right; font-variant-numeric: tabular-nums; }
a.row-link { color: #0550ae; }
a.row-link:hover { text-decoration: underline; }
.summary { display: grid; grid-template-columns: 9.5rem 1fr; gap: 0.35rem 1rem; margin-bottom: 1.5rem; font-size: 0.92rem; }
.summary dt { color: #5c6670; }
.summary dd { margin: 0; font-family: ui-monospace, monospace; word-break: break-all; }
.mono-wrap { display: inline-block; max-width: 100%; overflow-wrap: anywhere; word-break: break-all; white-space: normal; }
.collapsible { margin: 0; }
.collapsible > summary { cursor: pointer; color: #0550ae; user-select: none; }
.collapsible > summary:hover { text-decoration: underline; }
section.entries { margin-top: 2rem; }
.entry-card { background: #fffefb; border: 1px solid #e8e2d4; border-radius: 6px; padding: 1rem; margin-bottom: 1rem; box-shadow: 0 1px 2px rgba(0,0,0,0.03); }
.entry-card h3 { margin: 0 0 0.5rem; font-size: 0.95rem; color: #5c6670; }
pre { margin: 0; overflow-x: auto; font-size: 0.78rem; color: #24292f; }
pre.reg-json { font-size: 0.8rem; background: #fffefb; border: 1px solid #e8e2d4; border-radius: 6px; padding: 1rem; }
.badge { display: inline-block; padding: 0.15rem 0.45rem; border-radius: 4px; background: #efe9dc; font-size: 0.75rem; color: #5c6670; border: 1px solid #e0d8c8; }
.muted { color: #5c6670; }
.explorer-subsec { margin-top: 1.85rem; }
.explorer-subsec:first-of-type { margin-top: 1rem; }
.explorer-subsec h2 { font-size: 1.18rem; font-weight: 620; color: #1f2328; margin: 0 0 0.5rem; padding-bottom: 0.35rem; border-bottom: 1px solid #e8e2d4; }
.visually-hidden { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border: 0; }
.action-btn { display: inline-block; text-decoration: none; background: #f0ebe0; border: 1px solid #d8d0c0; color: #1f2328; border-radius: 7px; padding: 0.45rem 0.75rem; font-size: 0.86rem; }
.action-btn:hover { background: #e8e2d4; color: #0550ae; }
.account-hero { display: flex; align-items: flex-start; gap: 1rem; margin-bottom: 1rem; }
.account-avatar { width: 64px; height: 64px; border-radius: 999px; border: 1px solid #d8d0c0; background: radial-gradient(circle at 30% 30%, #fffefb 0%, #f0ebe0 70%, #e3dbca 100%); display: inline-flex; align-items: center; justify-content: center; color: #8b949e; font-size: 1.5rem; flex-shrink: 0; }
.account-hero-main { min-width: 0; }
.account-hero-main h1 { margin: 0 0 0.4rem; }
.account-head-row { display: flex; align-items: center; gap: 0.55rem; flex-wrap: wrap; margin: 0 0 0.5rem; }
.account-npub-title { font-size: 1.06rem; font-weight: 700; letter-spacing: 0.01em; font-family: ui-monospace, monospace; overflow-wrap: anywhere; }
</style>"#
}

fn site_header(search_value: &str) -> String {
    let q = html_escape(search_value);
    let logo_src = html_escape(EXPLORER_LOGO_URL);
    format!(
        r#"<header class="site-header" role="banner">
<div class="header-left">
<a class="brand" href="/batches">
<img class="brand-logo" src="{}" alt="" width="38" height="38" decoding="async" referrerpolicy="no-referrer">
<span class="brand-text">Cube Block Explorer</span>
</a>
<nav aria-label="Primary">
<a href="/batches">Batches</a>
<a href="/accounts">Accounts</a>
<a href="/contracts">Contracts</a>
</nav>
</div>
<form class="search-form" action="/search" method="get" role="search">
<label for="explorer-q" class="visually-hidden">Search batch by height</label>
<input id="explorer-q" type="text" name="q" placeholder="Search batch height..." value="{}" autocomplete="off">
<button type="submit">Search</button>
</form>
</header>"#,
        logo_src,
        q
    )
}

fn site_footer() -> &'static str {
    r#"<footer class="site-footer" role="contentinfo">
<p class="footer-tagline">© 2026 Black Box Labs Inc. All Rights Reserved.</p>
</footer>"#
}

fn layout(title: &str, body: &str, search_value: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{}</title>{}</head>
<body>{}<main>{}</main>{}</body></html>"#,
        html_escape(title),
        explorer_css(),
        site_header(search_value),
        body,
        site_footer()
    )
}

fn truncated_head_tail(value: &str, head: usize, tail: usize) -> String {
    if value.len() <= head + tail + 3 {
        return value.to_string();
    }
    format!("{}...{}", &value[..head], &value[value.len() - tail..])
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

async fn search_batch(
    State(st): State<ExplorerState>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let q_trim = params.q.unwrap_or_default().trim().to_string();
    if q_trim.is_empty() {
        return Redirect::to("/batches").into_response();
    }
    if let Ok(h) = q_trim.parse::<u64>() {
        return Redirect::to(&format!("/batch/height/{}", h)).into_response();
    }

    if let Ok(txid) = Txid::from_str(&q_trim) {
        return Redirect::to(&format!("/batch/tx/{}", txid)).into_response();
    }

    if let Some(key32) = parse_entry_id_hex(&q_trim) {
        let is_entry = {
            let m = st.archival.lock().await;
            m.entry_record_by_entry_id(&key32).is_some()
        };
        if is_entry {
            return Redirect::to(&format!("/entry/{}", hex::encode(key32))).into_response();
        }
        let is_account = {
            let r = st.registery.lock().await;
            r.get_account_body_by_account_key(key32).is_some()
        };
        if is_account {
            return Redirect::to(&account_url(key32)).into_response();
        }
    }

    if let Some(account_key) = q_trim.as_str().from_npub() {
        let is_account = {
            let r = st.registery.lock().await;
            r.get_account_body_by_account_key(account_key).is_some()
        };
        if is_account {
            return Redirect::to(&account_url(account_key)).into_response();
        }
    }

    (
        StatusCode::BAD_REQUEST,
        Html(layout(
            "Search — Cube explorer",
            r#"<h1>Invalid search</h1><p>Search by batch height, batch txid, entry id (32-byte hex), account key (32-byte hex), or npub.</p>"#,
            &q_trim,
        )),
    )
        .into_response()
}

async fn page_accounts(State(st): State<ExplorerState>) -> Html<String> {
    let mut rows: Vec<(u64, [u8; 32], u64, u64)> = {
        let g = st.registery.lock().await;
        let full = g.json();
        let mut parsed = Vec::new();
        if let Some(accounts_obj) = full.get("accounts").and_then(|v| v.as_object()) {
            for (account_key_hex, body) in accounts_obj {
                let Some(bytes_vec) = hex::decode(account_key_hex).ok() else {
                    continue;
                };
                let Ok(account_key): Result<[u8; 32], _> = bytes_vec.try_into() else {
                    continue;
                };
                let registery_index = body
                    .get("registery_index")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0);
                let call_counter = body
                    .get("call_counter")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0);
                let last_activity_timestamp = body
                    .get("last_activity_timestamp")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0);
                parsed.push((registery_index, account_key, call_counter, last_activity_timestamp));
            }
        }
        parsed
    };
    rows.sort_by_key(|(registery_index, _, _, _)| *registery_index);

    let mut table_rows = String::new();
    for (registery_index, account_key, call_counter, last_activity_timestamp) in rows {
        table_rows.push_str(&format!(
            r#"<tr><td class="num">{}</td><td>{}</td><td class="num">{}</td><td>{}</td></tr>"#,
            registery_index,
            account_link(account_key),
            call_counter,
            explorer_timestamp_html(last_activity_timestamp),
        ));
    }
    if table_rows.is_empty() {
        table_rows = r#"<tr><td colspan="5">No accounts in registery.</td></tr>"#.to_string();
    }

    let body = format!(
        r#"<h1>Accounts</h1>
<p class="muted">Search by account key hex or npub to open details.</p>
<table><thead><tr><th class="num">Index</th><th>Account</th><th class="num">Call counter</th><th>Last activity</th></tr></thead><tbody>{}</tbody></table>"#,
        table_rows
    );
    Html(layout("Accounts — Cube explorer", &body, ""))
}

async fn page_account_by_id(
    State(st): State<ExplorerState>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let Some(account_key) = parse_account_key(&account_id) else {
        return (
            StatusCode::BAD_REQUEST,
            Html(layout(
                "Account — Cube explorer",
                &format!(
                    r#"<h1>Invalid account id</h1><p>Expected 32-byte hex or npub; got <code class="mono">{}</code>.</p><p><a class="row-link" href="/accounts">← Accounts</a></p>"#,
                    html_escape(account_id.trim()),
                ),
                "",
            )),
        )
            .into_response();
    };

    let account_body = {
        let r = st.registery.lock().await;
        r.get_account_body_by_account_key(account_key)
    };
    let Some(account_body) = account_body else {
        return (
            StatusCode::NOT_FOUND,
            Html(layout(
                "Account — Cube explorer",
                &format!(
                    r#"<h1>Account not found</h1><p>No account found for <code class="mono">{}</code>.</p><p><a class="row-link" href="/accounts">← Accounts</a></p>"#,
                    html_escape(&account_id),
                ),
                "",
            )),
        )
            .into_response();
    };

    let privilege_body = match st.privileges_manager.as_ref() {
        Some(pm) => {
            let p = pm.lock().unwrap();
            p.get_account_body_by_account_key(account_key)
        }
        None => None,
    };
    let hierarchy = privilege_body
        .as_ref()
        .map(|b| b.hierarchy.clone())
        .unwrap_or(AccountHierarchy::Pleb);

    let history = {
        let a = st.archival.lock().await;
        a.retrieve_account_history(account_key)
    };
    let mut history_rows = String::new();
    for (batch_height, _batch_txid, batch_ts, entry_id, entry) in history.iter().rev() {
        let entry_id_hex = hex::encode(entry_id);
        let entry_kind = match entry {
            Entry::Move(_) => "Move",
            Entry::Call(_) => "Call",
            Entry::Liftup(_) => "Liftup",
        };
        history_rows.push_str(&format!(
            r#"<tr><td><a class="row-link" href="/entry/{0}"><code class="mono">{0}</code></a></td><td>{1}</td><td><a class="row-link" href="/batch/height/{2}">{2}</a></td><td>{3}</td></tr>"#,
            html_escape(&entry_id_hex),
            entry_kind,
            batch_height,
            explorer_timestamp_html(*batch_ts),
        ));
    }
    if history_rows.is_empty() {
        history_rows = r#"<tr><td colspan="4">No transaction history in archival records.</td></tr>"#.to_string();
    }

    let privileges_json = if let Some(pb) = privilege_body {
        serde_json::json!({
            "liveness_flag": pb.liveness_flag,
            "hierarchy": pb.hierarchy.to_string(),
            "txfee_exemptions": pb.txfee_exemptions,
            "can_deploy_liquidity": pb.can_deploy_liquidity,
            "can_deploy_contract": pb.can_deploy_contract,
        })
    } else {
        Value::Null
    };
    let privileges_pretty =
        serde_json::to_string_pretty(&privileges_json).unwrap_or_else(|_| "null".to_string());

    let vtxo_json = {
        let fm = st.flame_manager.lock().await;
        fm.get_account_flame_set(account_key)
            .and_then(|v| serde_json::to_value(v).ok())
            .unwrap_or(Value::Null)
    };
    let vtxo_pretty = serde_json::to_string_pretty(&vtxo_json).unwrap_or_else(|_| "null".to_string());

    let npub = account_key.to_npub().unwrap_or_else(|| "n/a".to_string());
    let npub_short = if npub.len() > 16 {
        format!("{}...{}", &npub[..8], &npub[npub.len() - 5..])
    } else {
        npub.clone()
    };
    let nostr_profile_url = if npub == "n/a" {
        "https://iris.to/".to_string()
    } else {
        format!("https://iris.to/{}", npub)
    };
    let account_hex = hex::encode(account_key);
    let account_body_json_pretty =
        serde_json::to_string_pretty(&account_body.json()).unwrap_or_else(|_| "{}".to_string());

    let body = format!(
        r#"<section class="account-hero">
<div class="account-avatar" aria-label="Profile placeholder">👻</div>
<div class="account-hero-main">
<h1>Account</h1>
<div class="account-head-row"><span class="account-npub-title">{}</span><span class="badge">{}</span></div>
<p style="margin:0 0 0.5rem"><a class="action-btn" href="{}" target="_blank" rel="noopener">View Nostr Profile ↗</a></p>
</div>
</section>
<dl class="summary">
<dt>Account key (hex)</dt><dd><code class="mono">{}</code></dd>
<dt>Account npub</dt><dd><code class="mono">{}</code></dd>
<dt>Registery index</dt><dd>{}</dd>
<dt>Last active</dt><dd>{}</dd>
<dt>Call counter</dt><dd>{}</dd>
</dl>
<section class="explorer-subsec">
<h2>Transaction History</h2>
<table><thead><tr><th>Entry ID</th><th>Kind</th><th>Batch</th><th>Timestamp</th></tr></thead><tbody>{}</tbody></table>
</section>
<section class="explorer-subsec">
<h2>Privileges</h2>
<pre class="reg-json">{}</pre>
</section>
<section class="explorer-subsec">
<h2>VTXO Set</h2>
<pre class="reg-json">{}</pre>
</section>
<section class="explorer-subsec">
<h2>Registery Account Body</h2>
<pre class="reg-json">{}</pre>
</section>
<p style="margin-top:1.25rem"><a class="row-link" href="/accounts">← Accounts</a></p>"#,
        html_escape(&npub_short),
        hierarchy.to_string(),
        html_escape(&nostr_profile_url),
        html_escape(&account_hex),
        html_escape(&npub),
        account_body.registery_index,
        explorer_timestamp_html(account_body.last_activity_timestamp),
        account_body.call_counter,
        history_rows,
        html_escape(&privileges_pretty),
        html_escape(&vtxo_pretty),
        html_escape(&account_body_json_pretty),
    );
    Html(layout("Account — Cube explorer", &body, &account_id)).into_response()
}

async fn page_contracts(State(st): State<ExplorerState>) -> Html<String> {
    let contracts_json = {
        let g = st.registery.lock().await;
        let full = g.json();
        full.get("contracts")
            .cloned()
            .unwrap_or(Value::Null)
    };
    let pretty =
        serde_json::to_string_pretty(&contracts_json).unwrap_or_else(|_| "{}".to_string());
    Html(layout(
        "Contracts — Cube explorer",
        &format!(
            r#"<h1>Contracts</h1><p class="muted" style="font-size:0.9rem">Registery <code>contracts</code> map (JSON).</p><pre class="reg-json">{}</pre>"#,
            html_escape(&pretty)
        ),
        "",
    ))
}

async fn page_batches(State(st): State<ExplorerState>) -> Html<String> {
    let rows: Vec<(u64, u64, String, usize)> = {
        let m = st.archival.lock().await;
        let mut v: Vec<_> = m
            .batch_records()
            .iter()
            .map(|r| {
                (
                    r.batch_height,
                    r.batch_timestamp,
                    r.batch_container.signed_batch_txn.txid().to_string(),
                    r.entries.len(),
                )
            })
            .collect();
        v.sort_by_key(|(h, _, _, _)| std::cmp::Reverse(*h));
        v
    };

    let mut table_rows = String::new();
    for (height, ts, txid, num_entries) in rows {
        let ts_html = batch_table_relative_time_html(ts);
        table_rows.push_str(&format!(
            r#"<tr><td><a class="row-link" href="/batch/height/{0}">{0}</a></td><td>{1}</td><td class="mono"><a class="row-link" href="/batch/tx/{3}">{2}</a></td><td class="num">{4}</td></tr>"#,
            height,
            ts_html,
            html_escape(&txid),
            html_escape(&txid),
            num_entries
        ));
    }

    let body = format!(
        r#"<h1><span class="badge">{}</span> Latest batches</h1>
<table><thead><tr><th>Height</th><th>Seen</th><th>Txid</th><th class="num">Number of Entries</th></tr></thead><tbody>{}</tbody></table>"#,
        st.chain.to_string(),
        if table_rows.is_empty() {
            r#"<tr><td colspan="4">No batches in archival storage yet.</td></tr>"#.to_string()
        } else {
            table_rows
        }
    );

    Html(layout("Batches — Cube explorer", &body, ""))
}

async fn page_batch_by_height(
    State(st): State<ExplorerState>,
    Path(height): Path<u64>,
) -> impl IntoResponse {
    let record = {
        let m = st.archival.lock().await;
        m.batch_record_by_height(height)
    };
    match record {
        Some(r) => Html(render_batch_page(st.chain, &r)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Html(layout(
                "Not found",
                &format!(r#"<h1>Batch not found</h1><p>No batch at height <code>{}</code>.</p><p><a class="row-link" href="/batches">← Batches</a></p>"#, height),
                "",
            )),
        )
            .into_response(),
    }
}

async fn page_batch_by_txid(
    State(st): State<ExplorerState>,
    Path(txid_hex): Path<String>,
) -> impl IntoResponse {
    let txid_bytes: Option<[u8; 32]> = Txid::from_str(&txid_hex)
        .ok()
        .map(|t| t.to_byte_array());
    let Some(txid_bytes) = txid_bytes else {
        return (
            StatusCode::BAD_REQUEST,
            Html(layout(
                "Bad txid",
                &format!(
                    r#"<h1>Invalid txid</h1><p><code>{}</code> is not a valid txid.</p>"#,
                    html_escape(&txid_hex)
                ),
                "",
            )),
        )
            .into_response();
    };

    let record = {
        let m = st.archival.lock().await;
        m.batch_record_by_txid(&txid_bytes)
    };
    match record {
        Some(r) => Html(render_batch_page(st.chain, &r)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Html(layout(
                "Not found",
                &format!(
                    r#"<h1>Batch not found</h1><p>No batch with txid <code class="mono">{}</code>.</p><p><a class="row-link" href="/batches">← Batches</a></p>"#,
                    html_escape(&txid_hex)
                ),
                "",
            )),
        )
            .into_response(),
    }
}

async fn page_entry_by_id(
    State(st): State<ExplorerState>,
    Path(entry_hex): Path<String>,
) -> impl IntoResponse {
    let Some(entry_id) = parse_entry_id_hex(&entry_hex) else {
        return (
            StatusCode::BAD_REQUEST,
            Html(layout(
                "Entry — Cube explorer",
                &format!(
                    r#"<h1>Invalid entry id</h1><p>Expected 64 hex characters (32 bytes); got <code class="mono">{}</code>.</p><p><a class="row-link" href="/batches">← Batches</a></p>"#,
                    html_escape(entry_hex.trim())
                ),
                "",
            )),
        )
            .into_response();
    };

    let resolved = {
        let m = st.archival.lock().await;
        m.entry_record_by_entry_id(&entry_id)
    };

    let Some((batch_height, batch_txid_bytes, batch_ts, _eid, entry, collected_bits, fees)) = resolved else {
        return (
            StatusCode::NOT_FOUND,
            Html(layout(
                "Entry — Cube explorer",
                &format!(
                    r#"<h1>Entry not found</h1><p>No entry with id <code class="mono">{}</code> in archival history.</p><p><a class="row-link" href="/batches">← Batches</a></p>"#,
                    html_escape(&hex::encode(entry_id))
                ),
                "",
            )),
        )
            .into_response();
    };

    let eid_hex = hex::encode(entry_id);
    let batch_txid = Txid::from_byte_array(batch_txid_bytes).to_string();
    let ts_html = explorer_timestamp_html(batch_ts);
    let batch_txid_dd = format!(
        r#"<a class="row-link" href="/batch/tx/{0}"><code class="mono">{1}</code></a>"#,
        html_escape(&batch_txid),
        html_escape(&batch_txid),
    );
    let entry_json = serde_json::to_string_pretty(&entry.json()).unwrap_or_else(|_| "{}".to_string());
    let entry_fees_json =
        serde_json::to_string_pretty(&fees.map(|v| v.json()).unwrap_or(serde_json::Value::Null))
            .unwrap_or_else(|_| "null".to_string());
    let entry_kind_title = match &entry {
        Entry::Liftup(_) => "🛗 Liftup",
        Entry::Move(_) => "💰 Move",
        Entry::Call(_) => "📞 Call",
    };
    let involved_accounts = entry_involved_account_keys(&entry)
        .into_iter()
        .map(|account_key| {
            let npub = account_key.to_npub().unwrap_or_else(|| "n/a".to_string());
            format!(
                r#"<a class="row-link" href="{}"><code class="mono">{}</code></a>"#,
                html_escape(&account_url(account_key)),
                html_escape(&npub)
            )
        })
        .collect::<Vec<_>>()
        .join("<br/>");
    let collected_bits_html = match collected_bits {
        Some(bits) => {
            let bits_preview = truncated_head_tail(&bits, 28, 16);
            format!(
                r#"<details class="collapsible"><summary><code class="mono mono-wrap">{}</code></summary><code class="mono mono-wrap">{}</code></details>"#,
                html_escape(&bits_preview),
                html_escape(&bits)
            )
        }
        None => "N/A (non-archival record)".to_string(),
    };

    let body = format!(
        r#"<h1>{}</h1>
<dl class="summary">
<dt>Entry id</dt><dd><code class="mono">{}</code></dd>
<dt>Batch height</dt><dd><a class="row-link" href="/batch/height/{}">{}</a></dd>
<dt>Batch txid</dt><dd>{}</dd>
<dt>Batch timestamp</dt><dd>{}</dd>
<dt>APE bitstream</dt><dd>{}</dd>
<dt>Involved account(s)</dt><dd>{}</dd>
</dl>
<h2 style="font-size:1.1rem;margin:1.25rem 0 0.65rem">Entry fees</h2>
<pre class="reg-json">{}</pre>
<h2 style="font-size:1.1rem;margin:1.25rem 0 0.65rem">Entry data</h2>
<pre class="reg-json">{}</pre>
<p style="margin-top:1.25rem"><a class="row-link" href="/batch/height/{}">← Batch #{}</a> · <a class="row-link" href="/batches">All batches</a></p>"#,
        entry_kind_title,
        html_escape(&eid_hex),
        batch_height,
        batch_height,
        batch_txid_dd,
        ts_html,
        collected_bits_html,
        involved_accounts,
        html_escape(&entry_fees_json),
        html_escape(&entry_json),
        batch_height,
        batch_height,
    );

    Html(layout(
        &format!("{} — {}", entry_kind_title, &eid_hex),
        &body,
        "",
    ))
    .into_response()
}

fn render_batch_page(chain: Chain, record: &BatchRecord) -> String {
    let ts_html = explorer_timestamp_html(record.batch_timestamp);
    let txid = record.batch_container.signed_batch_txn.txid().to_string();
    let txid_cell = match mempool_tx_url(chain, &txid) {
        Some(url) => format!(
            r#"<a class="row-link" href="{}" target="_blank" rel="noopener"><code class="mono">{}</code></a>"#,
            html_escape(&url),
            html_escape(&txid)
        ),
        None => format!(r#"<code class="mono">{}</code>"#, html_escape(&txid)),
    };

    let mut entries_html = String::new();
    for (i, (entry_id, entry)) in record.entries.iter().enumerate() {
        let eid = hex::encode(entry_id);
        let eid_short = format!("{}…", &eid[..24]);
        let json_pretty = serde_json::to_string_pretty(&entry.json()).unwrap_or_else(|_| "{}".into());
        let fees_pretty = serde_json::to_string_pretty(
            &record
                .entry_fees
                .get(i)
                .map(|f| f.json())
                .unwrap_or(serde_json::Value::Null),
        )
        .unwrap_or_else(|_| "null".to_string());
        entries_html.push_str(&format!(
            r#"<div class="entry-card" id="entry-{1}"><h3>Entry {0} — <a class="row-link" href="/entry/{1}" title="{2}"><code class="mono">{3}</code></a></h3><p class="muted" style="margin-bottom:0.35rem"><strong>Fees</strong></p><pre>{4}</pre><p class="muted" style="margin-top:0.75rem;margin-bottom:0.35rem"><strong>Entry data</strong></p><pre>{5}</pre></div>"#,
            i + 1,
            eid,
            html_escape(&eid),
            html_escape(&eid_short),
            html_escape(&fees_pretty),
            html_escape(&json_pretty)
        ));
    }

    if entries_html.is_empty() {
        entries_html = r#"<p class="muted">No entries in this batch.</p>"#.into();
    }

    let body = format!(
        r#"<h1>📦 Batch #{}</h1>
<dl class="summary">
<dt>Height</dt><dd>{}</dd>
<dt>Timestamp</dt><dd>{}</dd>
<dt>Txid</dt><dd>{}</dd>
<dt>Payload version</dt><dd>{}</dd>
<dt>Entries</dt><dd>{}</dd>
<dt>BLS agg sig</dt><dd>{}</dd>
</dl>
<section class="entries"><h2 style="font-size:1.1rem;margin-bottom:0.75rem">Entries</h2>{}</section>
<p><a class="row-link" href="/batches">← All batches</a></p>"#,
        record.batch_height,
        record.batch_height,
        ts_html,
        txid_cell,
        record.payload_version,
        record.entries.len(),
        {
            let bls_full = hex::encode(&record.aggregate_bls_signature);
            let bls_preview = truncated_head_tail(&bls_full, 28, 16);
            format!(
                r#"<details class="collapsible"><summary><code class="mono mono-wrap">{}</code></summary><code class="mono mono-wrap">{}</code></details>"#,
                html_escape(&bls_preview),
                html_escape(&bls_full)
            )
        },
        entries_html
    );

    layout(
        &format!("Batch {} — Cube explorer", record.batch_height),
        &body,
        "",
    )
}
