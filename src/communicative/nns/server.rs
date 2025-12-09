use super::client::NNSClient;
use crate::{communicative::tcp::tcp::TCP_RESPONSE_TIMEOUT, operative::OperatingKind};
use colored::Colorize;
use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::Path,
    time::Duration,
};
use tokio::time::timeout;

/// Path to the file that stores the IP address of the running machine.
#[allow(non_camel_case_types)]
const IP_ADDR_FILE_PATH: &str = "nns_ip_address.txt";

/// Executes a persistent task that monitors changes
/// in the dynamic IP address of the running machine.
/// If a change is detected, it posts the update to Nostr,
/// allowing NNS clients to retrieve it via the well-known npub.
///
pub async fn run(nns_client: &NNSClient, operating_kind: OperatingKind) {
    match operating_kind {
        OperatingKind::Coordinator => (),
        OperatingKind::Operator => (),
        OperatingKind::Node => return, // Refular nodes do not run the server.
    }

    // Check if ip.txt exists. Create it otherwise.
    if !Path::new(IP_ADDR_FILE_PATH).exists() {
        fs::File::create(IP_ADDR_FILE_PATH).unwrap();
    }

    // Enter the periodic check loop.
    loop {
        match timeout(Duration::from_secs(TCP_RESPONSE_TIMEOUT), check_ip()).await {
            Ok(response) => {
                match response {
                    Ok(option) => match option {
                        Some(ip_address) => {
                            // IP address change detected.
                            println!("New IP address detected: {}", ip_address);

                            loop {
                                // Publish the new IP address.
                                match nns_client.publish_address(&ip_address).await {
                                    Some(event_id) => {
                                        println!(
                                            "{}",
                                            format!(
                                                "Published new address: {}",
                                                hex::encode(event_id)
                                            )
                                            .green()
                                        );
                                        break;
                                    }
                                    None => {
                                        // Failed to publish IP address.
                                        eprintln!(
                                            "{}",
                                            "Failed to publish IP address. Re-trying in 5.."
                                                .yellow()
                                        );

                                        tokio::time::sleep(Duration::from_secs(5)).await;
                                        continue;
                                    }
                                }
                            }
                        }
                        None => {
                            // No IP address change detected.
                            tokio::time::sleep(Duration::from_secs(30)).await;
                        }
                    },
                    Err(_) => {
                        // Failed to query IP address.
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
            Err(_) => {
                // Failed to query IP address due to timeout.
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

/// Checks whether there was a change in the IP address.
///
async fn check_ip() -> Result<Option<String>, reqwest::Error> {
    let ip_address = match query_ip_address().await {
        Ok(found_ip) => found_ip,
        Err(error) => return Err(error),
    };

    match retrieve_latest_known_ip_address().await {
        Some(ip_address_latest_known) => {
            if ip_address != ip_address_latest_known {
                let _ = update_latest_known_ip_address(&ip_address);
                return Ok(Some(ip_address));
            }
        }
        None => {
            let _ = update_latest_known_ip_address(&ip_address);
            return Ok(Some(ip_address));
        }
    }

    Ok(None)
}

/// Queries the dynamic IP address of the running machine.
///
async fn query_ip_address() -> Result<String, reqwest::Error> {
    let url = "https://api.ipify.org";
    let ip = reqwest::get(url).await?.text().await?;

    Ok(ip)
}

/// Retrieves the latest known IP address from disk.
///
async fn retrieve_latest_known_ip_address() -> Option<String> {
    let mut file = OpenOptions::new().read(true).open(IP_ADDR_FILE_PATH).ok()?;

    let mut read_ip = String::new();
    file.read_to_string(&mut read_ip).ok()?;

    match read_ip.len() {
        0 => None,
        _ => Some(read_ip),
    }
}

/// Updates the latest known IP address on disk.
///
fn update_latest_known_ip_address(ip_address: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(IP_ADDR_FILE_PATH)?;
    file.write_all(ip_address.as_bytes())?;
    Ok(())
}
