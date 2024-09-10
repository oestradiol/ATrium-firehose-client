use atrium_api::com::atproto::sync::subscribe_repos;
use chrono::Local;
use firehose_client::{
  firehose::{handlers::commit, Firehose},
  subscriptions::{self, handlers::Message},
  wss_client::{WssClient, XrpcUri, XrpcWssClient},
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
    if let Err(e) = connect(&mut last_cursor, &xrpc_uri).await {
      eprintln!("Error: {e:?}");
    }
  }
}

/// Connects to `ATProto` to receive real-time messages.
async fn connect(
  last_cursor: &mut Option<i64>,
  xrpc_uri: &XrpcUri<'_>,
) -> Result<(), anyhow::Error> {
  // Define the query parameters. In this case, just the cursor.
  let params = subscribe_repos::ParametersData {
    cursor: *last_cursor,
  };

  // Build a new WSS XRPC Client then connects to the API.
  let connection = XrpcWssClient::builder()
    .xrpc_uri(xrpc_uri.clone())
    .params(params)
    .build()
    .connect()
    .await?;

  // Builds a new subscription from the connection;
  let mut subscription = subscriptions::Repositories::builder()
    .connection(connection)
    .handler(Firehose) // Using the implemented `Firehose` handler.
    .build();

  // Receive messages by calling `StreamExt::next()`.
  while let Some(message) = subscription.next().await {
    if let Some((cursor, message)) = message {
      *last_cursor = Some(cursor);

      match message {
        Message::Commit(c) => {
          let commit::ProcessedData {
            too_big,
            repo,
            commit,
            ops,
          } = c;

          for r in ops {
            let commit::Operation {
              action,
              path,
              record,
            } = r;
            if let Some(record) = record {
              println!(
                "
                \n\n################  {} @ {}  ################\n\
                - Repository (User DID): {}\n\
                - Path: {path}\n\
                - Commit CID: {}\n\
                - Flagged as \"too big\"? {too_big}\n\
                //-----------------------------------------------------------------------//\n\n\
                {}
                ",
                action.to_uppercase(),
                record.created_at.as_ref().with_timezone(&Local),
                repo.as_str(),
                commit.0,
                record.text
              );
            } else {
              println!(
                "
                \n\n#################################  {}  ##################################\n\
                - Repository (User DID): {}\n\
                - Path: {path}\n\
                - Commit CID: {}\n\
                - Flagged as \"too big\"? {too_big}\n\
                //-----------------------------------------------------------------------//\n\n\
                ",
                action.to_uppercase(),
                repo.as_str(),
                commit.0
              );
            }
          }
        }
        _ => {}
      }
    }
  }

  Ok(())
}
