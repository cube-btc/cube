//! Engine-side handler for a single ping package.

use crate::communicative::tcp::package::{PackageKind, TCPPackage};

/// Builds the ping response package, or `None` if the request payload is invalid.
pub async fn handle_ping_request(timestamp: i64, payload: &[u8]) -> Option<TCPPackage> {
    if payload != &[0x00] {
        return None;
    }

    let response_package = {
        let kind = PackageKind::Ping;
        let payload = [0x01u8];
        TCPPackage::new(kind, timestamp, &payload)
    };

    Some(response_package)
}
