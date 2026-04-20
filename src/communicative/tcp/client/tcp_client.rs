use crate::communicative::tcp::protocol::batchrecord::BatchRecordResponseBody;
use crate::communicative::tcp::protocol::in_flight_sync::InFlightSyncResponseBody;
use crate::communicative::tcp::protocol::liftup_v1::LiftupV1ResponseBody;
use crate::communicative::tcp::request_error::RequestError;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait TCPClient {
    async fn ping(&self) -> Result<Duration, RequestError>;
    async fn request_liftup_v1(
        &self,
        liftup: &Liftup,
        liftup_bls_signature: [u8; 96],
    ) -> Result<(LiftupV1ResponseBody, Duration), RequestError>;
    async fn request_batchrecord(
        &self,
        batch_height: u64,
    ) -> Result<(BatchRecordResponseBody, Duration), RequestError>;
    async fn request_in_flight_sync(
        &self,
        cube_batch_sync_height_tip: u64,
    ) -> Result<(InFlightSyncResponseBody, Duration), RequestError>;
}
