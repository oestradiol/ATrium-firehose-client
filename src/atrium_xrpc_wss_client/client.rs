//! This file provides a client for the `ATProto` XRPC over WSS protocol.
//! It implements the [`WssClient`] trait for the [`XrpcWssClient`] struct.

use std::str::FromStr;

use futures::Stream;
use tokio::net::TcpStream;

use atrium_xrpc::{
  http::{Request, Uri},
  types::Header,
};
use bon::Builder;
use serde::Serialize;
use tokio_tungstenite::{
  connect_async,
  tungstenite::{self, handshake::client::generate_key},
  MaybeTlsStream, WebSocketStream,
};

use crate::atrium_xrpc_wss::client::{WssClient, XrpcUri};

/// An enum of possible error kinds for this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Invalid uri")]
  InvalidUri,
  #[error("Parsing parameters failed: {0}")]
  ParsingParameters(#[from] serde_html_form::ser::Error),
  #[error("Connection error: {0}")]
  Connection(#[from] tungstenite::Error),
}

#[derive(Builder)]
pub struct XrpcWssClient<'a, P: Serialize> {
  xrpc_uri: XrpcUri<'a>,
  params: Option<P>,
}

type StreamKind = WebSocketStream<MaybeTlsStream<TcpStream>>;
impl<P: Serialize + Send + Sync> WssClient<<StreamKind as Stream>::Item, Error>
  for XrpcWssClient<'_, P>
{
  async fn connect(&self) -> Result<impl Stream<Item = <StreamKind as Stream>::Item>, Error> {
    let Self { xrpc_uri, params } = self;
    let mut uri = xrpc_uri.to_uri();
    //// Query parameters
    if let Some(p) = &params {
      uri.push('?');
      uri += &serde_html_form::to_string(p)?;
    };
    ////

    //// Request
    // Extracting the authority from the URI to set the Host header.
    let uri = Uri::from_str(&uri).map_err(|_| Error::InvalidUri)?;
    let authority = uri.authority().ok_or_else(|| Error::InvalidUri)?.as_str();
    let host = authority
      .find('@')
      .map_or_else(|| authority, |idx| authority.split_at(idx + 1).1);

    // Building the request.
    let mut request = Request::builder()
      .uri(&uri)
      .method("GET")
      .header("Host", host)
      .header("Connection", "Upgrade")
      .header("Upgrade", "websocket")
      .header("Sec-WebSocket-Version", "13")
      .header("Sec-WebSocket-Key", generate_key());

    // Adding the ATProto headers.
    if let Some(proxy) = self.atproto_proxy_header().await {
      request = request.header(Header::AtprotoProxy, proxy);
    }
    if let Some(accept_labelers) = self.atproto_accept_labelers_header().await {
      request = request.header(Header::AtprotoAcceptLabelers, accept_labelers.join(", "));
    }

    // In our case, the only thing that could possibly fail is the URI. The headers are all `String`/`&str`.
    let request = request.body(()).map_err(|_| Error::InvalidUri)?;
    ////

    match connect_async(request).await {
      Ok((stream, _)) => Ok(stream),
      Err(e) => {
        match e {
          tungstenite::Error::Http(response) => {
            // TODO: Handle known HTTP errors provided by the ATProto spec. for the handshake request:
            // 405 Method Not Allowed: Returned to client for non-GET HTTP requests to a stream endpoint.
            // 426 Upgrade Required: Returned to client if Upgrade header is not included in a request to a stream endpoint.
            // 429 Too Many Requests: Frequently used for rate-limiting. Client may try again after a delay. Support for the Retry-After header is encouraged.
            // 500 Internal Server Error: Client may try again after a delay
            // 501 Not Implemented: Service does not implement WebSockets or streams, at least for this endpoint. Client should not try again.
            // 502 Bad Gateway, 503 Service Unavailable, 504 Gateway Timeout: Client may try again after a delay
            eprintln!("HTTP error: {:?}", response);
            todo!()
          }
          _ => Err(e.into()),
        }
      }
    }
  }
}
