use crate::constructive::bitcoiny::batch_record::batch_record::BatchRecord;
use crate::constructive::entry::entry::entry::Entry;
use crate::inscriptive::archival_manager::archival_manager::ARCHIVAL_MANAGER;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;
use crate::inscriptive::privileges_manager::elements::exemption::periodic_resource::periodic_resource::PeriodicResource;
use crate::inscriptive::privileges_manager::privileges_manager::PRIVILEGES_MANAGER;
use crate::inscriptive::registry::registry::REGISTRY;
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
use serde_json::{Map, Value};
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
struct ExplorerState {
    chain: Chain,
    archival: ARCHIVAL_MANAGER,
    registry: REGISTRY,
    privileges_manager: Option<PRIVILEGES_MANAGER>,
    coin_manager: COIN_MANAGER,
    flame_manager: FLAME_MANAGER,
}

/// Serves a small block-explorer-style UI for archived batches (requires archival mode).
pub async fn runexplorer_command(
    chain: Chain,
    port: u16,
    archival: Option<&ARCHIVAL_MANAGER>,
    registry: &REGISTRY,
    privileges_manager: Option<&PRIVILEGES_MANAGER>,
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
        registry: Arc::clone(registry),
        privileges_manager: privileges_manager.map(Arc::clone),
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
        .route("/account/:account_id/:section", get(page_account_section))
        .route("/account/:account_id", get(page_account_root_redirect))
        .route("/contract/:contract_id/:section", get(page_contract_section))
        .route("/contract/:contract_id", get(page_contract_root_redirect))
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
    format!("/account/{}/history", hex::encode(account_key))
}

fn contract_url(contract_id: [u8; 32]) -> String {
    format!("/contract/{}/registry", hex::encode(contract_id))
}

/// Account explorer subpages under `/account/:account_id/:section`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AccountExplorerSection {
    History,
    Registry,
    Vip,
    Privileges,
    Vtxo,
    CoinManager,
}

/// Contract explorer subpages under `/contract/:contract_id/:section`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ContractExplorerSection {
    Registry,
    Privileges,
    CoinManager,
}

impl ContractExplorerSection {
    const ALL: [Self; 3] = [Self::Registry, Self::Privileges, Self::CoinManager];

    fn from_slug(s: &str) -> Option<Self> {
        match s.trim() {
            "registry" => Some(Self::Registry),
            "privileges" => Some(Self::Privileges),
            "coin-manager" => Some(Self::CoinManager),
            _ => None,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Registry => "registry",
            Self::Privileges => "privileges",
            Self::CoinManager => "coin-manager",
        }
    }

    fn nav_label(self) -> &'static str {
        match self {
            Self::Registry => "Registry",
            Self::Privileges => "Privileges",
            Self::CoinManager => "Coin Manager",
        }
    }

    fn document_title(self) -> &'static str {
        match self {
            Self::Registry => "Registry",
            Self::Privileges => "Privileges",
            Self::CoinManager => "Coin manager",
        }
    }
}

fn contract_explorer_nav(contract_id: &str, current: ContractExplorerSection) -> String {
    let mut parts = Vec::with_capacity(ContractExplorerSection::ALL.len());
    for s in ContractExplorerSection::ALL {
        let active = s == current;
        let cls = if active {
            r#" class="tab-btn active""#
        } else {
            r#" class="tab-btn""#
        };
        let aria = if active {
            r#" aria-current="page""#
        } else {
            ""
        };
        parts.push(format!(
            r#"<a{}{} href="/contract/{}/{}">{}</a>"#,
            cls,
            aria,
            contract_id,
            s.slug(),
            html_escape(s.nav_label()),
        ));
    }
    format!(
        r#"<nav class="tab-menu" aria-label="Contract sections">{}</nav>"#,
        parts.join("\n")
    )
}

impl AccountExplorerSection {
    const ALL: [Self; 6] = [
        Self::History,
        Self::Registry,
        Self::Vip,
        Self::Privileges,
        Self::Vtxo,
        Self::CoinManager,
    ];

    fn from_slug(s: &str) -> Option<Self> {
        match s.trim() {
            "history" => Some(Self::History),
            "registry" => Some(Self::Registry),
            "vip" => Some(Self::Vip),
            "privileges" => Some(Self::Privileges),
            "vtxo" => Some(Self::Vtxo),
            "coin-manager" => Some(Self::CoinManager),
            _ => None,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::History => "history",
            Self::Registry => "registry",
            Self::Vip => "vip",
            Self::Privileges => "privileges",
            Self::Vtxo => "vtxo",
            Self::CoinManager => "coin-manager",
        }
    }

    fn nav_label(self) -> &'static str {
        match self {
            Self::History => "Transaction History",
            Self::Registry => "Registry",
            Self::Vip => "V.I.P.",
            Self::Privileges => "Privileges",
            Self::Vtxo => "VTXO Set",
            Self::CoinManager => "Coin Manager",
        }
    }

    fn document_title(self) -> &'static str {
        match self {
            Self::History => "History",
            Self::Registry => "Registry",
            Self::Vip => "V.I.P.",
            Self::Privileges => "Privileges",
            Self::Vtxo => "VTXO set",
            Self::CoinManager => "Coin manager",
        }
    }
}

fn account_explorer_nav(account_id: &str, current: AccountExplorerSection) -> String {
    let mut parts = Vec::with_capacity(AccountExplorerSection::ALL.len());
    for s in AccountExplorerSection::ALL {
        let active = s == current;
        let cls = if active {
            r#" class="tab-btn active""#
        } else {
            r#" class="tab-btn""#
        };
        let aria = if active {
            r#" aria-current="page""#
        } else {
            ""
        };
        parts.push(format!(
            r#"<a{}{} href="/account/{}/{}">{}</a>"#,
            cls,
            aria,
            account_id,
            s.slug(),
            html_escape(s.nav_label()),
        ));
    }
    format!(
        r#"<nav class="tab-menu" aria-label="Account sections">{}</nav>"#,
        parts.join("\n")
    )
}

/// Inline script for the V.I.P. periodic meter (only included on the V.I.P. subpage).
fn explorer_vip_periodic_meter_script() -> &'static str {
    r#"<script>
(function () {
  function cubeVipCurrentLeftSec(period, limit, latestLeft, latestActivityTs, nowTs) {
    const periodB = BigInt(period);
    const limitB = BigInt(limit);
    let latestB = BigInt(latestLeft);
    const activityB = BigInt(latestActivityTs);
    const nowB = BigInt(nowTs);
    if (activityB > nowB) {
      return latestB < limitB ? latestB : limitB;
    }
    const timePassed = nowB - activityB;
    if (periodB === 0n) {
      return latestB < limitB ? latestB : limitB;
    }
    if (timePassed >= periodB) {
      return limitB;
    }
    const refill = (timePassed * limitB) / periodB;
    let newAmt = latestB + refill;
    if (newAmt > limitB) newAmt = limitB;
    return newAmt;
  }

  function cubeFormatCommaU64FromBigint(n) {
    const s = n.toString();
    return s.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
  }

  function refreshVipPeriodicMeter() {
    const root = document.querySelector('[data-vip-meter]');
    if (!root) return;
    const period = root.dataset.period;
    const limit = root.dataset.limit;
    const latestLeft = root.dataset.latestLeft;
    const latestActivityTs = root.dataset.latestActivityTs;
    if (period === undefined || limit === undefined || latestLeft === undefined || latestActivityTs === undefined) return;
    const nowSec = Math.floor(Date.now() / 1000);
    const cur = cubeVipCurrentLeftSec(period, limit, latestLeft, latestActivityTs, nowSec);
    const limitB = BigInt(limit);
    const fill = root.querySelector('[data-vip-fill]');
    const track = root.querySelector('[data-vip-track]');
    const cap = root.querySelector('[data-vip-cur-left]');
    let fillPct = 0;
    if (limitB > 0n) {
      fillPct = Math.min(100, Number((cur * 10000n) / limitB) / 100);
    }
    if (fill) fill.style.width = fillPct.toFixed(4) + '%';
    if (track) track.setAttribute('aria-valuenow', String(Math.round(fillPct)));
    if (cap) cap.textContent = '¢ ' + cubeFormatCommaU64FromBigint(cur);
  }

  refreshVipPeriodicMeter();
  setInterval(refreshVipPeriodicMeter, 15000);
})();
</script>"#
}

fn account_link(account_key: [u8; 32]) -> String {
    let npub = account_key.to_npub().unwrap_or_else(|| "n/a".to_string());
    format!(
        r#"<a class="row-link" href="{}"><code class="mono">{}</code></a>"#,
        html_escape(&account_url(account_key)),
        html_escape(&npub)
    )
}

fn account_link_npub_truncated(account_key: [u8; 32]) -> String {
    let npub = account_key.to_npub().unwrap_or_else(|| "n/a".to_string());
    let label = if npub.len() > 16 {
        format!("{}...{}", &npub[..8], &npub[npub.len() - 5..])
    } else {
        npub.clone()
    };
    format!(
        r#"<a class="row-link" href="{}"><code class="mono">{}</code></a>"#,
        html_escape(&account_url(account_key)),
        html_escape(&label)
    )
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
.entries-table {
  border-collapse: separate;
  border-spacing: 0 0.48rem;
}
.entries-table thead th {
  border-bottom: 0;
  padding-bottom: 0.25rem;
}
.entries-table .entry-row-btn {
  cursor: pointer;
}
.entries-table .entry-row-btn td {
  background: linear-gradient(180deg, #fffefc 0%, #fbf7ee 100%);
  border-bottom: 0;
  border-top: 1px solid #efe6d6;
  border-bottom: 1px solid #e8dec9;
  box-shadow: 0 1px 3px rgba(31, 35, 40, 0.05), inset 0 1px 0 rgba(255,255,255,0.6);
  transition: box-shadow 0.14s ease, border-color 0.14s ease, background 0.14s ease;
}
.entries-table .entry-row-btn td:first-child {
  border-left: 1px solid #eadfca;
  border-top-left-radius: 10px;
  border-bottom-left-radius: 10px;
}
.entries-table .entry-row-btn td:last-child {
  border-right: 1px solid #eadfca;
  border-top-right-radius: 10px;
  border-bottom-right-radius: 10px;
}
.entries-table .entry-row-btn:hover td {
  background: linear-gradient(180deg, #fffefc 0%, #f8f2e6 100%);
  border-top-color: #e7dbc4;
  border-bottom-color: #e0d1b3;
  box-shadow: 0 2px 6px rgba(31, 35, 40, 0.07), inset 0 1px 0 rgba(255,255,255,0.7);
}
.entries-table .entry-row-btn:active td {
  box-shadow: 0 1px 4px rgba(31, 35, 40, 0.06), inset 0 1px 0 rgba(255,255,255,0.65);
}
.entries-table .entry-row-btn:focus-visible td {
  outline: 2px solid #bfae88;
  outline-offset: -2px;
  background: linear-gradient(180deg, #fffefc 0%, #f6efdf 100%);
}
a.row-link { color: #0550ae; }
a.row-link:hover { text-decoration: underline; }
.summary { display: grid; grid-template-columns: 9.5rem 1fr; gap: 0.35rem 1rem; margin-bottom: 1.5rem; font-size: 0.92rem; }
.summary dt { color: #5c6670; }
.summary dd { margin: 0; font-family: ui-monospace, monospace; word-break: break-all; }
.summary-copy-row { display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem; flex-wrap: wrap; }
.mono-wrap { display: inline-block; max-width: 100%; overflow-wrap: anywhere; word-break: break-all; white-space: normal; }
.expandable-mono { display: inline-flex; align-items: center; gap: 0.45rem; flex-wrap: wrap; }
.expandable-mono .mono { margin: 0; }
.expandable-mono-btn { background: #f0ebe0; border: 1px solid #d8d0c0; color: #1f2328; border-radius: 6px; padding: 0.2rem 0.5rem; cursor: pointer; font-size: 0.75rem; line-height: 1.2; }
.expandable-mono-btn:hover { background: #e8e2d4; color: #0550ae; }
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
.tab-menu { display: flex; gap: 0.45rem; flex-wrap: wrap; align-items: center; margin: 1rem 0 0.9rem; }
.tab-btn {
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  margin: 0;
  background: #f0ebe0;
  border: 1px solid #d8d0c0;
  color: #1f2328;
  border-radius: 8px;
  padding: 0.42rem 0.72rem;
  min-height: 2.05rem;
  cursor: pointer;
  font-size: 0.85rem;
  font-weight: 400;
  line-height: 1.2;
  text-align: center;
  text-decoration: none;
  -webkit-tap-highlight-color: transparent;
}
.tab-btn:hover { background: #e8e2d4; }
.tab-btn.active { background: #fffefb; border-color: #bfae88; color: #0550ae; }
.tab-btn:focus-visible { outline: 2px solid #bfae88; outline-offset: 2px; }
.account-section-page { margin-top: 0.2rem; }
.account-section-page > h2 { font-size: 1.18rem; font-weight: 620; color: #1f2328; margin: 0 0 0.55rem; padding-bottom: 0.35rem; border-bottom: 1px solid #e8e2d4; }
.account-subpage-footer { margin-top: 1.25rem; }
.account-card-body:has(#account-section-vip) {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  padding-bottom: 0.55rem;
}
.account-card-body:has(#account-section-vip) .account-subpage-footer {
  margin-top: 0.65rem;
  padding-top: 0;
}
#account-section-vip {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  text-align: center;
  margin-top: 0.2rem;
  padding: 0 0.75rem 0.35rem;
  box-sizing: border-box;
}
#account-section-vip::before {
  content: "";
  display: block;
  flex-shrink: 0;
  width: 100%;
  align-self: stretch;
  box-sizing: border-box;
  margin: 0 0 0.55rem;
  height: calc(1.18rem * 1.22 + 0.35rem + 1px);
  border-bottom: 1px solid #e8e2d4;
  pointer-events: none;
}
#account-section-vip > p.muted {
  max-width: 28rem;
  margin-left: auto;
  margin-right: auto;
}
#account-section-vip .vip-tier-root {
  width: 100%;
  max-width: 24rem;
  margin-top: 0;
  margin-left: auto;
  margin-right: auto;
}
#account-section-vip .vip-meter-block {
  width: min(100%, 24rem);
  margin-left: auto;
  margin-right: auto;
}
#account-section-vip .vip-meter-row {
  justify-content: center;
  flex-wrap: wrap;
}
#account-section-vip .vip-meter-track {
  flex: 1 1 9rem;
  min-width: 7rem;
  max-width: 16rem;
}
#account-section-vip .vip-stat-line {
  justify-content: center;
  max-width: 26rem;
  width: 100%;
  margin-left: auto;
  margin-right: auto;
}
.visually-hidden { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border: 0; }
.action-btn { display: inline-block; text-decoration: none; background: #f0ebe0; border: 1px solid #d8d0c0; color: #1f2328; border-radius: 7px; padding: 0.45rem 0.75rem; font-size: 0.86rem; }
.action-btn:hover { background: #e8e2d4; color: #0550ae; }
.copy-btn { background: #f0ebe0; border: 1px solid #d8d0c0; color: #1f2328; border-radius: 6px; padding: 0.3rem 0.58rem; cursor: pointer; font-size: 0.9rem; line-height: 1; white-space: nowrap; }
.copy-btn:hover { background: #e8e2d4; color: #0550ae; }
.account-header-row { display: flex; justify-content: space-between; align-items: flex-start; gap: 0.8rem; margin-bottom: 1rem; }
.account-header-main { display: flex; align-items: flex-start; gap: 1rem; min-width: 0; }
.account-avatar { width: 64px; height: 64px; border-radius: 999px; border: 1px solid #d8d0c0; background: radial-gradient(circle at 30% 30%, #fffefb 0%, #f0ebe0 70%, #e3dbca 100%); display: inline-flex; align-items: center; justify-content: center; color: #8b949e; font-size: 1.5rem; flex-shrink: 0; }
.vip-tier-root { max-width: 22rem; margin-top: 0.35rem; }
.vip-card { width: 11.2rem; margin: 0 auto 1.1rem; border-radius: 12px; padding: 0.85rem 0.75rem 0.75rem; text-align: center; border: 1px solid rgba(0,0,0,0.12); box-shadow: 0 4px 14px rgba(31,35,40,0.12), inset 0 1px 0 rgba(255,255,255,0.35); position: relative; overflow: hidden; }
.vip-card::before { content: ""; position: absolute; inset: 0; opacity: 0.22; pointer-events: none; background-image: repeating-linear-gradient(-18deg, transparent, transparent 3px, rgba(255,255,255,0.08) 3px, rgba(255,255,255,0.08) 5px); }
.vip-card-emoji { font-size: 2.05rem; line-height: 1.15; position: relative; z-index: 1; text-shadow: 0 1px 2px rgba(0,0,0,0.18); }
.vip-card-label { margin-top: 0.35rem; font-size: 0.78rem; font-weight: 700; letter-spacing: 0.14em; text-transform: uppercase; position: relative; z-index: 1; text-shadow: 0 1px 1px rgba(0,0,0,0.12); }
.vip-card-silver { background: linear-gradient(145deg, #e4e4e8 0%, #b8bcc4 45%, #9a9faa 100%); color: #2f343d; border-color: #aeb3bd; }
.vip-card-gold { background: linear-gradient(145deg, #fff6d2 0%, #e8c86a 42%, #c9a227 100%); color: #3d3208; border-color: #c4a035; }
.vip-card-diamond { background: linear-gradient(145deg, #2a2a32 0%, #121218 55%, #0a0a0e 100%); color: #e8eaef; border-color: #3a3d48; }
.vip-meter-block { margin-top: 0.15rem; }
.vip-meter-row { display: flex; align-items: center; gap: 0.55rem; flex-wrap: nowrap; }
.vip-meter-track { flex: 1; min-width: 0; height: 0.65rem; border-radius: 999px; background: #e4dcc8; border: 1px solid #d8d0c0; overflow: hidden; box-shadow: inset 0 1px 2px rgba(31,35,40,0.08); }
.vip-meter-fill { height: 100%; border-radius: inherit; background: linear-gradient(90deg, #6eb5f7 0%, #0550ae 100%); transition: width 0.25s ease; }
.vip-tier-root:has(.vip-card-silver) .vip-meter-fill { background: linear-gradient(90deg, #aeb3bd 0%, #6e7781 100%); }
.vip-tier-root:has(.vip-card-gold) .vip-meter-fill { background: linear-gradient(90deg, #f0d78c 0%, #b8891a 100%); }
.vip-tier-root:has(.vip-card-diamond) .vip-meter-fill { background: linear-gradient(90deg, #9aa3b5 0%, #4a5160 100%); }
.vip-meter-suffix { flex-shrink: 0; font-size: 0.78rem; color: #5c6670; font-variant-numeric: tabular-nums; white-space: nowrap; }
.vip-stat-line { margin-top: 0.65rem; font-size: 0.88rem; color: #24292f; display: flex; justify-content: space-between; gap: 0.75rem; flex-wrap: wrap; }
.vip-stat-line dt { color: #5c6670; font-weight: 600; margin: 0; }
.vip-stat-line dd { margin: 0; font-family: ui-monospace, monospace; font-variant-numeric: tabular-nums; }
.account-header-left { min-width: 0; }
.account-header-left h1 { margin: 0 0 0.4rem; }
.account-head-row { display: flex; align-items: center; gap: 0.55rem; flex-wrap: wrap; margin: 0 0 0.5rem; }
.account-npub-title { font-size: 1.06rem; font-weight: 700; letter-spacing: 0.01em; font-family: ui-monospace, monospace; overflow-wrap: anywhere; }
.account-vip-emoji { font-size: 1.12rem; line-height: 1; display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; }
.account-summary-wrap { display: flex; justify-content: space-between; align-items: flex-start; gap: 0.8rem; flex-wrap: wrap; margin-bottom: 0.8rem; }
.account-summary-wrap .summary { margin-bottom: 0; flex: 1; min-width: 18rem; }
.account-shell { display: grid; gap: 1rem; }
.account-card { background: #fffefb; border: 1px solid #e8e2d4; border-radius: 10px; box-shadow: 0 1px 2px rgba(0,0,0,0.03); }
.account-card-header { padding: 1rem 1rem 0.2rem; }
.account-card-summary { padding: 0 1rem 1rem; }
.account-card-body { padding: 0.9rem 1rem 1rem; }
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
<body>{}<main>{}</main>{}<script>
(function () {{
  const rowLinks = Array.from(document.querySelectorAll('[data-row-href]'));
  rowLinks.forEach((row) => {{
    const href = row.getAttribute('data-row-href');
    if (!href) return;
    row.addEventListener('click', (ev) => {{
      const target = ev.target;
      if (target && target.closest && target.closest('a,button,input,textarea,select,label')) return;
      window.location.href = href;
    }});
    row.addEventListener('keydown', (ev) => {{
      if (ev.key !== 'Enter' && ev.key !== ' ') return;
      const target = ev.target;
      if (target && target.closest && target.closest('a,button,input,textarea,select,label')) return;
      ev.preventDefault();
      window.location.href = href;
    }});
  }});
  const buttons = Array.from(document.querySelectorAll('[data-expand-target]'));
  buttons.forEach((button) => {{
    const targetId = button.getAttribute('data-expand-target');
    if (!targetId) return;
    const container = button.closest('.expandable-mono');
    const shortNode = container ? container.querySelector('[data-expand-short]') : null;
    const fullNode = document.getElementById(targetId);
    if (!fullNode) return;
    button.addEventListener('click', () => {{
      if (shortNode) shortNode.style.display = 'none';
      button.style.display = 'none';
      fullNode.style.display = 'inline';
    }});
  }});
  const copyButtons = Array.from(document.querySelectorAll('[data-copy-value]'));
  copyButtons.forEach((button) => {{
    const original = button.textContent || 'Copy';
    button.addEventListener('click', async () => {{
      const value = button.getAttribute('data-copy-value') || '';
      try {{
        await navigator.clipboard.writeText(value);
        button.textContent = 'Copied';
        setTimeout(() => {{
          button.textContent = original;
        }}, 900);
      }} catch (_err) {{
        button.textContent = 'Failed';
        setTimeout(() => {{
          button.textContent = original;
        }}, 1100);
      }}
    }});
  }});
}})();
</script></body></html>"#,
        html_escape(title),
        explorer_css(),
        site_header(search_value),
        body,
        site_footer()
    )
}

fn expandable_mono_html(full: &str, head: usize, tail: usize, id_prefix: &str) -> String {
    let short = truncated_head_tail(full, head, tail);
    if short == full {
        return format!(
            r#"<span class="expandable-mono"><code class="mono mono-wrap">{}</code></span>"#,
            html_escape(full)
        );
    }
    let target_id = format!(
        "{}-{}",
        id_prefix,
        hex::encode(bitcoin::hashes::sha256::Hash::hash(full.as_bytes()))
    );
    format!(
        r#"<span class="expandable-mono"><code class="mono mono-wrap" data-expand-short>{}</code><button type="button" class="expandable-mono-btn" data-expand-target="{}">Expand</button><code id="{}" class="mono mono-wrap" style="display:none">{}</code></span>"#,
        html_escape(&short),
        html_escape(&target_id),
        html_escape(&target_id),
        html_escape(full)
    )
}

fn account_avatar_emoji(account_key: [u8; 32]) -> &'static str {
    // Deterministic avatar choice from public key bytes.
    const AVATARS: [&str; 40] = [
        "👻", "🦘", "🐱", "🐰", "🦊", "🐼", "🐨", "🦁", "🐯", "🐸", "🐵", "🐶", "🐻", "🦉",
        "🦄", "🐙", "🐧", "🦋", "🦒", "🦔", "🐺", "🦇", "🦅", "🐬", "🐢", "🦓", "🐘", "🦥",
        "🦦", "🦩", "🦜", "🐿️", "🦝", "🐆", "🐮", "🐗", "🐭", "🐹", "🦎", "🦛",
    ];
    let idx = (account_key[0] as usize) % AVATARS.len();
    AVATARS[idx]
}

fn explorer_format_u64_dec_groups(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn explorer_format_u64_commas(n: u64) -> String {
    explorer_format_u64_dec_groups(&n.to_string())
}

/// Coin amounts in HTML: cent sign, space, then the integer (with thousands grouping).
fn explorer_format_coins_u64(n: u64) -> String {
    format!("¢ {}", explorer_format_u64_commas(n))
}

/// Human-readable refill period for the VIP meter suffix (e.g. 600 → "10 minutes").
fn explorer_format_period_for_bar(secs: u64) -> String {
    if secs == 0 {
        return "0 seconds".to_string();
    }
    let mut rem = secs;
    let mut parts: Vec<String> = Vec::new();
    const UNITS: &[(u64, &str, &str)] = &[
        (86400, "day", "days"),
        (3600, "hour", "hours"),
        (60, "minute", "minutes"),
        (1, "second", "seconds"),
    ];
    for &(div, one, many) in UNITS {
        if rem >= div {
            let k = rem / div;
            rem %= div;
            parts.push(if k == 1 {
                format!("1 {}", one)
            } else {
                format!("{} {}", k, many)
            });
            if parts.len() >= 2 {
                break;
            }
        }
    }
    if parts.is_empty() {
        "0 seconds".to_string()
    } else {
        parts.join(" ")
    }
}

fn explorer_vip_discount_percent_label(discount: u8) -> String {
    let d = (discount as f64).min(200.0);
    let pct = d * 0.5;
    format!("{:.1}%", pct)
}

/// Title-row VIP marker from `reserved_flag_1` (same tiers as the V.I.P. tab); empty when unset.
fn explorer_account_vip_title_emoji_html(reserved_flag_1: u8) -> String {
    let (emoji, label) = match reserved_flag_1 {
        1 => ("⚪", "Silver VIP"),
        2 => ("🪙", "Gold VIP"),
        3 => ("💎", "Diamond VIP"),
        _ => return String::new(),
    };
    format!(
        r#"<span class="account-vip-emoji" title="{}" aria-label="{}">{}</span>"#,
        html_escape(label),
        html_escape(label),
        emoji,
    )
}

fn explorer_vip_tab_inner(
    reserved_flag_1: u8,
    txfee: &Exemption,
    latest_activity_timestamp: u64,
) -> String {
    if reserved_flag_1 == 0 || reserved_flag_1 > 3 {
        return String::new();
    }
    let (emoji, label, card_class) = if reserved_flag_1 == 1 {
        ("⚪", "silver", "vip-card-silver")
    } else if reserved_flag_1 == 2 {
        ("🪙", "gold", "vip-card-gold")
    } else {
        ("💎", "diamond", "vip-card-diamond")
    };
    let default_periodic = PeriodicResource::new(0, 0, 0);
    let pr = txfee
        .periodic_credit
        .as_ref()
        .map(|(periodic, _)| periodic)
        .unwrap_or(&default_periodic);
    let period_label = explorer_format_period_for_bar(pr.period);
    let suffix = format!("/ per {}", period_label);
    let discount_label = explorer_vip_discount_percent_label(txfee.discount.map(|v| v.0).unwrap_or(0));
    let direct_coins = explorer_format_coins_u64(txfee.direct_credit.map(|v| v.0).unwrap_or(0));
    let limit_coins = explorer_format_coins_u64(pr.limit);
    format!(
        r#"<div class="vip-tier-root">
<div class="vip-card {}" role="img" aria-label="{} VIP card">
<div class="vip-card-emoji">{}</div>
<div class="vip-card-label">{}</div>
</div>
<div class="vip-meter-block" data-vip-meter data-period="{}" data-limit="{}" data-latest-left="{}" data-latest-activity-ts="{}">
<div class="vip-meter-row">
<div class="vip-meter-track" data-vip-track role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="0" aria-label="Periodic fee credit remaining (live from your clock)">
<div class="vip-meter-fill" data-vip-fill style="width:0%"></div>
</div>
<span class="vip-meter-suffix">{}</span>
</div>
<p class="muted" style="margin:0.45rem 0 0;font-size:0.78rem"><span class="mono" data-vip-cur-left>—</span> left of <span class="mono">{}</span> per period</p>
</div>
<dl class="vip-stat-line"><dt>Direct credit</dt><dd class="mono">{}</dd></dl>
<dl class="vip-stat-line"><dt>Fee discount (of post-direct fee)</dt><dd class="mono">{}</dd></dl>
</div>"#,
        card_class,
        html_escape(label),
        emoji,
        html_escape(label),
        pr.period,
        pr.limit,
        pr.latest_left,
        latest_activity_timestamp,
        html_escape(&suffix),
        html_escape(&limit_coins),
        html_escape(&direct_coins),
        html_escape(&discount_label),
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
            let r = st.registry.lock().await;
            r.get_account_body_by_account_key(key32).is_some()
        };
        if is_account {
            return Redirect::to(&account_url(key32)).into_response();
        }
        let is_contract = {
            let r = st.registry.lock().await;
            r.get_contract_body_by_contract_id(key32).is_some()
        };
        if is_contract {
            return Redirect::to(&contract_url(key32)).into_response();
        }
    }

    if let Some(account_key) = q_trim.as_str().from_npub() {
        let is_account = {
            let r = st.registry.lock().await;
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
        let g = st.registry.lock().await;
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
                let registry_index = body
                    .get("registry_index")
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
                parsed.push((registry_index, account_key, call_counter, last_activity_timestamp));
            }
        }
        parsed
    };
    rows.sort_by_key(|(registry_index, _, _, _)| *registry_index);

    let mut table_rows = String::new();
    for (registry_index, account_key, call_counter, last_activity_timestamp) in rows {
        let row_href = account_url(account_key);
        table_rows.push_str(&format!(
            r#"<tr class="entry-row-btn" data-row-href="{}" tabindex="0" role="link" aria-label="Open account details"><td class="num">{}</td><td>{}</td><td class="num">{}</td><td>{}</td></tr>"#,
            html_escape(&row_href),
            registry_index,
            account_link(account_key),
            call_counter,
            batch_table_relative_time_html(last_activity_timestamp),
        ));
    }
    if table_rows.is_empty() {
        table_rows = r#"<tr><td colspan="5">No accounts in registry.</td></tr>"#.to_string();
    }

    let body = format!(
        r#"<h1>Accounts</h1>
<p class="muted">Search by account key hex or npub to open details.</p>
<table class="entries-table"><thead><tr><th class="num">Index</th><th>Account</th><th class="num">Call counter</th><th>Last activity</th></tr></thead><tbody>{}</tbody></table>"#,
        table_rows
    );
    Html(layout("Accounts — Cube explorer", &body, ""))
}

async fn page_account_root_redirect(
    State(_st): State<ExplorerState>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let trimmed = account_id.trim();
    if parse_account_key(trimmed).is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Html(layout(
                "Account — Cube explorer",
                &format!(
                    r#"<h1>Invalid account id</h1><p>Expected 32-byte hex or npub; got <code class="mono">{}</code>.</p><p><a class="row-link" href="/accounts">← Accounts</a></p>"#,
                    html_escape(trimmed),
                ),
                "",
            )),
        )
            .into_response();
    }
    let dest = format!("/account/{}/history", account_id.trim());
    Redirect::temporary(dest.as_str()).into_response()
}

async fn page_account_section(
    State(st): State<ExplorerState>,
    Path((account_id, section_slug)): Path<(String, String)>,
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

    let Some(section) = AccountExplorerSection::from_slug(&section_slug) else {
        return (
            StatusCode::NOT_FOUND,
            Html(layout(
                "Account — Cube explorer",
                &format!(
                    r#"<h1>Unknown account section</h1><p>No tab <code class="mono">{}</code>. Use <code>history</code>, <code>registry</code>, <code>vip</code>, <code>privileges</code>, <code>vtxo</code>, or <code>coin-manager</code>.</p><p><a class="row-link" href="/accounts">← Accounts</a></p>"#,
                    html_escape(section_slug.trim()),
                ),
                "",
            )),
        )
            .into_response();
    };

    let account_body = {
        let r = st.registry.lock().await;
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
            let p = pm.lock().await;
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
        let entry_href = format!("/entry/{}", entry_id_hex);
        let entry_kind = match entry {
            Entry::Move(_) => "💰 Move",
            Entry::Call(_) => "📞 Call",
            Entry::Liftup(_) => "🛗 Liftup",
            Entry::Swapout(_) => "🚪 Swapout",
            Entry::Deploy(_) => "🏗 Deploy",
            Entry::Config(_) => "⚙️ Config",
        };
        history_rows.push_str(&format!(
            r#"<tr class="entry-row-btn" data-row-href="{}" tabindex="0" role="link" aria-label="Open entry details"><td>{}</td><td><code class="mono">{}</code></td><td><a class="row-link" href="/batch/height/{}">#{}</a></td><td>{}</td></tr>"#,
            html_escape(&entry_href),
            entry_kind,
            html_escape(&entry_id_hex),
            batch_height,
            batch_height,
            batch_table_relative_time_html(*batch_ts),
        ));
    }
    if history_rows.is_empty() {
        history_rows = r#"<tr><td colspan="4">No transaction history in archival records.</td></tr>"#.to_string();
    }

    let privileges_json = if let Some(ref pb) = privilege_body {
        let liveness_flag = match &pb.liveness_flag {
            crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag::Operational => "operational",
            crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag::ToBeFrozen(_) => "to_be_frozen",
            crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag::ToBeDestroyed(_) => "to_be_destroyed",
        };
        serde_json::json!({
            "liveness_flag": liveness_flag,
            "hierarchy": pb.hierarchy.to_string().to_lowercase(),
            "txfee_exemptions": pb.txfee_exemptions,
            "reserved_flag_1": pb.reserved_flag_1,
            "reserved_flag_2": pb.reserved_flag_2,
            "can_deploy_liquidity": pb.can_deploy_liquidity,
            "can_deploy_contract": pb.can_deploy_contract,
        })
    } else {
        Value::Null
    };
    let privileges_pretty =
        serde_json::to_string_pretty(&privileges_json).unwrap_or_else(|_| "null".to_string());
    let registry_pretty =
        serde_json::to_string_pretty(&account_body.json()).unwrap_or_else(|_| "null".to_string());

    let vtxo_json = {
        let fm = st.flame_manager.lock().await;
        fm.get_account_flame_set(account_key)
            .and_then(|v| serde_json::to_value(v).ok())
            .unwrap_or(Value::Object(Map::new()))
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
    let coin_balance = {
        let cm = st.coin_manager.lock().await;
        cm.get_account_balance(account_key)
    };
    let coin_balance_text = coin_balance
        .map(explorer_format_coins_u64)
        .unwrap_or_else(|| "N/A".to_string());
    let coin_manager_account_json = {
        let cm = st.coin_manager.lock().await;
        cm.get_account_body(account_key)
            .map(|body| {
                serde_json::json!({
                    "balance": body.balance.to_string(),
                    "global_shadow_allocs_sum": body.global_shadow_allocs_sum.to_string(),
                })
            })
            .unwrap_or(serde_json::Value::Null)
    };
    let coin_manager_account_json_pretty = serde_json::to_string_pretty(&coin_manager_account_json)
        .unwrap_or_else(|_| "null".to_string());
    let avatar_emoji = account_avatar_emoji(account_key);

    let vip_tab_body_html = match &privilege_body {
        None => r#"<p class="muted">Privileges manager is not available for this explorer instance.</p>"#
            .to_string(),
        Some(pb) => explorer_vip_tab_inner(
            pb.reserved_flag_1,
            &pb.txfee_exemptions,
            account_body.last_activity_timestamp,
        ),
    };

    let account_vip_title_emoji_html = privilege_body
        .as_ref()
        .map(|pb| explorer_account_vip_title_emoji_html(pb.reserved_flag_1))
        .unwrap_or_default();

    let nav_html = account_explorer_nav(account_id.trim(), section);

    let section_block = match section {
        AccountExplorerSection::History => format!(
            r#"<article class="account-section-page" id="account-section-history" aria-labelledby="account-section-history-heading">
<h2 id="account-section-history-heading">Transaction History</h2>
<table class="entries-table"><thead><tr><th>Entry Kind</th><th>Entry ID</th><th>Batch</th><th>Seen</th></tr></thead><tbody>{}</tbody></table>
</article>"#,
            history_rows,
        ),
        AccountExplorerSection::Registry => format!(
            r#"<article class="account-section-page" id="account-section-registry" aria-labelledby="account-section-registry-heading">
<h2 id="account-section-registry-heading">Registry Account Body</h2>
<pre class="reg-json">{}</pre>
</article>"#,
            html_escape(&registry_pretty),
        ),
        AccountExplorerSection::Vip => format!(
            r#"<article class="account-section-page" id="account-section-vip" aria-label="V.I.P.">
{}
</article>
{}"#,
            vip_tab_body_html,
            explorer_vip_periodic_meter_script(),
        ),
        AccountExplorerSection::Privileges => format!(
            r#"<article class="account-section-page" id="account-section-privileges" aria-labelledby="account-section-privileges-heading">
<h2 id="account-section-privileges-heading">Privileges</h2>
<pre class="reg-json">{}</pre>
</article>"#,
            html_escape(&privileges_pretty),
        ),
        AccountExplorerSection::Vtxo => format!(
            r#"<article class="account-section-page" id="account-section-vtxo" aria-labelledby="account-section-vtxo-heading">
<h2 id="account-section-vtxo-heading">VTXO Set</h2>
<pre class="reg-json">{}</pre>
</article>"#,
            html_escape(&vtxo_pretty),
        ),
        AccountExplorerSection::CoinManager => format!(
            r#"<article class="account-section-page" id="account-section-coin-manager" aria-labelledby="account-section-coin-manager-heading">
<h2 id="account-section-coin-manager-heading">Coin Manager Account Body</h2>
<pre class="reg-json">{}</pre>
</article>"#,
            html_escape(&coin_manager_account_json_pretty),
        ),
    };

    let body = format!(
        r#"<section class="account-shell">
<section class="account-card">
<div class="account-card-header">
<section class="account-header-row">
<div class="account-header-main">
<div class="account-avatar" aria-label="Profile placeholder">{}</div>
<div class="account-header-left">
<h1>Account</h1>
<div class="account-head-row"><span class="account-npub-title">{}</span>{}<span class="badge">{}</span></div>
</div>
</div>
<a class="action-btn" href="{}" target="_blank" rel="noopener">View Nostr Profile ↗</a>
</section>
 </div>
<div class="account-card-summary">
<section class="account-summary-wrap">
<dl class="summary">
<dt>Account key (hex)</dt><dd><div class="summary-copy-row"><code class="mono">{}</code><button type="button" class="copy-btn" data-copy-value="{}" aria-label="Copy account key hex" title="Copy account key hex">&#128203;</button></div></dd>
<dt>Account npub</dt><dd><div class="summary-copy-row"><code class="mono">{}</code><button type="button" class="copy-btn" data-copy-value="{}" aria-label="Copy account npub" title="Copy account npub">&#128203;</button></div></dd>
<dt>Last time called</dt><dd>{}</dd>
<dt>Registry index</dt><dd>{}</dd>
<dt>Call counter</dt><dd>{}</dd>
<dt>Coins</dt><dd>{}</dd>
</dl>
</section>
</div>
</section>
<section class="account-card">
<div class="account-card-body">
{}
{}
<p class="account-subpage-footer"><a class="row-link" href="/accounts">← Accounts</a></p>
</div>
</section>
</section>"#,
        html_escape(avatar_emoji),
        html_escape(&npub_short),
        account_vip_title_emoji_html,
        hierarchy.to_string(),
        html_escape(&nostr_profile_url),
        html_escape(&account_hex),
        html_escape(&account_hex),
        html_escape(&npub),
        html_escape(&npub),
        explorer_timestamp_html(account_body.last_activity_timestamp),
        account_body.registry_index,
        account_body.call_counter,
        html_escape(&coin_balance_text),
        nav_html,
        section_block,
    );
    let page_title = format!("Account — {} — Cube explorer", section.document_title());
    Html(layout(&page_title, &body, &account_id)).into_response()
}

async fn page_contract_root_redirect(
    State(_st): State<ExplorerState>,
    Path(contract_id): Path<String>,
) -> impl IntoResponse {
    let trimmed = contract_id.trim();
    if parse_entry_id_hex(trimmed).is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Html(layout(
                "Contract — Cube explorer",
                &format!(
                    r#"<h1>Invalid contract id</h1><p>Expected 32-byte hex; got <code class="mono">{}</code>.</p><p><a class="row-link" href="/contracts">← Contracts</a></p>"#,
                    html_escape(trimmed),
                ),
                "",
            )),
        )
            .into_response();
    }
    let dest = format!("/contract/{}/registry", trimmed);
    Redirect::temporary(dest.as_str()).into_response()
}

async fn page_contract_section(
    State(st): State<ExplorerState>,
    Path((contract_id, section_slug)): Path<(String, String)>,
) -> impl IntoResponse {
    let Some(contract_key) = parse_entry_id_hex(&contract_id) else {
        return (
            StatusCode::BAD_REQUEST,
            Html(layout(
                "Contract — Cube explorer",
                &format!(
                    r#"<h1>Invalid contract id</h1><p>Expected 32-byte hex; got <code class="mono">{}</code>.</p><p><a class="row-link" href="/contracts">← Contracts</a></p>"#,
                    html_escape(contract_id.trim()),
                ),
                "",
            )),
        )
            .into_response();
    };

    let Some(section) = ContractExplorerSection::from_slug(&section_slug) else {
        return (
            StatusCode::NOT_FOUND,
            Html(layout(
                "Contract — Cube explorer",
                &format!(
                    r#"<h1>Unknown contract section</h1><p>No tab <code class="mono">{}</code>. Use <code>registry</code>, <code>privileges</code>, or <code>coin-manager</code>.</p><p><a class="row-link" href="/contracts">← Contracts</a></p>"#,
                    html_escape(section_slug.trim()),
                ),
                "",
            )),
        )
            .into_response();
    };

    let contract_body = {
        let r = st.registry.lock().await;
        r.get_contract_body_by_contract_id(contract_key)
    };
    let Some(contract_body) = contract_body else {
        return (
            StatusCode::NOT_FOUND,
            Html(layout(
                "Contract — Cube explorer",
                &format!(
                    r#"<h1>Contract not found</h1><p>No contract found for <code class="mono">{}</code>.</p><p><a class="row-link" href="/contracts">← Contracts</a></p>"#,
                    html_escape(&contract_id),
                ),
                "",
            )),
        )
            .into_response();
    };

    let privilege_body = match st.privileges_manager.as_ref() {
        Some(pm) => {
            let p = pm.lock().await;
            p.get_contract_body_by_contract_id(contract_key)
        }
        None => None,
    };

    let contract_hex = hex::encode(contract_key);
    let contract_short = truncated_head_tail(&contract_hex, 8, 6);
    let registry_pretty =
        serde_json::to_string_pretty(&contract_body.json()).unwrap_or_else(|_| "null".to_string());
    let privileges_json = if let Some(ref pb) = privilege_body {
        let liveness_flag = match &pb.liveness_flag {
            crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag::Operational => "operational",
            crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag::ToBeFrozen(_) => "to_be_frozen",
            crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag::ToBeDestroyed(_) => "to_be_destroyed",
        };
        serde_json::json!({
            "liveness_flag": liveness_flag,
            "immutability": pb.immutability,
            "tax_exemptions": pb.tax_exemptions,
        })
    } else {
        Value::Null
    };
    let privileges_pretty =
        serde_json::to_string_pretty(&privileges_json).unwrap_or_else(|_| "null".to_string());
    let contract_coin_balance_text = {
        let cm = st.coin_manager.lock().await;
        cm.get_contract_balance(contract_key)
            .map(explorer_format_coins_u64)
            .unwrap_or_else(|| "N/A".to_string())
    };
    let contract_coin_manager_json_pretty = {
        let cm = st.coin_manager.lock().await;
        let v = cm
            .get_contract_body(contract_key)
            .map(|body| {
                serde_json::json!({
                    "balance": body.balance.to_string(),
                    "shadow_space_allocs_sum": body.shadow_space.allocs_sum.to_string(),
                    "allocs": body.shadow_space.allocs.iter().map(|(k, v)| {
                        (hex::encode(k), serde_json::Value::String(v.to_string()))
                    }).collect::<serde_json::Map<String, serde_json::Value>>(),
                })
            })
            .unwrap_or(serde_json::Value::Null);
        serde_json::to_string_pretty(&v).unwrap_or_else(|_| "null".to_string())
    };

    let nav_html = contract_explorer_nav(contract_id.trim(), section);
    let section_block = match section {
        ContractExplorerSection::Registry => format!(
            r#"<article class="account-section-page" id="contract-section-registry" aria-labelledby="contract-section-registry-heading">
<h2 id="contract-section-registry-heading">Registry Contract Body</h2>
<pre class="reg-json">{}</pre>
</article>"#,
            html_escape(&registry_pretty),
        ),
        ContractExplorerSection::Privileges => format!(
            r#"<article class="account-section-page" id="contract-section-privileges" aria-labelledby="contract-section-privileges-heading">
<h2 id="contract-section-privileges-heading">Privileges</h2>
<pre class="reg-json">{}</pre>
</article>"#,
            html_escape(&privileges_pretty),
        ),
        ContractExplorerSection::CoinManager => format!(
            r#"<article class="account-section-page" id="contract-section-coin-manager" aria-labelledby="contract-section-coin-manager-heading">
<h2 id="contract-section-coin-manager-heading">Coin Manager Contract Body</h2>
<pre class="reg-json">{}</pre>
</article>"#,
            html_escape(&contract_coin_manager_json_pretty),
        ),
    };

    let body = format!(
        r#"<section class="account-shell">
<section class="account-card">
<div class="account-card-header">
<section class="account-header-row">
<div class="account-header-main">
<div class="account-avatar" aria-label="Contract icon">📁</div>
<div class="account-header-left">
<h1>Contract</h1>
<div class="account-head-row"><span class="account-npub-title">{}</span></div>
</div>
</div>
</section>
 </div>
<div class="account-card-summary">
<section class="account-summary-wrap">
<dl class="summary">
<dt>Contract ID</dt><dd><div class="summary-copy-row"><code class="mono">{}</code><button type="button" class="copy-btn" data-copy-value="{}" aria-label="Copy contract id" title="Copy contract id">&#128203;</button></div></dd>
<dt>Program name</dt><dd>{}</dd>
<dt>Last active</dt><dd>{}</dd>
<dt>Registry index</dt><dd>{}</dd>
<dt>Call counter</dt><dd>{}</dd>
<dt>Coins</dt><dd>{}</dd>
</dl>
</section>
</div>
</section>
<section class="account-card">
<div class="account-card-body">
{}
{}
<p class="account-subpage-footer"><a class="row-link" href="/contracts">← Contracts</a></p>
</div>
</section>
</section>"#,
        html_escape(&contract_short),
        html_escape(&contract_hex),
        html_escape(&contract_hex),
        html_escape(contract_body.executable.program_name()),
        explorer_timestamp_html(contract_body.last_activity_timestamp),
        contract_body.registry_index,
        contract_body.call_counter,
        html_escape(&contract_coin_balance_text),
        nav_html,
        section_block,
    );
    let page_title = format!("Contract — {} — Cube explorer", section.document_title());
    Html(layout(&page_title, &body, &contract_id)).into_response()
}

async fn page_contracts(State(st): State<ExplorerState>) -> Html<String> {
    let mut rows: Vec<(u64, [u8; 32], String, u64, u64)> = {
        let g = st.registry.lock().await;
        let full = g.json();
        let mut parsed = Vec::new();
        if let Some(contracts_obj) = full.get("contracts").and_then(|v| v.as_object()) {
            for (contract_id_hex, body) in contracts_obj {
                let Some(bytes_vec) = hex::decode(contract_id_hex).ok() else {
                    continue;
                };
                let Ok(contract_id): Result<[u8; 32], _> = bytes_vec.try_into() else {
                    continue;
                };
                let registry_index = body
                    .get("registry_index")
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
                let program_name = body
                    .get("executable")
                    .and_then(|v| v.get("program_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A")
                    .to_string();
                parsed.push((
                    registry_index,
                    contract_id,
                    program_name,
                    call_counter,
                    last_activity_timestamp,
                ));
            }
        }
        parsed
    };
    rows.sort_by_key(|(registry_index, _, _, _, _)| *registry_index);

    let mut table_rows = String::new();
    for (registry_index, contract_id, program_name, call_counter, last_activity_timestamp) in rows {
        let contract_id_hex = hex::encode(contract_id);
        let contract_id_short = truncated_head_tail(&contract_id_hex, 8, 6);
        let row_href = contract_url(contract_id);
        table_rows.push_str(&format!(
            r#"<tr class="entry-row-btn" data-row-href="{}" tabindex="0" role="link" aria-label="Open contract details"><td class="num">{}</td><td>{}</td><td><code class="mono">{}</code></td><td class="num">{}</td><td>{}</td></tr>"#,
            html_escape(&row_href),
            registry_index,
            html_escape(&program_name),
            html_escape(&contract_id_short),
            call_counter,
            batch_table_relative_time_html(last_activity_timestamp),
        ));
    }
    if table_rows.is_empty() {
        table_rows = r#"<tr><td colspan="5">No contracts in registry.</td></tr>"#.to_string();
    }

    Html(layout(
        "Contracts — Cube explorer",
        &format!(r#"<h1>Contracts</h1>
<p class="muted">Contracts indexed by registry order.</p>
<table class="entries-table"><thead><tr><th class="num">Index</th><th>Program name</th><th>Contract ID</th><th class="num">Call counter</th><th>Last time called</th></tr></thead><tbody>{}</tbody></table>"#, table_rows),
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
        let row_href = format!("/batch/height/{}", height);
        table_rows.push_str(&format!(
            r#"<tr class="entry-row-btn" data-row-href="{5}" tabindex="0" role="link" aria-label="Open batch details"><td><span class="mono">#{0}</span></td><td class="mono"><a class="row-link" href="/batch/tx/{3}">{2}</a></td><td class="num">{4}</td><td>{1}</td></tr>"#,
            height,
            ts_html,
            html_escape(&txid),
            html_escape(&txid),
            num_entries,
            html_escape(&row_href),
        ));
    }

    let body = format!(
        r#"<h1><span class="badge">{}</span> Latest batches</h1>
<table class="entries-table"><thead><tr><th>Height</th><th>Txid</th><th class="num">Number of Entries</th><th>Seen</th></tr></thead><tbody>{}</tbody></table>"#,
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
        Entry::Swapout(_) => "🚪 Swapout",
        Entry::Deploy(_) => "🏗 Deploy",
        Entry::Config(_) => "⚙️ Config",
    };
    let entry_accounts_html = match &entry {
        Entry::Move(move_entry) => format!(
            r#"<dt>From</dt><dd>{}</dd><dt>To</dt><dd>{}</dd>"#,
            account_link(move_entry.from.account_key()),
            account_link(move_entry.to.account_key())
        ),
        Entry::Liftup(liftup) => format!(
            r#"<dt>Account</dt><dd>{}</dd>"#,
            account_link(liftup.root_account.account_key())
        ),
        Entry::Swapout(swapout) => format!(
            r#"<dt>Account</dt><dd>{}</dd>"#,
            account_link(swapout.root_account.account_key())
        ),
        Entry::Call(call) => format!(
            r#"<dt>Account</dt><dd>{}</dd>"#,
            account_link(call.account.account_key())
        ),
        Entry::Deploy(deploy) => format!(
            r#"<dt>Account</dt><dd>{}</dd>"#,
            account_link(deploy.root_account.account_key())
        ),
        Entry::Config(config) => format!(
            r#"<dt>Account</dt><dd>{}</dd>"#,
            account_link(config.root_account.account_key())
        ),
    };
    let entry_coins_html = match &entry {
        Entry::Move(move_entry) => format!(
            r#"<dt>Coins</dt><dd>{}</dd>"#,
            html_escape(&explorer_format_coins_u64(move_entry.amount as u64))
        ),
        _ => String::new(),
    };
    let collected_bits_html = match collected_bits {
        Some(bits) => {
            expandable_mono_html(&bits, 28, 16, "ape-bits")
        }
        None => "N/A (non-archival record)".to_string(),
    };

    let body = format!(
        r#"<section class="account-card">
<div class="account-card-header"><h1>{}</h1></div>
<div class="account-card-summary">
<dl class="summary">
<dt>Entry id</dt><dd><code class="mono">{}</code></dd>
<dt>Batch height</dt><dd><a class="row-link" href="/batch/height/{}">#{}</a></dd>
<dt>Batch txid</dt><dd>{}</dd>
<dt>Batch timestamp</dt><dd>{}</dd>
<dt>APE bitstream</dt><dd>{}</dd>
{}
{}
</dl>
</div>
</section>
<h2 style="font-size:1.1rem;margin:1.25rem 0 0.65rem">Entry data</h2>
<pre class="reg-json">{}</pre>
<h2 style="font-size:1.1rem;margin:1.25rem 0 0.65rem">Entry fees</h2>
<pre class="reg-json">{}</pre>
<p style="margin-top:1.25rem"><a class="row-link" href="/batch/height/{}">← Batch #{}</a> · <a class="row-link" href="/batches">All batches</a></p>"#,
        entry_kind_title,
        html_escape(&eid_hex),
        batch_height,
        batch_height,
        batch_txid_dd,
        ts_html,
        collected_bits_html,
        entry_accounts_html,
        entry_coins_html,
        html_escape(&entry_json),
        html_escape(&entry_fees_json),
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
    let txid_cell = format!(r#"<code class="mono">{}</code>"#, html_escape(&txid));
    let mempool_button_html = match mempool_tx_url(chain, &txid) {
        Some(url) => format!(
            r#"<a class="action-btn" href="{}" target="_blank" rel="noopener">View on mempool.space ↗</a>"#,
            html_escape(&url),
        ),
        None => String::new(),
    };

    let mut entries_rows_html = String::new();
    for (entry_id, entry) in record.entries.iter() {
        let eid = hex::encode(entry_id);
        let entry_href = format!("/entry/{}", eid);
        let entry_kind = match entry {
            Entry::Move(_) => "💰 Move",
            Entry::Call(_) => "📞 Call",
            Entry::Liftup(_) => "🛗 Liftup",
            Entry::Swapout(_) => "🚪 Swapout",
            Entry::Deploy(_) => "🏗 Deploy",
            Entry::Config(_) => "⚙️ Config",
        };
        let amount_cell = match entry {
            Entry::Move(move_entry) => explorer_format_coins_u64(move_entry.amount as u64),
            Entry::Liftup(liftup) => explorer_format_coins_u64(liftup.liftup_sum_value_in_satoshis()),
            Entry::Swapout(swapout) => explorer_format_coins_u64(swapout.amount as u64),
            Entry::Deploy(deploy) => explorer_format_coins_u64(deploy.initial_balance as u64),
            Entry::Call(_) | Entry::Config(_) => "N/A".to_string(),
        };
        let account_cell = match entry {
            Entry::Move(move_entry) => format!(
                "{} → {}",
                account_link_npub_truncated(move_entry.from.account_key()),
                account_link_npub_truncated(move_entry.to.account_key())
            ),
            Entry::Liftup(liftup) => account_link_npub_truncated(liftup.root_account.account_key()),
            Entry::Swapout(swapout) => account_link_npub_truncated(swapout.root_account.account_key()),
            Entry::Call(call) => account_link_npub_truncated(call.account.account_key()),
            Entry::Deploy(deploy) => account_link_npub_truncated(deploy.root_account.account_key()),
            Entry::Config(config) => account_link_npub_truncated(config.root_account.account_key()),
        };
        entries_rows_html.push_str(&format!(
            r#"<tr class="entry-row-btn" data-row-href="{}" tabindex="0" role="link" aria-label="Open entry details"><td>{}</td><td>{}</td><td class="mono">{}</td></tr>"#,
            html_escape(&entry_href),
            entry_kind,
            account_cell,
            html_escape(&amount_cell),
        ));
    }

    if entries_rows_html.is_empty() {
        entries_rows_html =
            r#"<tr><td colspan="3">No entries in this batch.</td></tr>"#.to_string();
    }

    let body = format!(
        r#"<section class="account-card">
<div class="account-card-header">
<section class="account-header-row">
<div class="account-header-main"><h1>📦 Batch #{}</h1></div>
{}
</section>
</div>
<div class="account-card-summary">
<dl class="summary">
<dt>Height</dt><dd><span class="mono">#{}</span></dd>
<dt>Timestamp</dt><dd>{}</dd>
<dt>Txid</dt><dd><div class="summary-copy-row">{}<button type="button" class="copy-btn" data-copy-value="{}" aria-label="Copy batch txid" title="Copy batch txid">&#128203;</button></div></dd>
<dt>BLS agg sig</dt><dd>{}</dd>
<dt>Payload version</dt><dd><span class="mono">{}</span></dd>
<dt>Entries</dt><dd><span class="mono">{}</span></dd>
</dl>
</div>
</section>
<section class="entries"><h2 style="font-size:1.1rem;margin-bottom:0.75rem">Entries</h2><table class="entries-table"><thead><tr><th>Entry Kind</th><th>Account(s)</th><th>Amount</th></tr></thead><tbody>{}</tbody></table></section>
<p><a class="row-link" href="/batches">← All batches</a></p>"#,
        record.batch_height,
        mempool_button_html,
        record.batch_height,
        ts_html,
        txid_cell,
        txid,
        {
            let bls_full = hex::encode(&record.aggregate_bls_signature);
            expandable_mono_html(&bls_full, 28, 16, "bls-agg-sig")
        },
        record.payload_version,
        record.entries.len(),
        entries_rows_html
    );

    layout(
        &format!("Batch {} — Cube explorer", record.batch_height),
        &body,
        "",
    )
}
