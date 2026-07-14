//! Deterministic stack archive packing and unpacking.
//!
//! The pack surface is an allow-list: every entry the archive carries is the
//! stack manifest (`stack.config.ts`) or a file under one of the domain dirs
//! in `stack_model::DOMAIN_DIRS`. A file outside that set is not stack content
//! and is never packed. The hash and the archive are derived from the same
//! allow-list, so the `hash` recorded in `ship.json` matches the bytes the
//! install side recomputes from the extracted tree.
//!
//! `ship.json` itself is synthesized at pack time from the `ArchiveHeader`
//! the caller hands in — ship-store does not read it from the source tree
//! (the source has no `ship.json`; the manifest is `stack.config.ts`). The
//! header travels inside the archive as a self-describing transport layer.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use flate2::Compression;
use flate2::write::GzEncoder;
use stack_model::ArchiveHeader;

use crate::hash::compute_tree_hash;

/// Filename of the synthesized archive header — written into every archive at
/// pack time and read back at unpack. Not part of the allow-listed source
/// content the hash covers; it travels alongside as transport.
pub const HEADER_PATH: &str = "ship.json";

/// Filename of the stack manifest — the on-disk marker of a stack root and
/// the only top-level source file on the pack allow-list. The constant
/// mirrors `ship_paths::project::STACK_CONFIG_FILENAME`; duplicated as a
/// literal here to keep ship-store independent of the substrate crate's
/// surface (the file is referenced by string in the archive layout only).
pub const MANIFEST_PATH: &str = "stack.config.ts";

/// Hard ceiling on the uncompressed content a single stack may pack.
///
/// Packing is an in-memory operation — the tar and the gzip output are both
/// `Vec<u8>` — so an unbounded tree is an unbounded allocation. A stack root
/// may sit at a repo root, where a single unpacked directory can exceed
/// available memory. This ceiling fails loud with the offending path rather
/// than allocating until the machine dies.
const MAX_STACK_BYTES: u64 = 256 * 1024 * 1024;

/// Pack a stack tree using the conventional domain dir set.
///
/// `header` carries the publisher-stated identity (`path`, `version`,
/// `commit`, `recommended_*`); its `hash` field is overwritten with the
/// canonical tree hash computed from the allow-listed content. The
/// manifest (`stack.config.ts`) is required at the stack root; absent
/// manifest is a hard error.
pub fn pack_stack(
    root: &Path,
    header: ArchiveHeader,
) -> anyhow::Result<(Vec<u8>, ArchiveHeader)> {
    let mut header = header;
    header.hash = compute_tree_hash(root).context("hashing stack tree")?;
    let header_bytes = serde_json::to_vec_pretty(&header).context("serializing ship.json")?;
    let entries = collect_entries(root)?;

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        builder.mode(tar::HeaderMode::Deterministic);
        append_bytes(&mut builder, Path::new(HEADER_PATH), &header_bytes, 0o644)?;
        let mut packed_bytes: u64 = 0;
        for rel in entries {
            let full = root.join(&rel);
            if full.is_dir() {
                append_dir(&mut builder, &rel)?;
            } else {
                // Check the size BEFORE reading: the whole point is not to
                // pull an unbounded file into memory to discover it was too
                // big.
                let len = fs::metadata(&full)
                    .with_context(|| format!("stat archive entry {}", full.display()))?
                    .len();
                packed_bytes = packed_bytes.saturating_add(len);
                if packed_bytes > MAX_STACK_BYTES {
                    bail!(
                        "stack at {} exceeds the {} MiB packing limit (hit at {}). \
                         A stack is agents, skills, and config — anything else \
                         sitting at the stack root is not stack content and must \
                         be moved out of {}.",
                        root.display(),
                        MAX_STACK_BYTES / (1024 * 1024),
                        rel.display(),
                        root.display()
                    );
                }
                let bytes = fs::read(&full)
                    .with_context(|| format!("reading archive entry {}", full.display()))?;
                append_bytes(&mut builder, &rel, &bytes, file_mode(&full)?)?;
            }
        }
        builder.finish().context("finishing stack tar")?;
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&tar_bytes)
        .context("compressing stack tar")?;
    let bytes = encoder.finish().context("finishing stack gzip")?;
    Ok((bytes, header))
}

/// Unpack a stack archive into `dest` and return its `ship.json` header.
///
/// The destination must be empty. The extracted tree is re-hashed and must
/// match the header before success is returned.
pub fn unpack_stack(bytes: &[u8], dest: &Path) -> anyhow::Result<ArchiveHeader> {

    if dest.exists()
        && fs::read_dir(dest)
            .context("reading archive destination")?
            .next()
            .is_some()
    {
        bail!("archive destination {} is not empty", dest.display());
    }
    fs::create_dir_all(dest)
        .with_context(|| format!("creating archive destination {}", dest.display()))?;
    // Stream-extract with a decompressed-size cap. `tar::Archive::unpack`
    // rejects `..` and absolute paths on its own; the cap is the bomb
    // guard so a malicious archive cannot exhaust disk.
    bundle_limits::extract_tar_capped(bytes, dest, bundle_limits::MAX_BUNDLE_UNPACKED)
        .with_context(|| format!("unpacking stack archive into {}", dest.display()))?;
    let header = read_header(&dest.join(HEADER_PATH))?;
    let actual = compute_tree_hash(dest).context("hashing unpacked stack tree")?;
    if header.hash != actual {
        bail!(
            "archive tree hash mismatch: expected {}, got {actual}",
            header.hash
        );
    }
    Ok(header)
}

fn read_header(path: &Path) -> anyhow::Result<ArchiveHeader> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}

/// Walk `root` and return every entry that is on the pack allow-list:
/// the manifest file at the root, plus every entry under a domain dir
/// (`stack_model::DOMAIN_DIRS`). Symlinks and special files are rejected
/// (a stack archive can only carry files and directories). The synthesized
/// `ship.json` transport header that pack writes to the archive is not on
/// the source tree, so the source walk never sees it — the only path
/// that could observe it is a re-pack of an extracted tree, and the
/// hash already excludes it by virtue of the allow-list (transport
/// is not content).
fn collect_entries(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();

    let manifest = root.join(MANIFEST_PATH);
    if !manifest.is_file() {
        bail!(
            "no {MANIFEST_PATH} found in {} — not a stack root",
            root.display()
        );
    }
    entries.push(PathBuf::from(MANIFEST_PATH));

    for domain in stack_model::DOMAIN_DIRS {
        let dir = root.join(domain);
        if !dir.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir)
            .min_depth(1)
            .follow_links(false)
            .into_iter()
        {
            let entry = entry
                .with_context(|| format!("walking stack domain dir {}", dir.display()))?;
            if entry.file_type().is_symlink() {
                bail!(
                    "stack archive cannot contain symlink {}",
                    entry.path().display()
                );
            }
            if !entry.file_type().is_dir() && !entry.file_type().is_file() {
                bail!(
                    "stack archive cannot contain special file {}",
                    entry.path().display()
                );
            }
            let rel = entry
                .path()
                .strip_prefix(root)
                .context("stripping stack root")?;
            entries.push(rel.to_path_buf());
        }
    }

    entries.sort();
    entries.dedup();
    Ok(entries)
}


fn append_dir(builder: &mut tar::Builder<&mut Vec<u8>>, path: &Path) -> anyhow::Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Directory);
    header.set_size(0);
    header.set_mode(0o755);
    set_common_header(&mut header);
    builder
        .append_data(&mut header, path, std::io::empty())
        .with_context(|| format!("archiving directory {}", path.display()))
}

fn append_bytes(
    builder: &mut tar::Builder<&mut Vec<u8>>,
    path: &Path,
    bytes: &[u8],
    mode: u32,
) -> anyhow::Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(mode);
    set_common_header(&mut header);
    builder
        .append_data(&mut header, path, bytes)
        .with_context(|| format!("archiving file {}", path.display()))
}

fn set_common_header(header: &mut tar::Header) {
    header.set_mtime(0);
    header.set_uid(0);
    header.set_gid(0);
    header
        .set_username("")
        .expect("empty tar username is valid");
    header
        .set_groupname("")
        .expect("empty tar group name is valid");
    header.set_cksum();
}

#[cfg(unix)]
fn file_mode(path: &Path) -> anyhow::Result<u32> {
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::metadata(path)
        .with_context(|| format!("reading metadata for {}", path.display()))?
        .permissions()
        .mode();
    Ok(if mode & 0o111 == 0 { 0o644 } else { 0o755 })
}

#[cfg(not(unix))]
fn file_mode(_path: &Path) -> anyhow::Result<u32> {
    Ok(0o644)
}
