//! Deterministic stack archive packing and unpacking.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use flate2::{Compression, GzBuilder, read::GzDecoder};
use stack_model::ArchiveHeader;

use crate::hash::{compute_tree_hash, should_exclude, should_prune};

const HEADER_PATH: &str = "ship.json";

/// Hard ceiling on the uncompressed content a single stack may pack.
///
/// Packing is an in-memory operation — the tar and the gzip output are both
/// `Vec<u8>` — so an unbounded tree is an unbounded allocation. A stack root
/// may sit at a repo root, where a single unpacked directory can exceed
/// available memory. This ceiling fails loud with the offending path rather
/// than allocating until the machine dies.
///
/// A stack is agents, skills, and config: 256 MiB is already generous.
const MAX_STACK_BYTES: u64 = 256 * 1024 * 1024;

/// Pack a stack tree as a deterministic gzip-compressed tar archive.
///
/// The root must contain a `ship.json` carrying the stack identity. Its `hash`
/// is replaced with the canonical tree hash before the header is archived and
/// returned. Archive paths are sorted, timestamps and ownership are zeroed, and
/// permissions are normalized so equal trees produce equal bytes.
pub fn pack_stack(root: &Path) -> anyhow::Result<(Vec<u8>, ArchiveHeader)> {
    let mut header = read_header(&root.join(HEADER_PATH))?;
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
                // Check the size BEFORE reading: the whole point is not to pull
                // an unbounded file into memory to discover it was too big.
                let len = fs::metadata(&full)
                    .with_context(|| format!("stat archive entry {}", full.display()))?
                    .len();
                packed_bytes = packed_bytes.saturating_add(len);
                if packed_bytes > MAX_STACK_BYTES {
                    bail!(
                        "stack at {} exceeds the {} MiB packing limit (hit at {}). \
                         A stack is agents, skills, and config — build output and \
                         dependencies ({}) are never packed. If this is real stack \
                         content, split the stack.",
                        root.display(),
                        MAX_STACK_BYTES / (1024 * 1024),
                        rel.display(),
                        crate::hash::NEVER_PACK_DIRS.join(", "),
                    );
                }
                let bytes = fs::read(&full)
                    .with_context(|| format!("reading archive entry {}", full.display()))?;
                append_bytes(&mut builder, &rel, &bytes, file_mode(&full)?)?;
            }
        }
        builder.finish().context("finishing stack tar")?;
    }

    let mut encoder = GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), Compression::default());
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

    let decoder = GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(dest)
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

fn collect_entries(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .min_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !should_prune(entry.file_name()))
    {
        let entry = entry.context("walking stack tree")?;
        let rel = entry
            .path()
            .strip_prefix(root)
            .context("stripping stack root")?;
        let rel_str = display_path(rel);
        if should_exclude(&rel_str) {
            continue;
        }
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
        let rel = rel.to_path_buf();
        entries.push(rel);
    }
    entries.sort();
    Ok(entries)
}

fn display_path(path: &Path) -> String {
    path.components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
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
