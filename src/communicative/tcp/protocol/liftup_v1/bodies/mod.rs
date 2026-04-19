//! Bincode wire bodies for Liftup v1 over TCP.

mod request_body;
mod response_body;

pub use request_body::LiftupV1RequestBody;
pub use response_body::{
    LiftupV1ResponseBody, LiftupV1ResponseError, LiftupV1SuccessBody,
};
