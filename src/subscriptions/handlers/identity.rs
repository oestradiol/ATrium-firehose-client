use anyhow::Result;
use atrium_api::com::atproto::sync::subscribe_repos::Identity as Message;
use std::fmt::Debug;

pub trait Identity {
  type ProcessedData: Debug;

  fn handle(
    &self,
    message: Message,
  ) -> impl std::future::Future<Output = Result<(i64, Self::ProcessedData)>>;
}
