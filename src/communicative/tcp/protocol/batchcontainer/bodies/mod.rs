//! Bincode wire bodies for Batch container over TCP.

mod request_body;
mod response_body;

pub use request_body::BatchContainerRequestBody;
pub use response_body::{
    BatchContainerResponseBody, BatchContainerResponseError, BatchContainerSuccessBody,
};
