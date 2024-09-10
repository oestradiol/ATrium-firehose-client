mod xprc_uri;

use std::str::FromStr;

use anyhow::anyhow;
use tokio::net::TcpStream;
pub use xprc_uri::XrpcUri;

use atrium_xrpc::{
  http::{Request, Uri},
  types::Header,
};
use bon::bon;
use serde::Serialize;
use tokio_tungstenite::{
  connect_async, tungstenite::handshake::client::generate_key, MaybeTlsStream, WebSocketStream,
};

/// An abstract WSS client.
///
/// # Returns
/// [`anyhow::Result`](anyhow::Result)<[`Subscription`](crate::subscription::Subscription)>
pub trait WssClient<E> {
  /// Send an XRPC request and return the response.
  fn connect(
    &self,
  ) -> impl std::future::Future<Output = Result<WebSocketStream<MaybeTlsStream<TcpStream>>, E>> + Send;

  /// Get the `atproto-proxy` header.
  fn atproto_proxy_header(&self) -> impl std::future::Future<Output = Option<String>> + Send {
    async { None }
  }

  /// Get the `atproto-accept-labelers` header.
  fn atproto_accept_labelers_header(
    &self,
  ) -> impl std::future::Future<Output = Option<Vec<String>>> + Send {
    async { None }
  }
}

pub struct XrpcWssClient<'a, P: Serialize> {
  xrpc_uri: XrpcUri<'a>,
  params: Option<P>,
}
#[bon]
impl<P: Serialize> XrpcWssClient<'_, P> {
  #[builder]
  pub fn new(xrpc_uri: XrpcUri<'__i0>, params: Option<P>) -> Self {
    Self { xrpc_uri, params }
  }
}

impl<P: Serialize + Send + Sync> WssClient<anyhow::Error> for XrpcWssClient<'_, P> {
  async fn connect(&self) -> anyhow::Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let Self { xrpc_uri, params } = self;
    let mut uri = xrpc_uri.to_uri();
    // Query parameters
    if let Some(p) = &params {
      uri.push('?');
      uri += &serde_html_form::to_string(p)?;
    };

    let uri = Uri::from_str(&uri)?;
    let authority = uri
      .authority()
      .ok_or_else(|| anyhow!("Invalid URL: No hostname"))?
      .as_str();
    let host = authority
      .find('@')
      .map(|idx| authority.split_at(idx + 1).1)
      .unwrap_or_else(|| authority);

    // Request
    let mut request = Request::builder()
      .uri(&uri)
      .method("GET")
      .header("Host", host)
      .header("Connection", "Upgrade")
      .header("Upgrade", "websocket")
      .header("Sec-WebSocket-Version", "13")
      .header("Sec-WebSocket-Key", generate_key());

    // Custom Headers
    if let Some(proxy) = self.atproto_proxy_header().await {
      request = request.header(Header::AtprotoProxy, proxy);
    }
    if let Some(accept_labelers) = self.atproto_accept_labelers_header().await {
      request = request.header(Header::AtprotoAcceptLabelers, accept_labelers.join(", "));
    }

    // Connect
    let (stream, _) = connect_async(request.body(())?).await?;
    Ok(stream)
  }
}
