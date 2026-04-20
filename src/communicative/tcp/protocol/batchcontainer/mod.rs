//! Batch container TCP: wire bodies, client send path, server handler.

pub mod bodies;
pub mod client;
pub mod server;

pub use bodies::{
    BatchContainerRequestBody, BatchContainerResponseBody, BatchContainerResponseError,
    BatchContainerSuccessBody,
};
