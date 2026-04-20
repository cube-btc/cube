use crate::communicative::peer::peer::PEER;
use crate::communicative::tcp::protocol::batchrecord::client::request_batchrecord;
use crate::communicative::tcp::protocol::batchrecord::BatchRecordResponseBody;
use crate::communicative::tcp::protocol::batchcontainer::client::request_batchcontainer;
use crate::communicative::tcp::protocol::batchcontainer::BatchContainerResponseBody;
use crate::communicative::tcp::protocol::batchcontainer_by_prevoutpoint::client::request_batchcontainer_by_prevoutpoint;
use crate::communicative::tcp::protocol::batchcontainer_by_prevoutpoint::BatchContainerByPrevOutpointResponseBody;
use crate::communicative::tcp::protocol::in_flight_sync::client::request_in_flight_sync::request_in_flight_sync;
use crate::communicative::tcp::protocol::in_flight_sync::InFlightSyncResponseBody;
use crate::communicative::tcp::protocol::liftup_v1::client::request_liftup_v1;
use crate::communicative::tcp::protocol::liftup_v1::LiftupV1ResponseBody;
use crate::communicative::tcp::request_error::RequestError;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use async_trait::async_trait;
use bitcoin::OutPoint;
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

    async fn request_batchcontainer(
        &self,
        batch_height: u64,
    ) -> Result<(BatchContainerResponseBody, Duration), RequestError> {
        request_batchcontainer(self, batch_height).await
    }

    async fn request_batchcontainer_by_prevoutpoint(
        &self,
        prev_payload_outpoint: OutPoint,
    ) -> Result<(BatchContainerByPrevOutpointResponseBody, Duration), RequestError> {
        request_batchcontainer_by_prevoutpoint(self, prev_payload_outpoint).await
    }

    async fn request_in_flight_sync(
        &self,
        cube_batch_sync_height_tip: u64,
    ) -> Result<(InFlightSyncResponseBody, Duration), RequestError> {
        request_in_flight_sync(self, cube_batch_sync_height_tip).await
    }
}
