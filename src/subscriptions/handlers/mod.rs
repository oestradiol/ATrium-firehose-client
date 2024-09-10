mod commit;

pub use commit::Commit;

mod account;
pub use account::Account;

mod handle;
pub use handle::Handle;

mod identity;
pub use identity::Identity;

mod info;
pub use info::Info;

mod migrate;
pub use migrate::Migrate;

mod tombstone;
pub use tombstone::Tombstone;

use crate::utils::frames::MessageFrame;
use anyhow::{bail, Result};

pub type HandledMessage<H> = Message<
  <H as Commit>::ProcessedData,
  <H as Identity>::ProcessedData,
  <H as Account>::ProcessedData,
  <H as Handle>::ProcessedData,
  <H as Migrate>::ProcessedData,
  <H as Tombstone>::ProcessedData,
  <H as Info>::ProcessedData,
>;

#[derive(Debug)]
pub enum Message<C, I0, A, H, M, T, I1> {
  Commit(C),
  Identity(I0),
  Account(A),
  Handle(H),
  Migrate(M),
  Tombstone(T),
  Info(I1),
}

/// A trait that combines all the handlers into one.
/// Any struct that correctly implements this trait will be able to handle all the
/// different message types that the firehose can send.
pub trait Handler: Account + Commit + Handle + Identity + Info + Migrate + Tombstone {
  /// Handle a message from the firehose. This function will deserialize the message body
  /// and call the appropriate handler for each message type. Implemented by default.
  fn handle(
    &self,
    t: String,
    message: MessageFrame,
  ) -> impl std::future::Future<Output = Result<(i64, HandledMessage<Self>)>> {
    async move {
      let res = match t.as_str() {
        "#commit" => {
          let commit = serde_ipld_dagcbor::from_reader(message.body.as_slice())?;
          let (seq, data) = Commit::handle(self, commit).await?;
          (seq, Message::Commit(data))
        }
        "#identity" => {
          let identity = serde_ipld_dagcbor::from_reader(message.body.as_slice())?;
          let (seq, data) = Identity::handle(self, identity).await?;
          (seq, Message::Identity(data))
        }
        "#account" => {
          let account = serde_ipld_dagcbor::from_reader(message.body.as_slice())?;
          let (seq, data) = Account::handle(self, account).await?;
          (seq, Message::Account(data))
        }
        "#handle" => {
          let handle = serde_ipld_dagcbor::from_reader(message.body.as_slice())?;
          let (seq, data) = Handle::handle(self, handle).await?;
          (seq, Message::Handle(data))
        }
        "#migrate" => {
          let migrate = serde_ipld_dagcbor::from_reader(message.body.as_slice())?;
          let (seq, data) = Migrate::handle(self, migrate).await?;
          (seq, Message::Migrate(data))
        }
        "#tombstone" => {
          let tombstone = serde_ipld_dagcbor::from_reader(message.body.as_slice())?;
          let (seq, data) = Tombstone::handle(self, tombstone).await?;
          (seq, Message::Tombstone(data))
        }
        "#info" => {
          let info = serde_ipld_dagcbor::from_reader(message.body.as_slice())?;
          let (seq, data) = Info::handle(self, info).await?;
          (seq, Message::Info(data))
        }
        _ => bail!(anyhow::anyhow!("Unknown message type: {}", t)),
      };

      Ok(res)
    }
  }
}
