//! Bincode wire bodies for Batch record over TCP.

mod request_body;
mod response_body;

pub use request_body::BatchRecordRequestBody;
pub use response_body::{
    BatchRecordResponseBody, BatchRecordResponseError, BatchRecordSuccessBody,
};
