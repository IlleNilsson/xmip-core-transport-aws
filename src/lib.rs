#![forbid(unsafe_code)]

//! What every AWS technology speaks over HTTP. Not a transport of its own:
//! aws-s3, aws-sqs, aws-sns and aws-kinesis ride on it, and on the http
//! technology beneath it.
//!
//! ```text
//! sigv4.rs       Signature Version 4, both sides: a Location signs, a
//!                technology's session verifies
//! access_key.rs  the access key ID a Location signs as, and its secret
//! query.rs       the Query API, for aws-sqs and aws-sns
//! json.rs        the JSON 1.1 protocol, for aws-kinesis
//! ```
//!
//! The signer and the Query API lived in the http technology from
//! 2026-09-14, when aws-sqs was found importing aws-s3 and aws-sns importing
//! aws-sqs, and JSON 1.1 in aws-kinesis. The owner ruled on 2026-09-22 that
//! what one vendor speaks leaves HTTP for a crate of that vendor's: http
//! keeps HTTP, and a technology riding on AWS depends on this crate, never
//! on a sibling (ADR-0044).
//!
//! Built with its `declaration` feature alone, the crate is the setting
//! every AWS technology declares, `access_key::ACCESS_KEY_ID`, and none of
//! the code, which is behind the default feature (ADR-0064, amendment
//! 2026-11-26).

#[cfg(feature = "declaration")]
pub mod access_key;
#[cfg(feature = "default")]
pub mod json;
#[cfg(feature = "default")]
pub mod query;
#[cfg(feature = "default")]
pub mod sigv4;
