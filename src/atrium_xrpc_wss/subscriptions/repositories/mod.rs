use bon::bon;
use futures::Stream;
use std::marker::PhantomData;

use super::{ConnectionHandler, ProcessedPayload, Subscription};

mod handler;
pub use handler::{HandledData, Handler, ProcessedData};

/// A struct that represents the repositories subscription, used in `com.atproto.sync.subscribeRepos`.
pub struct Repositories<ConnectionPayload> {
  /// This is only here to constrain the `ConnectionPayload` used in [`Subscription`], or else we get a compile error.
  _payload_kind: PhantomData<ConnectionPayload>,
}
/// Defines the builder for any generic `Repositories` struct.
#[bon]
impl<ConnectionPayload> Repositories<ConnectionPayload>
where
  Self: Subscription<ConnectionPayload>,
{
  #[builder]
  pub fn new<H: ConnectionHandler + Sync>(
    connection: impl Stream<Item = ConnectionPayload> + Unpin,
    handler: H,
  ) -> impl Stream<Item = ProcessedPayload<H::HandledData>> {
    Self::handle_connection(connection, handler)
  }
}
