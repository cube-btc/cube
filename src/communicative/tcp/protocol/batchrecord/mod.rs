//! Batch record TCP: wire bodies, client send path, server handler.

pub mod bodies;
pub mod client;
pub mod server;

pub use bodies::{
    BatchRecordRequestBody, BatchRecordResponseBody, BatchRecordResponseError,
    BatchRecordSuccessBody,
};
