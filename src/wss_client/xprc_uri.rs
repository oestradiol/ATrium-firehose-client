#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XrpcUri<'a> {
  base_uri: &'a str,
  nsid: &'a str,
}
impl<'a> XrpcUri<'a> {
  pub fn new(base_uri: &'a str, nsid: &'a str) -> Self {
    Self { base_uri, nsid }
  }

  pub fn to_uri(&self) -> String {
    let XrpcUri { base_uri, nsid } = self;
    format!("wss://{}/xrpc/{}", base_uri, nsid)
  }
}
