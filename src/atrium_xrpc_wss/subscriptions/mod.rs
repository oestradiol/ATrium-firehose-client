pub mod frames;
pub mod repositories;

use futures::Stream;

/// A trait that defines the connection handler.
pub trait ConnectionHandler {
  /// The [`Self::HandledData`](ConnectionHandler::HandledData) type should be used to define the returned processed data type.
  type HandledData;
  /// The [`Self::HandlingError`](ConnectionHandler::HandlingError) type should be used to define the processing error type.
  type HandlingError: 'static + Send + Sync + std::fmt::Debug;

  /// Handles binary data coming from the connection. This function will deserialize the payload body and call the appropriate
  /// handler for each payload type.
  ///
  /// # Returns
  /// [`Result<Option<T>>`] like:
  /// - `Ok(Some(processedPayload))` where `processedPayload` is [`ProcessedPayload<ConnectionHandler::HandledData>`](ProcessedPayload)
  ///   if the payload was successfully processed.
  ///
  /// - `Ok(None)` if the payload was ignored.
  ///
  /// - `Err(e)` where `e` is [`ConnectionHandler::HandlingPayloadError`] if an error occurred while processing the payload.
  fn handle_payload(
    &self,
    t: String,
    payload: Vec<u8>,
  ) -> impl std::future::Future<
    Output = Result<Option<ProcessedPayload<Self::HandledData>>, Self::HandlingError>,
  >;
}

/// A trait that defines a subscription.
/// It should be implemented by any struct that wants to handle a connection.
/// The `ConnectionPayload` type parameter is the type of the payloads that will be received through the connection stream.
pub trait Subscription<ConnectionPayload> {
  /// The `handle_connection` method should be implemented to handle the connection.
  ///
  /// # Returns
  /// A stream of processed payloads.
  fn handle_connection<H: ConnectionHandler + Sync>(
    connection: impl Stream<Item = ConnectionPayload> + Unpin,
    handler: H,
  ) -> impl Stream<Item = ProcessedPayload<H::HandledData>>;
}

/// This struct represents a processed payloads.
/// It contains the sequence number (cursor) and the final processed data.
pub struct ProcessedPayload<Kind> {
  pub seq: i64,
  pub data: Kind,
}

/// Helper function to convert between payload kinds.
impl<Kind> ProcessedPayload<Kind> {
  pub fn map<NewKind, F: FnOnce(Kind) -> NewKind>(self, f: F) -> ProcessedPayload<NewKind> {
    ProcessedPayload {
      seq: self.seq,
      data: f(self.data),
    }
  }
}
