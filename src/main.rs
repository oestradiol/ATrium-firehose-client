use anyhow::bail;
use atrium_api::com::atproto::sync::subscribe_repos::{self, InfoData};
use firehose_client::{
  atrium_xrpc_wss::{
    client::{WssClient, XrpcUri},
    subscriptions::{
      repositories::{ProcessedData, Repositories},
      ProcessedPayload, SubscriptionError,
    },
  },
  atrium_xrpc_wss_client::{
    subscriptions::repositories::{
      firehose::Firehose,
      type_defs::{Operation, ProcessedCommitData},
    },
    Error, XrpcWssClient,
  },
};
use futures::StreamExt;
use tokio_tungstenite::tungstenite;

/// This example demonstrates how to connect to the ATProto Firehose.
#[tokio::main]
async fn main() {
  // Define the XrpcUri for the subscription.
  let xrpc_uri = XrpcUri::new("bsky.network", subscribe_repos::NSID);

  // Caching the last cursor is important.
  // The API has a backfilling mechanism that allows you to resume from where you stopped.
  let mut last_cursor = Some(1);
  drop(connect(&mut last_cursor, &xrpc_uri).await);
}

/// Connects to `ATProto` to receive real-time data.
async fn connect(
  last_cursor: &mut Option<i64>,
  xrpc_uri: &XrpcUri<'_>,
) -> Result<(), anyhow::Error> {
  // Define the query parameters. In this case, just the cursor.
  let params = subscribe_repos::ParametersData {
    cursor: *last_cursor,
  };

  // Build a new XRPC WSS Client.
  let client = XrpcWssClient::builder()
    .xrpc_uri(xrpc_uri.clone())
    .params(params)
    .build();

  // And then we connect to the API.
  let connection = match client.connect().await {
    Ok(connection) => connection,
    Err(Error::Connection(tungstenite::Error::Http(response))) => {
      // According to the API documentation, the following status codes are expected and should be treated accordingly:
      // 405 Method Not Allowed: Returned to client for non-GET HTTP requests to a stream endpoint.
      // 426 Upgrade Required: Returned to client if Upgrade header is not included in a request to a stream endpoint.
      // 429 Too Many Requests: Frequently used for rate-limiting. Client may try again after a delay. Support for the Retry-After header is encouraged.
      // 500 Internal Server Error: Client may try again after a delay
      // 501 Not Implemented: Service does not implement WebSockets or streams, at least for this endpoint. Client should not try again.
      // 502 Bad Gateway, 503 Service Unavailable, 504 Gateway Timeout: Client may try again after a delay.
      // https://atproto.com/specs/event-stream
      bail!("Status Code was: {response:?}")
    }
    Err(e) => bail!(e),
  };

  // Builds a new subscription from the connection, using handler provided
  // by atrium-xrpc-wss-client, the `Firehose`.
  let mut subscription = Repositories::builder()
    .connection(connection)
    .handler(Firehose)
    .build();

  // Receive payloads by calling `StreamExt::next()`.
  while let Some(payload) = subscription.next().await {
    let data = match payload {
      Ok(ProcessedPayload { seq, data }) => {
        if let Some(seq) = seq {
          *last_cursor = Some(seq);
        }
        data
      }
      Err(SubscriptionError::Abort(reason)) => {
        // This could mean multiple things, all of which are critical errors that require
        // immediate termination of connection.
        eprintln!("Aborted: {reason}");
        *last_cursor = None;
        break;
      }
      Err(e) => {
        // Errors such as `FutureCursor` and `ConsumerTooSlow` can be dealt with here.
        eprintln!("{e:?}");
        *last_cursor = None;
        break;
      }
    };

    match data {
      ProcessedData::Commit(data) => beauty_print_commit(data),
      ProcessedData::Info(InfoData { message, name }) => {
        println!("Received info. Message: {message:?}; Name: {name}.");
      }
      _ => { /* Ignored */ }
    };
  }

  Ok(())
}

fn beauty_print_commit(data: ProcessedCommitData) {
  let ProcessedCommitData {
    repo, commit, ops, ..
  } = data;
  if let Some(ops) = ops {
    for r in ops {
      let Operation {
        action,
        path,
        record,
      } = r;
      let print = format!(
        "\n\n\n#################################  {}  ##################################\n\
        - Repository (User DID): {}\n\
        - Commit CID: {}\n\
        - Path: {path}\n\
        - Flagged as \"too big\"? ",
        action.to_uppercase(),
        repo.as_str(),
        commit.0,
      );
      // Record is only `None` when the commit was flagged as "too big".
      if let Some(record) = record {
        println!(
          "{}No\n\
          //-------------------------------- Record Info -------------------------------//\n\n\
          {:?}",
          print, record
        );
      } else {
        println!(
          "{}Yes\n\
          //---------------------------------------------------------------------------//\n\n",
          print
        );
      }
    }
  }
}
