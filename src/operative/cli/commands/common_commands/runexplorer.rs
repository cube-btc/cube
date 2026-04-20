use crate::constructive::bitcoiny::batch_record::batch_record::BatchRecord;
use crate::inscriptive::archival_manager::archival_manager::ARCHIVAL_MANAGER;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::operative::run_args::chain::Chain;
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
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
struct ExplorerState {
    chain: Chain,
    archival: ARCHIVAL_MANAGER,
    registery: REGISTERY,
    coin_manager: COIN_MANAGER,
    flame_manager: FLAME_MANAGER,
}

/// Serves a small block-explorer-style UI for archived batches (requires archival mode).
pub async fn runexplorer_command(
    chain: Chain,
    port: u16,
    archival: Option<&ARCHIVAL_MANAGER>,
    registery: &REGISTERY,
    coin_manager: &COIN_MANAGER,
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
        coin_manager: Arc::clone(coin_manager),
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
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{} {}", format!("runexplorer: failed to bind {}:", addr).yellow(), e);
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
h1 { font-size: 1.35rem; font-weight: 600; margin: 0 0 1rem; color: #1f2328; }
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
section.entries { margin-top: 2rem; }
.entry-card { background: #fffefb; border: 1px solid #e8e2d4; border-radius: 6px; padding: 1rem; margin-bottom: 1rem; box-shadow: 0 1px 2px rgba(0,0,0,0.03); }
.entry-card h3 { margin: 0 0 0.5rem; font-size: 0.95rem; color: #5c6670; }
pre { margin: 0; overflow-x: auto; font-size: 0.78rem; color: #24292f; }
pre.reg-json { font-size: 0.8rem; background: #fffefb; border: 1px solid #e8e2d4; border-radius: 6px; padding: 1rem; }
.badge { display: inline-block; padding: 0.15rem 0.45rem; border-radius: 4px; background: #efe9dc; font-size: 0.75rem; color: #5c6670; border: 1px solid #e0d8c8; }
.muted { color: #5c6670; }
.explorer-subsec { margin-top: 1.85rem; }
.explorer-subsec:first-of-type { margin-top: 1rem; }
.explorer-subsec h2 { font-size: 1.1rem; font-weight: 600; color: #1f2328; margin: 0 0 0.5rem; padding-bottom: 0.35rem; border-bottom: 1px solid #e8e2d4; }
.visually-hidden { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border: 0; }
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

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

async fn search_batch(Query(params): Query<SearchParams>) -> impl IntoResponse {
    let q_trim = params.q.unwrap_or_default().trim().to_string();
    if q_trim.is_empty() {
        return Redirect::to("/batches").into_response();
    }
    match q_trim.parse::<u64>() {
        Ok(h) => Redirect::to(&format!("/batch/height/{}", h)).into_response(),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Html(layout(
                "Search — Cube explorer",
                r#"<h1>Invalid search</h1><p>Enter a batch height (non-negative integer).</p>"#,
                &q_trim,
            )),
        )
            .into_response(),
    }
}

async fn page_accounts(State(st): State<ExplorerState>) -> Html<String> {
    let reg_pretty = {
        let g = st.registery.lock().await;
        let full = g.json();
        let accounts_json = full
            .get("accounts")
            .cloned()
            .unwrap_or(Value::Null);
        serde_json::to_string_pretty(&accounts_json).unwrap_or_else(|_| "{}".to_string())
    };
    let coins_pretty = {
        let g = st.coin_manager.lock().await;
        serde_json::to_string_pretty(&g.json()).unwrap_or_else(|_| "{}".to_string())
    };
    let vtxo_pretty = {
        let g = st.flame_manager.lock().await;
        serde_json::to_string_pretty(&g.json()).unwrap_or_else(|_| "{}".to_string())
    };
    let body = format!(
        r#"<h1>Accounts</h1>
<section class="explorer-subsec">
<h2>Registery</h2>
<p class="muted" style="font-size:0.9rem"><code>Registery::json()</code> — <code>accounts</code> map.</p>
<pre class="reg-json">{}</pre>
</section>
<section class="explorer-subsec">
<h2>Coins</h2>
<p class="muted" style="font-size:0.9rem"><code>CoinManager::json()</code>.</p>
<pre class="reg-json">{}</pre>
</section>
<section class="explorer-subsec">
<h2>VTXO set</h2>
<p class="muted" style="font-size:0.9rem"><code>FlameManager::json()</code>.</p>
<pre class="reg-json">{}</pre>
</section>"#,
        html_escape(&reg_pretty),
        html_escape(&coins_pretty),
        html_escape(&vtxo_pretty),
    );
    Html(layout("Accounts — Cube explorer", &body, ""))
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

    let Some((batch_height, batch_txid_bytes, batch_ts, _eid, entry)) = resolved else {
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

    let body = format!(
        r#"<h1>Entry</h1>
<dl class="summary">
<dt>Entry id</dt><dd><code class="mono">{}</code></dd>
<dt>Batch height</dt><dd><a class="row-link" href="/batch/height/{1}">{1}</a></dd>
<dt>Batch txid</dt><dd>{2}</dd>
<dt>Batch timestamp</dt><dd>{3}</dd>
</dl>
<h2 style="font-size:1.1rem;margin:1.25rem 0 0.65rem">Entry data</h2>
<pre class="reg-json">{4}</pre>
<p style="margin-top:1.25rem"><a class="row-link" href="/batch/height/{1}">← Batch #{1}</a> · <a class="row-link" href="/batches">All batches</a></p>"#,
        html_escape(&eid_hex),
        batch_height,
        batch_txid_dd,
        ts_html,
        html_escape(&entry_json),
    );

    Html(layout(
        &format!("Entry — {}", &eid_hex),
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
        let json_pretty = serde_json::to_string_pretty(&entry.json()).unwrap_or_else(|_| "{}".into());
        entries_html.push_str(&format!(
            r#"<div class="entry-card" id="entry-{1}"><h3>Entry {0} — <a class="row-link" href="/entry/{1}"><code class="mono">{2}</code></a></h3><pre>{3}</pre></div>"#,
            i + 1,
            eid,
            html_escape(&eid),
            html_escape(&json_pretty)
        ));
    }

    if entries_html.is_empty() {
        entries_html = r#"<p class="muted">No entries in this batch.</p>"#.into();
    }

    let body = format!(
        r#"<h1><span class="badge">{}</span> Batch #{}</h1>
<dl class="summary">
<dt>Height</dt><dd>{}</dd>
<dt>Timestamp</dt><dd>{}</dd>
<dt>Txid</dt><dd>{}</dd>
<dt>Payload version</dt><dd>{}</dd>
<dt>Entries</dt><dd>{}</dd>
<dt>BLS agg sig</dt><dd><code class="mono">{}…</code></dd>
</dl>
<section class="entries"><h2 style="font-size:1.1rem;margin-bottom:0.75rem">Entries</h2>{}</section>
<p><a class="row-link" href="/batches">← All batches</a></p>"#,
        chain.to_string(),
        record.batch_height,
        record.batch_height,
        ts_html,
        txid_cell,
        record.payload_version,
        record.entries.len(),
        html_escape(&hex::encode(&record.aggregate_bls_signature[..24])),
        entries_html
    );

    layout(
        &format!("Batch {} — Cube explorer", record.batch_height),
        &body,
        "",
    )
}
