//! The access key ID a Location signs as, and the secret access key the
//! secret it names holds (ADR-0019, amendment 2026-10-29; ADR-0015,
//! amendment 2026-11-25): declared and held once for every AWS technology.
//! The secret is a `password` secret, kept in the Administration Database
//! and read at each request (`transport::secured::presented`), since
//! Signature Version 4 signs every request.

use transport::Secured;
use transport::error::Result;
use xcore::settings::{Applies, Kind, Presence, Setting};

/// The setting a Location names the access key ID it signs as by, among
/// each AWS technology's `Configured::SETTINGS`.
pub const ACCESS_KEY_ID: Setting = Setting {
    name: "access_key_id",
    kind: Kind::Text,
    presence: Presence::Optional,
    meaning: "The access key ID requests are signed as, with the secret access key the \
              Location's secret holds: AKIAIOSFODNN7EXAMPLE.",
    applies: Applies::Both,
};

/// `secured`, the secret a Location names, kept where `access_key` is the
/// access key ID it signs as: Signature Version 4 names the access key ID
/// beside every signature, so a Location naming a secret and no
/// `access_key_id` is refused as the node starts
/// (`transport::secured::without_holder`), as MQTT refuses a password with
/// no user name.
///
/// # Errors
/// Permanent, in words, where `access_key` is empty.
pub fn signing_as(access_key: &str, secured: Secured) -> Result<Secured> {
    if access_key.is_empty() {
        return Err(transport::secured::without_holder(
            &secured,
            "the secret access key of the access key ID",
            ACCESS_KEY_ID.name,
        ));
    }
    Ok(secured)
}
