//! Confined save-slot persistence for the presentation shell.
//!
//! `mc_core` owns the serialised [`mc_data::save::Save`] model but never performs
//! filesystem I/O. This module is the shell-side boundary: slot paths are
//! confined below the user data root, saves carry the loaded content digest,
//! and replacement is staged so a failed write does not truncate the previous
//! slot.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use mc_core::command::SaveSlot;
use mc_core::world::World;
use mc_data::error::SaveError;
use mc_data::save::Save;

use crate::fsroot::{self, FsError, Root};

/// Errors surfaced by a save-slot operation.
#[derive(Debug)]
pub enum SlotError {
    /// The save could not be encoded or decoded.
    Save(SaveError),
    /// The confined data root or slot path was unavailable.
    Fs(FsError),
    /// A slot file could not be read or its directory could not be created.
    Io(std::io::Error),
    /// The save belongs to a different content pack.
    ContentMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    /// The requested slot does not contain a save.
    EmptySlot(SaveSlot),
    /// A staged replacement could not be completed.
    Replace(std::io::Error),
}

impl std::fmt::Display for SlotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlotError::Save(error) => write!(f, "save error: {error}"),
            SlotError::Fs(error) => write!(f, "filesystem error: {error}"),
            SlotError::Io(error) => write!(f, "save I/O failed: {error}"),
            SlotError::ContentMismatch { expected, actual } => write!(
                f,
                "content digest mismatch: expected {}, got {}",
                hex(expected),
                hex(actual)
            ),
            SlotError::EmptySlot(slot) => write!(f, "save slot {} is empty", slot.0),
            SlotError::Replace(error) => write!(f, "save replacement failed: {error}"),
        }
    }
}

impl std::error::Error for SlotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SlotError::Save(error) => Some(error),
            SlotError::Fs(error) => Some(error),
            SlotError::Io(error) => Some(error),
            SlotError::Replace(error) => Some(error),
            SlotError::ContentMismatch { .. } | SlotError::EmptySlot(_) => None,
        }
    }
}

impl From<SaveError> for SlotError {
    fn from(error: SaveError) -> Self {
        SlotError::Save(error)
    }
}

impl From<FsError> for SlotError {
    fn from(error: FsError) -> Self {
        SlotError::Fs(error)
    }
}

/// The confined save-slot store for one loaded content pack.
#[derive(Clone, Debug)]
pub struct SlotStore {
    data_root: PathBuf,
    content_digest: [u8; 32],
}

impl SlotStore {
    /// Construct a store rooted below `data_root` and tied to `content_digest`.
    pub fn new(data_root: PathBuf, content_digest: [u8; 32]) -> Self {
        Self {
            data_root,
            content_digest,
        }
    }

    /// Save the world into a slot using a staged replacement.
    pub fn save(&self, slot: SaveSlot, world: &World) -> Result<(), SlotError> {
        let started = Instant::now();
        let result = self.save_inner(slot, world);
        crate::obs::record_save_write(started.elapsed());
        result
    }

    fn save_inner(&self, slot: SaveSlot, world: &World) -> Result<(), SlotError> {
        let save_dir = self.save_dir()?;
        let path = self.slot_path(slot)?;
        let temporary = save_dir.join(format!(".slot-{}.sav.{}.tmp", slot.0, std::process::id()));
        let save = Save::new(
            mc_data::save::CURRENT_SCHEMA_VERSION,
            env!("CARGO_PKG_VERSION").to_string(),
            self.content_digest,
            world.clone(),
        )?;
        let bytes = save.to_bytes()?;

        let result = (|| {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(SlotError::Replace)?;
            file.write_all(&bytes).map_err(SlotError::Replace)?;
            file.sync_all().map_err(SlotError::Replace)?;
            replace_file(&temporary, &path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    /// Load and verify a slot against the currently loaded content pack.
    pub fn load(&self, slot: SaveSlot) -> Result<Save, SlotError> {
        let started = Instant::now();
        let result = self.load_inner(slot);
        crate::obs::record_save_load(started.elapsed());
        result
    }

    /// Report whether a slot has a save file without decoding it.
    ///
    /// The file-select screen uses this lightweight status so rendering never
    /// treats a corrupt or content-mismatched save as an empty slot. A later
    /// load still performs the complete integrity and content-digest checks.
    pub fn is_occupied(&self, slot: SaveSlot) -> Result<bool, SlotError> {
        Ok(self.slot_path(slot)?.is_file())
    }

    fn load_inner(&self, slot: SaveSlot) -> Result<Save, SlotError> {
        let path = self.slot_path(slot)?;
        if !path.is_file() {
            return Err(SlotError::EmptySlot(slot));
        }
        let bytes = fsroot::read_confined_to_path(
            &self.data_root,
            Root::Data,
            &PathBuf::from("saves").join(format!("slot-{}.sav", slot.0)),
        )
        .map_err(SlotError::Fs)?;
        let save = Save::load(&bytes)?;
        if save.content_digest != self.content_digest {
            return Err(SlotError::ContentMismatch {
                expected: self.content_digest,
                actual: save.content_digest,
            });
        }
        Ok(save)
    }

    fn save_dir(&self) -> Result<PathBuf, SlotError> {
        let path = fsroot::confine_to_path(&self.data_root, Root::Data, Path::new("saves"))?;
        fs::create_dir_all(&path).map_err(SlotError::Io)?;
        Ok(path)
    }

    fn slot_path(&self, slot: SaveSlot) -> Result<PathBuf, SlotError> {
        let relative = PathBuf::from("saves").join(format!("slot-{}.sav", slot.0));
        Ok(fsroot::confine_to_path(
            &self.data_root,
            Root::Data,
            &relative,
        )?)
    }
}

/// Replace a slot after the staged file has been fully written and synced.
///
/// Windows does not allow `rename` over an existing file. Move the old slot to
/// a same-directory backup first and restore it if the final rename fails.
fn replace_file(temporary: &Path, target: &Path) -> Result<(), SlotError> {
    let backup = target.with_extension(format!("sav.previous.{}", std::process::id()));
    let had_previous = target.exists();
    if had_previous {
        fs::rename(target, &backup).map_err(SlotError::Replace)?;
    }
    match fs::rename(temporary, target) {
        Ok(()) => {
            if had_previous {
                if let Err(error) = fs::remove_file(backup) {
                    // The new slot is already durable; retain that success but
                    // make the cleanup failure observable for operators.
                    tracing::warn!("save-slot backup cleanup failed: {error}");
                }
            }
            Ok(())
        }
        Err(error) => {
            if had_previous {
                if let Err(restore_error) = fs::rename(&backup, target) {
                    return Err(SlotError::Replace(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "replacement failed: {error}; restoring previous save failed: {restore_error}"
                        ),
                    )));
                }
            }
            Err(SlotError::Replace(error))
        }
    }
}

fn hex(bytes: &[u8; 32]) -> String {
    let mut value = String::with_capacity(64);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("writing to a String cannot fail");
    }
    value
}
