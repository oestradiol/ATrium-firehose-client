//! This file defines the types used in the Firehose handler.

use atrium_api::{
  record::KnownRecord,
  types::{string::Did, CidLink},
};

// region: Commit
#[derive(Debug)]
pub struct ProcessedCommitData {
  pub repo: Did,
  pub commit: CidLink,
  // `ops` can be `None` if the commit is marked as `too_big`.
  pub ops: Option<Vec<Operation>>,
}
#[derive(Debug)]
pub struct Operation {
  pub action: String,
  pub path: String,
  pub record: Option<KnownRecord>,
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
