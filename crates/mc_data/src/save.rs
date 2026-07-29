//! Save/load for game state snapshots.
//!
//! M6: A `Save` wraps a [`World`] together with a `schema_version`,
//! `product_version`, `content_digest`, and an integrity `digest` over the
//! other fields. The on-disk encoding uses postcard for the metadata + world
//! and appends the Blake3 digest as the final 32 bytes.

use std::fs;
use std::path::Path;

use mc_core::world::World;
use serde::{Deserialize, Serialize};

use crate::error::SaveError;

/// Current schema version for save files.
pub const CURRENT_SCHEMA_VERSION: u16 = 2;

/// Maximum allowed schema version (0–256).
pub const MAX_SCHEMA_VERSION: u16 = 256;

/// Maximum length (in characters) for `product_version`.
pub const MAX_PRODUCT_VERSION_LEN: usize = 128;

/// A game-state save with content-addressing integrity.
///
/// Fields (in postcard order):
///  1. `schema_version`   — format version (currently 2)
///  2. `product_version`  — human-readable product version string
///  3. `content_digest`   — blake3 hash of the baked content pack
///  4. `world`            — serialised [`World`]
///  5. `digest`           — blake3 hash of fields 1–4 (appended after postcard)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Save {
    pub schema_version: u16,
    pub product_version: String,
    pub content_digest: [u8; 32],
    pub world: World,
    pub digest: [u8; 32],
}

impl Save {
    /// Create a new `Save` with bounds-checked fields.
    ///
    /// The integrity `digest` is computed automatically over the other four fields.
    pub fn new(
        version: u16,
        product: String,
        content_digest: [u8; 32],
        world: World,
    ) -> Result<Self, SaveError> {
        if version > MAX_SCHEMA_VERSION {
            return Err(SaveError::Deserialize(format!(
                "schema_version {version} exceeds max {MAX_SCHEMA_VERSION}",
            )));
        }
        if product.len() > MAX_PRODUCT_VERSION_LEN {
            return Err(SaveError::Deserialize(format!(
                "product_version length {} exceeds max {MAX_PRODUCT_VERSION_LEN}",
                product.len(),
            )));
        }
        let mut s = Save {
            schema_version: version,
            product_version: product,
            content_digest,
            world,
            digest: [0u8; 32],
        };
        s.digest = s.compute_digest()?;
        Ok(s)
    }

    /// Canonical binary encoding.
    ///
    /// Layout: `postcard(schema_version, product_version, content_digest, world)` followed
    /// by the 32-byte Blake3 digest of that segment.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SaveError> {
        let mut data = self.encode_body()?;
        data.extend_from_slice(&self.digest);
        Ok(data)
    }

    /// Deserialise a `Save` from bytes with full integrity verification.
    ///
    /// Strict load order:
    ///  1. Split off the trailing 32-byte digest.
    ///  2. Decode the body with postcard.
    ///  3. Reject saves with `schema_version > CURRENT_SCHEMA_VERSION`.
    ///  4. Check bounds: max 256 for version, max 128 chars for product.
    ///  5. Verify the stored digest matches a recomputation over the body.
    ///  6. Return the decoded `Save`.
    pub fn load(data: &[u8]) -> Result<Self, SaveError> {
        if data.len() < 32 {
            return Err(SaveError::Deserialize(
                "save data too short (missing trailing digest)".into(),
            ));
        }
        let (body, stored_digest_slice) = data.split_at(data.len() - 32);
        let stored_digest: [u8; 32] = stored_digest_slice
            .try_into()
            .map_err(|_| SaveError::Deserialize("trailing digest is not 32 bytes".into()))?;

        // 1. Decode the body
        let (schema_version, product_version, content_digest, world): (
            u16,
            String,
            [u8; 32],
            World,
        ) = postcard::from_bytes(body)
            .map_err(|e| SaveError::Deserialize(format!("postcard decode failed: {e}")))?;

        // 2. Version check – refuse newer
        if schema_version > CURRENT_SCHEMA_VERSION {
            return Err(SaveError::Deserialize(format!(
                "save schema version {schema_version} is newer than supported {CURRENT_SCHEMA_VERSION}"
            )));
        }

        // 3. Bounds
        if schema_version > MAX_SCHEMA_VERSION {
            return Err(SaveError::Deserialize(format!(
                "schema_version {schema_version} exceeds max {MAX_SCHEMA_VERSION}"
            )));
        }
        if product_version.len() > MAX_PRODUCT_VERSION_LEN {
            return Err(SaveError::Deserialize(format!(
                "product_version length {} exceeds max {MAX_PRODUCT_VERSION_LEN}",
                product_version.len(),
            )));
        }

        // 4. Verify digest
        let actual_hash = blake3::hash(body);
        let actual_digest = *actual_hash.as_bytes();
        if actual_digest != stored_digest {
            return Err(SaveError::DigestMismatch {
                expected: hex_str(&stored_digest),
                actual: hex_str(&actual_digest),
            });
        }

        // 5. Basic sanity check on content_digest (non-zero)
        if content_digest == [0u8; 32] {
            return Err(SaveError::Deserialize("content_digest is all zeros".into()));
        }

        Ok(Save {
            schema_version,
            product_version,
            content_digest,
            world,
            digest: stored_digest,
        })
    }

    /// Write the canonical encoding to `path`.
    pub fn to_file(&self, path: &Path) -> Result<(), SaveError> {
        let bytes = self.to_bytes()?;
        fs::write(path, &bytes)?;
        Ok(())
    }

    /// Read and verify a save from `path`.
    pub fn from_file(path: &Path) -> Result<Self, SaveError> {
        let data = fs::read(path)?;
        Self::load(&data)
    }

    // ── private helpers ──────────────────────────────────────────────────

    /// Postcard-encode the four data fields (everything *except* the digest).
    fn encode_body(&self) -> Result<Vec<u8>, SaveError> {
        postcard::to_stdvec(&(
            self.schema_version,
            &self.product_version,
            self.content_digest,
            &self.world,
        ))
        .map_err(|e| SaveError::Deserialize(format!("serialization failed: {e}")))
    }

    /// Compute the Blake3 digest over the body.
    fn compute_digest(&self) -> Result<[u8; 32], SaveError> {
        let body = self.encode_body()?;
        Ok(*blake3::hash(&body).as_bytes())
    }
}

/// Helper: format a 32-byte digest as a lowercase hex string.
fn hex_str(bytes: &[u8; 32]) -> String {
    blake3::Hash::from(*bytes).to_hex().to_string()
}
