use atrium_api::com::atproto::sync::subscribe_repos;
use firehose_client::{
  atrium_xrpc_wss::{
    client::{WssClient, XrpcUri},
    subscriptions::{
      repositories::{Error, ProcessedData, Repositories},
      SubscriptionError,
    },
  },
  atrium_xrpc_wss_client::{
    client::XrpcWssClient,
    subscriptions::repositories::{
      firehose::Firehose,
      type_defs::{Operation, ProcessedCommitData},
    },
  },
};
use futures::StreamExt;

/// This example demonstrates how to connect to the ATProto Firehose.
#[tokio::main]
async fn main() {
  // Define the XrpcUri for the subscription.
  let xrpc_uri = XrpcUri::new("bsky.network", subscribe_repos::NSID);

  // Caching the last cursor is important.
  // The API has a backfilling mechanism that allows you to resume from where you stopped.
  let mut last_cursor = None;
  loop {
    // Loop to reconnect every time the connection is lost.
    // Do not do this in production. This is just for demonstration purposes.
    // Instead, you should handle the error variant
    if let Err(e) = connect(&mut last_cursor, &xrpc_uri).await {
      eprintln!("Error: {e:?}");
    }
  }
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

  // Build a new XRPC WSS Client then connects to the API.
  let client = XrpcWssClient::builder()
    .xrpc_uri(xrpc_uri.clone())
    .params(params)
    .build();
  let connection = client.connect().await?;

  // Builds a new subscription from the connection;
  let mut subscription = Repositories::builder()
    .connection(connection)
    .handler(Firehose) // Using the implemented `Firehose` handler.
    .build();

  // Receive payloads by calling `StreamExt::next()`.
  while let Some(payload) = subscription.next().await {
    let data = match payload {
      Ok(payload) => {
        *last_cursor = Some(payload.seq);
        payload.data
      }
      Err(SubscriptionError::Abort(err)) => {
        // TODO: Add tracing crate logging.
        eprintln!("Aborted: {err}");
        *last_cursor = None;
        break;
      }
      Err(SubscriptionError::Other(Error::FutureCursor)) => {
        eprintln!("The cursor was in the future.");
        *last_cursor = None;
        break;
      }
      Err(SubscriptionError::Other(Error::ConsumerTooSlow)) => {
        eprintln!("The consumer could not keep up.");
        *last_cursor = None;
        break;
      }
    };

    if let ProcessedData::Commit(ProcessedCommitData { repo, commit, ops }) = data {
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
  }

  Ok(())
}
