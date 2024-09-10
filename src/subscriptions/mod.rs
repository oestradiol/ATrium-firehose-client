pub mod handlers;

use async_stream::stream;
use bon::bon;
use futures::{Stream, StreamExt};
use handlers::{HandledMessage, Handler};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message as WssMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::utils::frames::Frame;

pub trait Subscription {
  fn handle_conn<H: Handler + Sync>(
    connection: WebSocketStream<MaybeTlsStream<TcpStream>>,
    handler: H,
  ) -> impl Stream<Item = Option<(i64, HandledMessage<H>)>>;
}

pub struct Repositories;
#[bon]
impl Repositories {
  #[builder]
  pub fn new<H: Handler + Sync>(
    connection: WebSocketStream<MaybeTlsStream<TcpStream>>,
    handler: H,
  ) -> impl Stream<Item = Option<(i64, HandledMessage<H>)>> {
    Repositories::handle_conn(connection, handler)
  }
}
impl Subscription for Repositories {
  fn handle_conn<H: Handler + Sync>(
    mut connection: WebSocketStream<MaybeTlsStream<TcpStream>>,
    handler: H,
  ) -> impl Stream<Item = Option<(i64, HandledMessage<H>)>> {
    // Builds a new async stream that will deserialize the packets sent through the
    // TCP tunnel and then yield the results processed by the handler back to the caller.
    let stream = stream! {
      while let Some(result) = async {
        if let Some(Ok(WssMessage::Binary(data))) = connection.next().await {
          Some(Frame::try_from(data))
        } else {
          None
        }
      }.await {
        if let Ok(Frame::Message(Some(t), message)) = result {
          match Handler::handle(&handler, t, message).await {
            Ok(res) => yield Some(res),
            Err(_) => {
              // Commented until all the TODOs are done.
              // You should properly handle all the errors here, before yielding.
              // eprintln!("Handler failed! Error: {e:?}");
              yield None;
            }
          }
        }
      }
    };

    Box::pin(stream)
  }
}
