//! `deck scaffold view|component` — generate boilerplate for a new SUI view
//! or primitive. Ported verbatim from the retired
//! `scripts/scaffold-view.mjs` + `scripts/scaffold-sui-component.mjs`.
//!
//! Scaffolding is a deck capability because deck owns the app/component
//! shape (manifest fields, paired-controller convention, the `.sui` +
//! `.controller.ts` stem pair). The retired .mjs called into `node` and
//! pointed at since-renamed paths (`packages/ui-components/src/`); the
//! port writes to the current layout (`packages/components/primitives/`)
//! and the next-steps text reflects today's toolchain.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

/// Validate `name` against `/^[a-z][a-z0-9-]*$/` — the route-segment shape
/// the daemon resolver expects and the file-system shape we write.
fn validate_view_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("view name is required");
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_lowercase() {
        bail!("view name must start with a lowercase letter — got {name:?}");
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        bail!("view name must match /^[a-z][a-z0-9-]*$/ — got {name:?}");
    }
    Ok(())
}

/// Same shape as `validate_view_name` plus the `sui-` prefix the runtime
/// custom-element registry expects for primitives.
fn validate_component_tag(tag: &str) -> Result<()> {
    if !tag.starts_with("sui-") {
        bail!("component tag must start with `sui-` — got {tag:?}");
    }
    let rest = &tag[4..];
    if rest.is_empty()
        || !rest.starts_with(|c: char| c.is_ascii_lowercase())
        || !rest.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("component tag must match /^sui-[a-z][a-z0-9-]*$/ — got {tag:?}");
    }
    Ok(())
}

/// Titlecase `hello-world` → `Hello World` for default view titles.
fn title_case(name: &str) -> String {
    name.split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Resolve the Cargo workspace root from `start`. Lets `deck scaffold` work
/// from any cwd inside the repo; the views/ + deck-core monorepo markers
/// surface naturally when the scaffold tries to write into those paths.
fn find_repo_root(start: &Path) -> Result<PathBuf> {
    ship_substrate::locate::workspace_root_from(start).ok_or_else(|| {
        anyhow!("no Cargo workspace root above {}", start.display())
    })
}

pub fn run_view(name: String, route_path: Option<String>, component_tag: Option<String>) -> Result<()> {
    validate_view_name(&name)?;
    let route = route_path.unwrap_or_else(|| format!("/{name}"));
    if !route.starts_with('/') {
        bail!("--path must start with `/` — got {route:?}");
    }
    if let Some(tag) = component_tag.as_deref() {
        let prefix = format!("{name}-");
        if !tag.starts_with(&prefix) {
            bail!(
                "--component {tag:?} must be prefixed with view name `{name}-` to avoid registry collisions"
            );
        }
    }

    let repo_root = find_repo_root(&std::env::current_dir().context("cwd")?)?;
    let base = repo_root.join("views").join(&name);
    if base.exists() {
        bail!("directory already exists: {}", base.display());
    }
    fs::create_dir_all(&base).with_context(|| format!("mkdir {}", base.display()))?;

    let title = title_case(&name);
    let manifest = format!(
        "// Route metadata for views/{name}/. See\n\
         // docs/ship-adrs/references/docs/view-directory-convention.md.\n\
         {{\n  \
         \"name\": \"{name}\",\n  \
         \"path\": \"{route}\",\n  \
         \"title\": \"{title}\",\n  \
         \"layout\": \"default\",\n  \
         \"auth\": \"public\"\n\
         }}\n"
    );
    let page = format!(
        "<state name=\"ready\" value=\"true\" />\n\n<div class=\"{name}\">\n  \
         <h1>{title}</h1>\n  <p>New view — replace this body.</p>\n</div>\n"
    );
    let controller = format!(
        "// {name} view — tier-1 stub. Replace with state, derived, hooks, lifecycle as\n\
         // the view grows. See .ship/skills/ship-views/SKILL.md.\n\nexport default null\n"
    );
    let style = format!(
        ".{name} {{\n  max-width: 36rem;\n  margin: 4rem auto;\n  \
         font-family: system-ui, sans-serif;\n}}\n\
         .{name} h1 {{ margin: 0 0 0.5rem; font-size: 1.5rem; }}\n"
    );
    fs::write(base.join("manifest.jsonc"), manifest)?;
    fs::write(base.join("page.sui"), page)?;
    fs::write(base.join("controller.ts"), controller)?;
    fs::write(base.join("style.css"), style)?;

    if let Some(tag) = component_tag {
        let tag_dir = base.join("_components").join(&tag);
        fs::create_dir_all(&tag_dir)?;
        let comp_sui = format!(
            "<!--\n  \
             {tag} — view-private widget for views/{name}/.\n\n  \
             Tag prefix matches the parent view name so the runtime can register it\n  \
             without colliding with any other view's _components/ tree.\n\
             -->\n\n\
             <prop name=\"tone\" values=\"default|accent\" default=\"default\" reflect />\n\n\
             <style>\n  :host {{ display: block; }}\n  \
             .root {{ padding: 1rem; border: 1px solid var(--border, #d0d7de); border-radius: 0.5rem; }}\n\
             </style>\n\n\
             <div class=\"root\" part=\"root\"><slot></slot></div>\n"
        );
        let comp_ctrl = format!(
            "// {tag} — view-private widget controller. Tier-1 stub.\n\nexport default null\n"
        );
        fs::write(tag_dir.join(format!("{tag}.sui")), comp_sui)?;
        fs::write(tag_dir.join(format!("{tag}.controller.ts")), comp_ctrl)?;
    }

    println!("[scaffold] created views/{name}/");
    println!("[scaffold]   page.sui  controller.ts  style.css  manifest.jsonc");
    println!("[scaffold] next steps:");
    println!("  1. Edit page.sui — replace placeholder body");
    println!("  2. Edit manifest.jsonc — set rail/preload/events as needed");
    println!("  3. Reload the shell — views auto-mount via the daemon");
    Ok(())
}

pub fn run_component(tag: String, want_controller: bool) -> Result<()> {
    validate_component_tag(&tag)?;
    let repo_root = find_repo_root(&std::env::current_dir().context("cwd")?)?;
    // Primitives live under packages/components/primitives/src/ today — the
    // retired .mjs wrote to packages/ui-components/src/ which no longer exists.
    let base = repo_root.join("packages/components/primitives/src").join(&tag);
    if base.exists() {
        bail!("directory already exists: {}", base.display());
    }
    fs::create_dir_all(&base)?;

    let sui = format!(
        "<!--\n  \
         {tag} — SUI primitive.\n\n  \
         Parity vs Lit <ship-{stem}>:\n    \
         Props:    (list)\n    Slots:    (list)\n    Events:   (list)\n    \
         Methods:  (list)\n    ARIA:     (list)\n\
         -->\n\n\
         <prop name=\"tone\" values=\"default|primary|info|success|warning|destructive|accent|muted\" default=\"default\" reflect />\n\n\
         <style>\n  \
         :host {{ display: inline-flex; vertical-align: middle; --tone-fg: var(--foreground); }}\n  \
         :host([tone='primary'])     {{ --tone-fg: var(--primary); }}\n  \
         :host([tone='info'])        {{ --tone-fg: var(--info); }}\n  \
         :host([tone='success'])     {{ --tone-fg: var(--success); }}\n  \
         :host([tone='warning'])     {{ --tone-fg: var(--warning); }}\n  \
         :host([tone='destructive']) {{ --tone-fg: var(--destructive); }}\n  \
         :host([tone='accent'])      {{ --tone-fg: var(--accent); }}\n  \
         :host([tone='muted'])       {{ --tone-fg: var(--muted-foreground); }}\n\n  \
         .root {{ color: var(--tone-fg); }}\n\
         </style>\n\n\
         <span class=\"root\" part=\"root\"><slot></slot></span>\n",
        stem = tag.strip_prefix("sui-").unwrap_or(&tag),
    );
    fs::write(base.join(format!("{tag}.sui")), sui)?;

    if want_controller {
        let ts = format!(
            "import {{ controller }} from '@ship/sui-runtime'\n\
             import type {{ Scope }} from './{tag}.sui'\n\n\
             export default controller<Scope>({{\n  \
             mount: (_state, _host) => {{\n    return () => {{}}\n  }},\n\
             }})\n"
        );
        fs::write(base.join(format!("{tag}.controller.ts")), ts)?;
    }

    println!("[scaffold] created packages/components/primitives/src/{tag}/");
    println!("[scaffold] next steps:");
    println!("  1. Edit {tag}.sui — props, styles, template body, parity comment");
    println!("  2. Run: bun run --filter @ship/primitives build");
    println!("     (compiles the .sui via sui-build; emits dist/sui-primitives.js)");
    println!("  3. Run: cargo run -p xtask -- build-assets");
    println!("     (rebakes deck-core/assets/sui-components.js so the daemon serves it)");
    Ok(())
}
