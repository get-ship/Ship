# Installer assets

Art and legal text bundled into the native installers (`.msi`, `.pkg`,
`.deb`, `.rpm`).

## Inventory

| File | Format | Consumer | Status |
|------|--------|----------|--------|
| `ship-512.png` | 512×512 PNG (placeholder: 1×1 transparent) | Source for platform icons | placeholder |
| `LICENSE-INSTALLER.txt` | Plain text | EULA shown by `.msi` / `.pkg` | stub |

cargo-dist v0.28 uses its own default installer chrome; icon assets are
only required once we switch on custom branding. At that point CI will
derive `.icns` (macOS) and `.ico` (Windows) from `ship-512.png` using
`iconutil` and `imagemagick` respectively, and drop them into this
directory during the release job. The source PNG is checked in so the
derivation is deterministic.

Replace the 1×1 placeholder PNG with real art when it lands — no other
change is needed for the installers to pick it up.

## Do NOT re-encode this directory via an automated step

These assets are intentionally small and unchanging. If the installer
build pipeline needs a different format, generate it at CI time from
`ship-512.png` — don't check in derived files here.
