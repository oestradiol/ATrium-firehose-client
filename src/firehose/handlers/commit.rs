use anyhow::Result;
use atrium_api::{
  app::bsky::feed::post::Record,
  com::atproto::sync::subscribe_repos::{Commit as Message, CommitData, RepoOpData},
  types::{string::Did, CidLink, Object},
};
use futures::io::Cursor as FutCursor;
use ipld_core::cid::Cid;
use std::{collections::BTreeMap, io::Cursor};

use crate::subscriptions::handlers::Commit;

use super::super::Firehose;

#[derive(Debug)]
pub struct ProcessedData {
  pub too_big: bool,
  pub repo: Did,
  pub commit: CidLink,
  pub ops: Vec<Operation>,
}
#[derive(Debug)]
pub struct Operation {
  pub action: String,
  pub path: String,
  pub record: Option<Record>,
}

impl Commit for Firehose {
  type ProcessedData = self::ProcessedData;

  /// Handles a message of type `#commit`.
  async fn handle(&self, message: Message) -> Result<(i64, Self::ProcessedData)> {
    let Object {
      data:
        CommitData {
          seq,
          too_big,
          repo,
          commit,
          ops,
          blocks,
          ..
        },
      ..
    } = message;

    // We read all the blocks from the CAR file and store them in a map
    // so that we can look up the data for each operation by its CID.
    let mut cursor = FutCursor::new(blocks);
    let mut map = rs_car::car_read_all(&mut cursor, true)
      .await?
      .0
      .into_iter()
      .map(compat_cid)
      .collect::<BTreeMap<_, _>>();

    // Processing each operation
    let ops = process_ops(ops, &mut map);

    Ok((
      seq,
      ProcessedData {
        too_big,
        repo,
        commit,
        ops,
      },
    ))
  }
}

// Transmute is here because the version of the rs_car crate for cid is 0.10.1 whereas
// the ilpd_core crate is 0.11.1. Should work regardless, given that the Cid type's
// memory layout was not changed between the two versions. Temporary fix.
// TODO: Find a way to fix the version compatibility issue.
fn compat_cid((cid, item): (rs_car::Cid, Vec<u8>)) -> (ipld_core::cid::Cid, Vec<u8>) {
  (unsafe { std::mem::transmute::<_, Cid>(cid) }, item)
}

fn process_ops(ops: Vec<Object<RepoOpData>>, map: &mut BTreeMap<Cid, Vec<u8>>) -> Vec<Operation> {
  ops.into_iter().map(|op| process_op(map, op)).collect()
}

/// Processes a single operation.
fn process_op(map: &mut BTreeMap<Cid, Vec<u8>>, op: Object<RepoOpData>) -> Operation {
  let Object {
    data: RepoOpData { action, path, cid },
    ..
  } = op;

  // Finds in the map the `Record` with the operation's CID and deserializes it.
  // If the item is not found, returns `None`.
  let record = cid.and_then(|c| map.get_mut(&c.0)).map_or_else(
    || None,
    |item| serde_ipld_dagcbor::from_reader::<Record, _>(Cursor::new(item)).ok(),
  );

  Operation {
    action,
    path,
    record,
  }
}
