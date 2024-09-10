use anyhow::Result;
use atrium_api::com::atproto::sync::subscribe_repos::Info as Message;
use std::fmt::Debug;

pub trait Info {
  type ProcessedData: Debug;

  fn handle(
    &self,
    message: Message,
  ) -> impl std::future::Future<Output = Result<(i64, Self::ProcessedData)>>;
}
