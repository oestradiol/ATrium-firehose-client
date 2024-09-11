//! This file defines the types used in the Firehose handler.

use atrium_api::{
  app::bsky::feed::post::Record,
  types::{string::Did, CidLink},
};

// region: Commit
#[derive(Debug)]
pub struct ProcessedCommitData {
  pub too_big: bool,
  pub repo: Did,
  pub commit: CidLink,
  pub ops: Vec<Operation>,
}
#[derive(Debug)]
pub struct Operation {
  pub action: String,
  pub path: String,
  // Some(Ok(Record)) - the record was found and successfully deserialized
  // Some(Err(Cid)) - the record was found but could not be deserialized
  // None - this operation did not have an associated record
  pub record: Option<Record>,
}
// endregion: Commit

// region: Identity
#[derive(Debug)]
pub struct ProcessedIdentityData {}
// endregion: Identity

// region: Account
#[derive(Debug)]
pub struct ProcessedAccountData {}
// endregion: Account

// region: Handle
#[derive(Debug)]
pub struct ProcessedHandleData {}
// endregion: Handle

// region: Migrate
#[derive(Debug)]
pub struct ProcessedMigrateData {}
// endregion: Migrate

// region: Tombstone
#[derive(Debug)]
pub struct ProcessedTombstoneData {}
// endregion: Tombstone

// region: Info
#[derive(Debug)]
pub struct ProcessedInfoData {}
// endregion: Info
