//! Hard ceilings for the install pipeline. One source of truth — every
//! download, decompress, and per-file read is checked against these, so a
//! hostile or compromised registry/repo cannot exhaust memory or disk.
//!
//! The download cap bounds the compressed bytes off the wire; the unpack
//! cap bounds the sum of decompressed entry sizes; the per-file cap bounds
//! any single file slurped whole (a multi-GB member must not be read into
//! a single allocation). `read_capped` and `copy_capped` are the streaming
//! helpers every site uses — they read up to `limit` bytes and fail loud
//! with the limit + bytes-seen when the producer overruns. `extract_tar_capped`
//! is the bomb-resistant unpack primitive that replaces every site that
//! today calls `tar::Archive::unpack` after a `GzDecoder`.

use std::io::{self, Read, Write};

/// Compressed bytes off the wire for a bundle download (CDN tarball, GitHub
/// tarball, sync response, etc.). A stack is agents/skills/config; 64 MiB
/// compressed is already huge.
pub const MAX_BUNDLE_DOWNLOAD: u64 = 64 * 1024 * 1024;

/// Decompressed total for an archive's contents. Matches the pack ceiling
/// — a stack unpacked must not exceed what pack enforces.
pub const MAX_BUNDLE_UNPACKED: u64 = 256 * 1024 * 1024;

/// A single file read in full (per-file hash slurp, per-entry copy). A
/// single multi-GB member must not be read into one allocation.
pub const MAX_BUNDLE_FILE: u64 = 256 * 1024 * 1024;

/// Cap a streaming read at `limit` bytes, returning the collected bytes.
///
/// The producer runs in 16 KiB chunks, so the read never allocates past
/// `limit + 16 KiB` (a single chunk may be appended before the check fires).
/// When the producer overruns, the function returns
/// `Err(BoundsExceeded { limit, read })` naming the ceiling and the bytes
/// seen, so the failure points at the policy not the symptom.
pub fn read_capped<R: Read>(mut reader: R, limit: u64) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut chunk = [0u8; 16 * 1024];
    let mut read: u64 = 0;
    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            return Ok(out);
        }
        read = read.saturating_add(n as u64);
        if read > limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                BoundsExceeded { limit, read },
            ));
        }
        out.extend_from_slice(&chunk[..n]);
    }
}

/// Cap a streaming copy into `dst` at `limit` bytes. Same semantics as
/// [`read_capped`] but for the "copy into an existing writer" shape used
/// by per-file slurps in `compute_tree_hash` and the reqwest body
/// readers.
pub fn copy_capped<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    limit: u64,
) -> io::Result<u64> {
    let mut chunk = [0u8; 16 * 1024];
    let mut read: u64 = 0;
    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            return Ok(read);
        }
        read = read.saturating_add(n as u64);
        if read > limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                BoundsExceeded { limit, read },
            ));
        }
        writer.write_all(&chunk[..n])?;
    }
}

/// Extract a gzip-tarball into `dest`, summing entry sizes against
/// `limit` and aborting mid-stream when the decompressed total crosses
/// the ceiling. The entry-by-entry loop catches gzip bombs before they
/// fill the disk — a 1 MiB tarball that expands to 1 TiB must fail on
/// the first oversized entry, not after the partition is gone.
///
/// `tar` rejects `..` and absolute paths itself, so the unpacked tree
/// is bounded to `dest` (this is the path safety that makes the
/// `unpack`-based sites safe). Returns the total decompressed bytes
/// that landed in `dest`.
pub fn extract_tar_capped<R: Read>(
    reader: R,
    dest: &std::path::Path,
    limit: u64,
) -> io::Result<u64> {
    use flate2::read::GzDecoder;
    let gz = GzDecoder::new(reader);
    let mut archive = tar::Archive::new(gz);
    let mut total: u64 = 0;
    for entry in archive.entries()? {
        let mut entry = entry?;
        // Sum declared entry size first — refuse before writing even one
        // byte. The gzip-bomb signal lives here: a tarball that decodes
        // to many times its compressed size has a header in the GiB range.
        let entry_size = entry.header().entry_size().unwrap_or(0);
        let next = total.saturating_add(entry_size);
        if next > limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "bundle unpack exceeded limit {} bytes at total {} (entry declared {} bytes)",
                    limit, next, entry_size
                ),
            ));
        }
        // Stream the entry through a per-file cap so a sparse file whose
        // tar header lies about its size can't bypass the ceiling.
        if entry.header().entry_type().is_file() {
            // tar::Entry::unpack_in dest is path-safe (rejects `..` and
            // absolute paths); we additionally cap the per-file read at
            // MAX_BUNDLE_FILE by streaming into a temp Vec and writing
            // out from it. The cap-check on the entry Read is the
            // binding check.
            let mut buf = Vec::new();
            let read_total = copy_capped(&mut entry, &mut buf, MAX_BUNDLE_FILE)?;
            let path = entry
                .path()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?
                .into_owned();
            // Reject `..` and absolute paths explicitly (defense in depth
            // — tar already rejects them, but the threat model here is
            // the ZIP-SLIP class).
            for comp in path.components() {
                use std::path::Component;
                match comp {
                    Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("tar entry path escapes destination: {}", path.display()),
                        ));
                    }
                    _ => {}
                }
            }
            let dest_path = dest.join(&path);
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest_path, &buf)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            total = total.saturating_add(read_total);
        } else {
            // Directory, symlink, etc. — let tar unpack in place (it
            // already enforces path safety). The decompressed byte
            // count for non-file entries is zero, so total doesn't
            // move on these.
            entry
                .unpack_in(dest)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        }
    }
    Ok(total)
}

/// The reason a `read_capped` / `copy_capped` call failed: the producer
/// crossed the named ceiling. `read` is the bytes observed (already past
/// `limit` when surfaced) — large enough to be diagnostic but irrelevant
/// to the policy decision.
#[derive(Debug)]
pub struct BoundsExceeded {
    pub limit: u64,
    pub read: u64,
}

impl std::fmt::Display for BoundsExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bundle bound exceeded: read {} bytes, limit {} bytes",
            self.read, self.limit
        )
    }
}

impl std::error::Error for BoundsExceeded {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_capped_returns_whole_body_under_limit() {
        let body = vec![0xAB; 1024];
        let out = read_capped(Cursor::new(body), 2048).unwrap();
        assert_eq!(out.len(), 1024);
    }

    #[test]
    fn read_capped_fails_loud_past_limit() {
        let body = vec![0u8; 4096];
        let err = read_capped(Cursor::new(body), 1024).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("limit 1024"), "msg: {msg}");
        // Read at least up to limit + one chunk before the check fires.
        assert!(msg.contains("read "), "msg: {msg}");
    }

    #[test]
    fn read_capped_exact_limit_succeeds() {
        let body = vec![0u8; 1024];
        let out = read_capped(Cursor::new(body), 1024).unwrap();
        assert_eq!(out.len(), 1024);
    }

    #[test]
    fn copy_capped_copies_under_limit() {
        let body = vec![0u8; 512];
        let mut dst = Vec::new();
        let n = copy_capped(Cursor::new(body), &mut dst, 1024).unwrap();
        assert_eq!(n, 512);
        assert_eq!(dst.len(), 512);
    }

    #[test]
    fn copy_capped_fails_past_limit() {
        let body = vec![0u8; 4096];
        let mut dst = Vec::new();
        let err = copy_capped(Cursor::new(body), &mut dst, 1024).unwrap_err();
        assert!(err.to_string().contains("limit 1024"));
    }

    // ---- extract_tar_capped ----

    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write as IoWrite;

    fn make_tarball_with_files(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut tar_bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);
            builder.mode(tar::HeaderMode::Deterministic);
            for (name, body) in files {
                let mut header = tar::Header::new_gnu();
                header.set_path(name).unwrap();
                header.set_size(body.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                builder.append(&header, *body).unwrap();
            }
            builder.finish().unwrap();
        }
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&tar_bytes).unwrap();
        gz.finish().unwrap()
    }

    #[test]
    fn extract_tar_capped_writes_files_under_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let tar = make_tarball_with_files(&[
            ("a.txt", b"hello"),
            ("b.txt", b"world"),
        ]);
        let total = extract_tar_capped(Cursor::new(tar), tmp.path(), 1024 * 1024).unwrap();
        assert_eq!(total, 10);
        assert_eq!(std::fs::read_to_string(tmp.path().join("a.txt")).unwrap(), "hello");
        assert_eq!(std::fs::read_to_string(tmp.path().join("b.txt")).unwrap(), "world");
    }

    #[test]
    fn extract_tar_capped_rejects_oversized_decompressed() {
        let tmp = tempfile::tempdir().unwrap();
        // 4 MiB of data, but limit is 1 MiB.
        let big = vec![0u8; 4 * 1024 * 1024];
        let tar = make_tarball_with_files(&[("big.bin", &big)]);
        let err = extract_tar_capped(Cursor::new(tar), tmp.path(), 1024 * 1024).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("exceeded limit"), "msg: {msg}");
        // Filesystem must not contain the bomb.
        assert!(!tmp.path().join("big.bin").exists(), "bomb must not land on disk");
    }

    #[test]
    fn extract_tar_capped_rejects_zip_slip_traversal() {
        let tmp = tempfile::tempdir().unwrap();
        // Build a tarball with a `..` traversal entry. The high-level
        // tar::Header::set_path refuses `..`, so write the header bytes
        // directly — this is exactly what an attacker with a tarball
        // library would do.
        let mut tar_bytes = Vec::new();
        let body = b"pwned";
        let mut header = tar::Header::new_gnu();
        header.set_size(body.len() as u64);
        header.set_mode(0o644);
        // Build path bytes manually: "../escaped.txt" (octal null-padded
        // to 100 bytes for the ustar name field).
        let path_bytes = b"../escaped.txt";
        let mut path_field = [0u8; 100];
        path_field[..path_bytes.len()].copy_from_slice(path_bytes);
        header.set_entry_type(tar::EntryType::Regular);
        // The Header API refuses to compute a cksum for a path with `..`,
        // so write the whole header block by hand. Layout: 100-byte
        // name, then the rest of the standard ustar fields populated
        // by `new_gnu` (mode, uid, gid, size, mtime, cksum placeholder,
        // typeflag, magic, version). Easier: use `into_inner` and
        // patch the name field bytes directly.
        let raw = header.as_bytes();
        let mut raw = raw.to_vec();
        raw[..100].copy_from_slice(&path_field);
        // Recompute the checksum. The cksum field is bytes 148..155
        // (8 bytes); when tar verifies, it treats those bytes as
        // spaces (0x20), so the sum is over the patched header with
        // the cksum field zeroed.
        for b in raw[148..156].iter_mut() {
            *b = 0x20;
        }
        let sum: u32 = raw[..512].iter().map(|b| *b as u32).sum();
        let sum_str = format!("{:06o}\0 ", sum);
        raw[148..156].copy_from_slice(sum_str.as_bytes());
        tar_bytes.extend_from_slice(&raw);
        // Body: 512-byte block, padded.
        let mut block = [0u8; 512];
        block[..body.len()].copy_from_slice(body);
        tar_bytes.extend_from_slice(&block);
        // Two zero blocks terminate the tar.
        tar_bytes.extend_from_slice(&[0u8; 1024]);
        // Gzip it.
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&tar_bytes).unwrap();
        let tar = gz.finish().unwrap();
        let err = extract_tar_capped(Cursor::new(tar), tmp.path(), 1024 * 1024).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("escapes destination") || msg.contains("parent") || msg.contains(".."),
            "msg: {msg}"
        );
    }
}
