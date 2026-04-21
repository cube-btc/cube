//! Bincode wire bodies for Move over TCP.

mod request_body;
mod response_body;

pub use request_body::MoveRequestBody;
pub use response_body::{MoveResponseBody, MoveResponseError, MoveSuccessBody};
