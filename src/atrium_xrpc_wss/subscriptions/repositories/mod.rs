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

/// An error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("The cursor was in the future.")]
  FutureCursor,
  #[error("The consumer is too slow.")]
  ConsumerTooSlow,
}

/// Defines the builder for any generic `Repositories` struct that implements [`Subscription`](super::Subscription).
#[bon]
impl<ConnectionPayload> Repositories<ConnectionPayload>
where
  Self: Subscription<ConnectionPayload, Error>,
{
  #[builder]
  pub fn new<H: ConnectionHandler + Sync>(
    connection: impl Stream<Item = ConnectionPayload> + Unpin,
    handler: H,
  ) -> impl Stream<Item = Result<ProcessedPayload<H::HandledData>, super::SubscriptionError<Error>>>
  {
    Self::handle_connection(connection, handler)
  }
}
