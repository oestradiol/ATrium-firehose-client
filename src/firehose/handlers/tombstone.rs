use anyhow::{bail, Result};
use atrium_api::com::atproto::sync::subscribe_repos::Tombstone as Message;

use crate::subscriptions::handlers::Tombstone;

use super::super::Firehose;

#[derive(Debug)]
pub struct ProcessedData;

impl Tombstone for Firehose {
  type ProcessedData = self::ProcessedData;

  async fn handle(&self, message: Message) -> Result<(i64, Self::ProcessedData)> {
    bail!("Not implemented") // TODO
  }
}
