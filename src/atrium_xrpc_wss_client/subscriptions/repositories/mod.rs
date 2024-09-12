pub mod firehose;
pub mod type_defs;

use async_stream::stream;
use futures::{Stream, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use crate::atrium_xrpc_wss::subscriptions::{
  frames::{self, Frame},
  repositories::{self, Repositories},
  ConnectionHandler, ProcessedPayload, Subscription, SubscriptionError,
};

type WssResult = tokio_tungstenite::tungstenite::Result<Message>;
impl Subscription<WssResult, repositories::Error> for Repositories<WssResult> {
  fn handle_connection<H: ConnectionHandler + Sync>(
    mut connection: impl Stream<Item = WssResult> + Unpin,
    handler: H,
  ) -> impl Stream<Item = Result<ProcessedPayload<H::HandledData>, SubscriptionError<repositories::Error>>>
  {
    // Builds a new async stream that will deserialize the packets sent through the
    // TCP tunnel and then yield the results processed by the handler back to the caller.
    let stream = stream! {
      loop {
        match connection.next().await {
          None => break, // Server dropped connection
          Some(Err(e)) => { // WebSocket error
            // "Invalid framing or invalid DAG-CBOR encoding are hard errors,
            //  and the client should drop the entire connection instead of skipping the frame."
            // https://atproto.com/specs/event-stream

            // TODO: Add tracing crate logging.
            eprintln!("Invalid payload: {e:?}. Dropping connection...");
            yield Err(SubscriptionError::Abort("Received invalid payload".to_string()));
            break;
          }
          Some(Ok(Message::Binary(data))) => {
            match Frame::try_from(data) {
              Ok(Frame::Message { t, data: payload }) => {
                match handler.handle_payload(t, payload).await {
                  Ok(Some(res)) => yield Ok(res), // Payload was successfully handled.
                  Ok(None) => {}, // Payload was ignored by Handler.
                  Err(e) => {
                    // "Invalid framing or invalid DAG-CBOR encoding are hard errors,
                    //  and the client should drop the entire connection instead of skipping the frame."
                    // https://atproto.com/specs/event-stream

                    // TODO: Add tracing crate logging.
                    eprintln!("Invalid payload: {e:?}. Dropping connection...");
                    yield Err(SubscriptionError::Abort("Received invalid payload".to_string()));
                    break;
                  },
                }
              },
              Ok(Frame::Error { error, message }) => {
                match &*error {
                  "FutureCursor" => yield Err(SubscriptionError::Other(repositories::Error::FutureCursor)),
                  "ConsumerTooSlow" => yield Err(SubscriptionError::Other(repositories::Error::ConsumerTooSlow)),
                  _ => yield Err(SubscriptionError::Unknown(format!("Unknown Error Frame. Error: {error}. Message: {message:?}"))),
                }
                break;
              },
              Err(frames::Error::EmptyPayload(ipld)) => {
                // "Invalid framing or invalid DAG-CBOR encoding are hard frames::errors,
                //  and the client should drop the entire connection instead of skipping the frame."
                // https://atproto.com/specs/event-stream

                // TODO: Add tracing crate logging.
                eprintln!("Empty payload for header: {ipld:?}. Dropping connection...");
                yield Err(SubscriptionError::Abort("Received empty payload".to_string()));
                break;
              },
              Err(frames::Error::IpldDecoding(e)) => {
                // "Invalid framing or invalid DAG-CBOR encoding are hard errors,
                //  and the client should drop the entire connection instead of skipping the frame."
                // https://atproto.com/specs/event-stream

                // TODO: Add tracing crate logging.
                eprintln!("Frame was invalid: {e:?}. Dropping connection...");
                yield Err(SubscriptionError::Abort("Received frame was invalid".to_string()));
                break;
              },
              Err(frames::Error::UnknownFrameType(ipld)) => {
                // "Clients should ignore frames with headers that have unknown op or t values.
                //  Unknown fields in both headers and payloads should be ignored."
                // https://atproto.com/specs/event-stream

                // TODO: Add tracing crate logging.
                eprintln!("Unknown frame type: {ipld:?}. Ignoring...");
              },
            }
          }
          _ => {}, // Ignore other message types.
        }
      }
    };

    Box::pin(stream)
  }
}
