use crate::communicative::peer::peer::PEER;
use crate::communicative::tcp::protocol::batchrecord::client::request_batchrecord;
use crate::communicative::tcp::protocol::batchrecord::BatchRecordResponseBody;
use crate::communicative::tcp::protocol::liftup_v1::client::request_liftup_v1;
use crate::communicative::tcp::protocol::liftup_v1::LiftupV1ResponseBody;
use crate::communicative::tcp::request_error::RequestError;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use async_trait::async_trait;
use std::time::Duration;

use super::tcp_client::TCPClient;

#[async_trait]
impl TCPClient for PEER {
    async fn ping(&self) -> Result<Duration, RequestError> {
        crate::communicative::tcp::protocol::ping::client::request_ping(self).await
    }

    async fn request_liftup_v1(
        &self,
        liftup: &Liftup,
        liftup_bls_signature: [u8; 96],
    ) -> Result<(LiftupV1ResponseBody, Duration), RequestError> {
        request_liftup_v1(self, liftup, liftup_bls_signature).await
    }

    async fn request_batchrecord(
        &self,
        batch_height: u64,
    ) -> Result<(BatchRecordResponseBody, Duration), RequestError> {
        request_batchrecord(self, batch_height).await
    }
}
