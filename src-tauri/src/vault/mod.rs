//! Vault integration: vector index over existing notes + atomic writer for
//! synthesized append/create operations.

pub mod chunk;
pub mod index;
pub mod watcher;
pub mod writer;
