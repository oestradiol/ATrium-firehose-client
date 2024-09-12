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
    XrpcWssClient,
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

  // Build a new XRPC WSS Client then connects to the API.
  let client = XrpcWssClient::builder()
    .xrpc_uri(xrpc_uri.clone())
    .params(params)
    .build();
  let connection = client.connect().await?;

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

        // TODO: Add tracing crate logging.
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
