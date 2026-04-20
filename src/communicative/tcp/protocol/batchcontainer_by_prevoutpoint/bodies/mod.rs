//! Bincode wire bodies for Batch container-by-prevoutpoint over TCP.

mod request_body;
mod response_body;

pub use request_body::BatchContainerByPrevOutpointRequestBody;
pub use response_body::{
    BatchContainerByPrevOutpointResponseBody, BatchContainerByPrevOutpointResponseError,
    BatchContainerByPrevOutpointSuccessBody,
};
