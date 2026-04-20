//! Batch container-by-prevoutpoint TCP: wire bodies, client send path, server handler.

pub mod bodies;
pub mod client;
pub mod server;

pub use bodies::{
    BatchContainerByPrevOutpointRequestBody, BatchContainerByPrevOutpointResponseBody,
    BatchContainerByPrevOutpointResponseError, BatchContainerByPrevOutpointSuccessBody,
};
