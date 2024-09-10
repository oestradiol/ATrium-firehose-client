use anyhow::{bail, Result};
use atrium_api::com::atproto::sync::subscribe_repos::Handle as Message;

use crate::subscriptions::handlers::Handle;

use super::super::Firehose;

#[derive(Debug)]
pub struct ProcessedData;

impl Handle for Firehose {
  type ProcessedData = self::ProcessedData;

  async fn handle(&self, message: Message) -> Result<(i64, Self::ProcessedData)> {
    bail!("Not implemented") // TODO
  }
}
