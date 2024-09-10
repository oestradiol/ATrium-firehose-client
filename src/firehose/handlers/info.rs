use anyhow::{bail, Result};
use atrium_api::com::atproto::sync::subscribe_repos::Info as Message;

use crate::subscriptions::handlers::Info;

use super::super::Firehose;

#[derive(Debug)]
pub struct ProcessedData;

impl Info for Firehose {
  type ProcessedData = self::ProcessedData;

  async fn handle(&self, message: Message) -> Result<(i64, ProcessedData)> {
    bail!("Not implemented") // TODO
  }
}
