MONTE CRISTO 0.1.1
==================

Install
-------
Extract this archive into a new directory owned by your user account. Keep the
binary, content.pack, and content.pack.blake3 together. Run monte-cristo on
Linux or macOS, or monte-cristo.exe on Windows.

Uninstall
---------
Delete the extracted installation directory. Saves, settings, logs, and crash
reports are intentionally retained in the data directory. Delete that data
directory separately only if you no longer need its saves.

Data directories
----------------
Linux:   $XDG_DATA_HOME/monte-cristo, or ~/.local/share/monte-cristo
Windows: %APPDATA%\monte-cristo
macOS:   ~/Library/Application Support/monte-cristo

Integrity and diagnostics
-------------------------
Keep SHA256SUMS beside the downloaded archive and verify it before extraction.
The following commands do not open a window:

  monte-cristo --version
  monte-cristo --verify-content
  monte-cristo --check-paths
  monte-cristo --save-info <save-file>
  monte-cristo --replay <tape-file> --assert-hash

Set MC_CONTENT_DIR to this extracted directory before --verify-content. Save
and tape paths are untrusted input: set MC_DATA_DIR to their trusted parent
directory and pass a path contained by that root. Paths outside the declared
root are rejected.

Crash reports are local-only and are never transmitted. Sharing a crash report,
save, tape, or log is always your choice.
