use anyhow::{bail, Result};
use atrium_api::com::atproto::sync::subscribe_repos::Identity as Message;

use crate::subscriptions::handlers::Identity;

use super::super::Firehose;

#[derive(Debug)]
pub struct ProcessedData;

impl Identity for Firehose {
  type ProcessedData = self::ProcessedData;

  async fn handle(&self, message: Message) -> Result<(i64, Self::ProcessedData)> {
    bail!("Not implemented") // TODO
  }
}
