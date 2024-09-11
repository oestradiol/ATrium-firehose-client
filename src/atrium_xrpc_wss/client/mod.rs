mod xprc_uri;

use futures::Stream;
pub use xprc_uri::XrpcUri;

/// An abstract WSS client.
pub trait WssClient<ConnectionPayload, ConnectionError> {
  /// Send an XRPC request.
  ///
  /// # Returns
  /// [`Result<M, E>`]
  fn connect(
    &self,
  ) -> impl std::future::Future<Output = Result<impl Stream<Item = ConnectionPayload>, ConnectionError>>
       + Send;

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
