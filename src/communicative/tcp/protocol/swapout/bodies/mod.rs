//! Bincode wire bodies for Swapout over TCP.

mod request_body;
mod response_body;

pub use request_body::SwapoutRequestBody;
pub use response_body::{SwapoutResponseBody, SwapoutResponseError, SwapoutSuccessBody};
