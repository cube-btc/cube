//! Config TCP protocol: wire bodies, client send path, server handler.

pub mod bodies;
pub mod client;
pub mod server;

pub use crate::operative::tasks::engine_session::session_pool::error::exec_config_in_pool_error::ExecConfigInPoolError;
pub use bodies::{ConfigRequestBody, ConfigResponseBody, ConfigResponseError, ConfigSuccessBody};
