//! Deploy TCP protocol: wire bodies, client send path, server handler.

pub mod bodies;
pub mod client;
pub mod server;

pub use crate::operative::tasks::engine_session::session_pool::error::exec_deploy_in_pool_error::ExecDeployInPoolError;
pub use bodies::{DeployRequestBody, DeployResponseBody, DeployResponseError, DeploySuccessBody};
