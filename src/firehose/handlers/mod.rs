use crate::subscriptions::handlers::Handler;

use super::Firehose;

pub mod account;
pub mod commit;
pub mod handle;
pub mod identity;
pub mod info;
pub mod migrate;
pub mod tombstone;

impl Handler for Firehose {}
