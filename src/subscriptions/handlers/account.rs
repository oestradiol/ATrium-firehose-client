use anyhow::Result;
use atrium_api::com::atproto::sync::subscribe_repos::Account as Message;

pub trait Account {
  type ProcessedData;

  fn handle(
    &self,
    message: Message,
  ) -> impl std::future::Future<Output = Result<(i64, Self::ProcessedData)>>;
}
