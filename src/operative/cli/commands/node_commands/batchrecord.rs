use crate::communicative::peer::peer::PEER;
use crate::communicative::tcp::client::{BatchRecordResponseBody, TCPClient};
use colored::Colorize;
use serde_json::to_string_pretty;

// batchrecord
pub async fn batchrecord_command(batch_height: u64, engine_peer: &PEER) {
    // 1 Request the batch record from the engine.
    let (batchrecord_response_body, duration) = match engine_peer
        .request_batchrecord(batch_height)
        .await
    {
        Ok((body, duration)) => (body, duration),
        Err(error) => {
            println!(
                "{}",
                format!("Error requesting batch record: {:?}", error).red()
            );
            return;
        }
    };

    // 2 Match the batch record result (wire enum, not `Result`).
    match batchrecord_response_body {
        BatchRecordResponseBody::Ok(success_body) => match success_body.batch_record {
            Some(batch_record) => {
                println!(
                    "{}",
                    format!(
                        "Batch record for height #{} ({} ms):\n{}",
                        batch_height,
                        duration.as_millis(),
                        to_string_pretty(&batch_record.json())
                            .expect("serde_json::Value should serialize")
                    )
                    .green()
                );
            }
            None => {
                println!(
                    "{}",
                    format!(
                        "No batch record found for height #{} ({} ms).",
                        batch_height,
                        duration.as_millis()
                    )
                    .yellow()
                );
            }
        },
        BatchRecordResponseBody::Err(error) => {
            println!(
                "{}",
                format!(
                    "Error resolving batch record: {}",
                    to_string_pretty(&error.json()).expect("serde_json::Value should serialize")
                )
                .red()
            );
        }
    }
}
