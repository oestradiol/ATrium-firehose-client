use anyhow::{bail, Result};
use atrium_api::com::atproto::sync::subscribe_repos::Migrate as Message;

use crate::subscriptions::handlers::Migrate;

use super::super::Firehose;

#[derive(Debug)]
pub struct ProcessedData;

impl Migrate for Firehose {
  type ProcessedData = self::ProcessedData;

  async fn handle(&self, message: Message) -> Result<(i64, Self::ProcessedData)> {
    bail!("Not implemented") // TODO
  }
}
