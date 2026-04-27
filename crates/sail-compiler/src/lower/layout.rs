//! Compile-time layout resolution.
//!
//! Two patterns this phase resolves before emit runs:
//!
//! 1. **`<extends src="…">`** — the child file's `<fill name="X">`
//!    blocks override the parent's `<block name="X">` placeholders.
//!    Recursive: a layout can itself extend another layout.
//!
//! 2. **`<include src="…">`** — drop-in compile-time partial. The
//!    included file's template body replaces the include node.
//!    Resolved depth-first.
//!
//! Both rely on a [`Loader`] callback that maps a relative path
//! string to source. `compile_file` constructs a filesystem-rooted
//! loader from the file's parent directory; tests can pass an
//! in-memory map.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Result, anyhow};

use crate::parse::{self, ast::{Node, Template}};

/// Maps a `src` string from `<extends>` / `<include>` to its source
/// content. Returning `None` is an error reported by the caller.
pub trait Loader: Send + Sync {
    fn load(&self, src: &str) -> Option<String>;
}

/// Filesystem loader rooted at a directory. Resolves paths relative
/// to that directory. Returns `None` if the file doesn't exist.
pub struct FsLoader {
    pub base: PathBuf,
}

impl FsLoader {
    pub fn new(base: PathBuf) -> Self {
        Self { base }
    }
}

impl Loader for FsLoader {
    fn load(&self, src: &str) -> Option<String> {
        let path = self.base.join(src);
        std::fs::read_to_string(&path).ok()
    }
}

/// In-memory loader for tests + non-filesystem callers.
pub struct MapLoader {
    pub files: HashMap<String, String>,
}

impl Loader for MapLoader {
    fn load(&self, src: &str) -> Option<String> {
        self.files.get(src).cloned()
    }
}

/// Resolve a parsed file's template against extends + includes,
/// returning a flattened template with no `Block`/`Fill`/`Include`
/// nodes remaining (Blocks are replaced by Fills or their defaults;
/// Fills only matter inside an Extends; Includes are inlined).
///
/// Cycle protection: tracks the chain of `extends` srcs and errors
/// on revisit. Depth limit (16) guards against pathological loaders.
pub fn resolve(template: &Template, loader: &dyn Loader) -> Result<Template> {
    let mut fills: HashMap<String, Vec<Node>> = HashMap::new();
    let mut chain: Vec<String> = Vec::new();
    let resolved_nodes = resolve_chain(template, loader, &mut fills, &mut chain, 0)?;
    // After the chain produces a flat node list (with blocks substituted
    // for matching fills), strip any orphan blocks down to their
    // defaults so a layout file rendered standalone shows placeholders
    // rather than empty space.
    Ok(Template {
        root: strip_blocks(resolved_nodes),
        extends: None,
    })
}

/// Walk the extends chain bottom-up. At each layer:
/// 1. Inline includes in this layer's nodes.
/// 2. Add this layer's `<fill>` declarations to the merged map
///    (child overrides parent — but fills accumulate, so any layer
///    can supply a fill that a higher layer's block consumes).
/// 3. If this layer extends, recurse into the parent and merge.
/// 4. If not, this is the topmost — substitute blocks using the
///    fully-merged map and return.
fn resolve_chain(
    template: &Template,
    loader: &dyn Loader,
    fills: &mut HashMap<String, Vec<Node>>,
    extends_chain: &mut Vec<String>,
    depth: u32,
) -> Result<Vec<Node>> {
    if depth > 16 {
        return Err(anyhow!("layout resolution exceeded depth limit (16)"));
    }
    let nodes = inline_includes(&template.root, loader, depth)?;

    // Collect this layer's fills. Child fills are added FIRST in the
    // recursion order; if a parent layer also declares a fill of the
    // same name, the child's already-inserted entry wins.
    for n in &nodes {
        if let Node::Fill { name, body } = n {
            fills.entry(name.clone()).or_insert_with(|| body.clone());
        }
    }

    if let Some(src) = template.extends.clone() {
        if extends_chain.contains(&src) {
            return Err(anyhow!(
                "circular extends: {} appears twice in the chain",
                src
            ));
        }
        let parent_source = loader
            .load(&src)
            .ok_or_else(|| anyhow!("extends source not found: {}", src))?;
        let parent = parse::parse(&parent_source)
            .map_err(|e| anyhow!("parse extends source {}: {}", src, e))?;
        let parent_template = parent
            .template
            .ok_or_else(|| anyhow!("extends source {} has no <template>", src))?;
        extends_chain.push(src);
        let parent_nodes =
            resolve_chain(&parent_template, loader, fills, extends_chain, depth + 1)?;
        extends_chain.pop();
        // The parent's resolved nodes (already substituted at its layer
        // for fills available there) come back. We don't substitute
        // further at this layer because the parent's resolve_chain has
        // already used the full merged fills map.
        return Ok(parent_nodes);
    }

    // Topmost layer (no extends). Substitute blocks with the merged
    // fills map and return.
    Ok(replace_blocks_in(nodes, fills))
}

fn inline_includes(nodes: &[Node], loader: &dyn Loader, depth: u32) -> Result<Vec<Node>> {
    let mut out: Vec<Node> = Vec::with_capacity(nodes.len());
    for n in nodes {
        match n {
            Node::Include { src } => {
                let body = loader
                    .load(src)
                    .ok_or_else(|| anyhow!("include source not found: {}", src))?;
                let parsed = parse::parse(&body)
                    .map_err(|e| anyhow!("parse include source {}: {}", src, e))?;
                let included_template = match parsed.template {
                    Some(t) => t,
                    None => continue,
                };
                let resolved = resolve(&included_template, loader)?;
                out.extend(resolved.root);
            }
            other => out.push(walk_clone(other, loader, depth)?),
        }
    }
    Ok(out)
}

/// Recursive structure-preserving clone that resolves nested
/// `Include` nodes inside element/component children, branch bodies,
/// and slot fallbacks. Block/Fill bodies are kept as-is here — they
/// only matter at the extends boundary.
fn walk_clone(n: &Node, loader: &dyn Loader, depth: u32) -> Result<Node> {
    Ok(match n {
        Node::Element { tag, attrs, children } => Node::Element {
            tag: tag.clone(),
            attrs: attrs.clone(),
            children: inline_includes(children, loader, depth)?,
        },
        Node::Component { tag, props, events, slots, children } => Node::Component {
            tag: tag.clone(),
            props: props.clone(),
            events: events.clone(),
            slots: slots.clone(),
            children: inline_includes(children, loader, depth)?,
        },
        Node::If { cond, then_, elifs, else_ } => Node::If {
            cond: cond.clone(),
            then_: inline_includes(then_, loader, depth)?,
            elifs: elifs
                .iter()
                .map(|(e, body)| -> Result<_> {
                    Ok((e.clone(), inline_includes(body, loader, depth)?))
                })
                .collect::<Result<Vec<_>>>()?,
            else_: match else_ {
                Some(b) => Some(inline_includes(b, loader, depth)?),
                None => None,
            },
        },
        Node::For { item, iter, key, body, empty } => Node::For {
            item: item.clone(),
            iter: iter.clone(),
            key: key.clone(),
            body: inline_includes(body, loader, depth)?,
            empty: match empty {
                Some(b) => Some(inline_includes(b, loader, depth)?),
                None => None,
            },
        },
        Node::Await { promise, loading, then_, catch } => Node::Await {
            promise: promise.clone(),
            loading: inline_includes(loading, loader, depth)?,
            then_: match then_ {
                Some((p, body)) => Some((p.clone(), inline_includes(body, loader, depth)?)),
                None => None,
            },
            catch: match catch {
                Some((p, body)) => Some((p.clone(), inline_includes(body, loader, depth)?)),
                None => None,
            },
        },
        Node::Block { name, default } => Node::Block {
            name: name.clone(),
            default: inline_includes(default, loader, depth)?,
        },
        Node::Fill { name, body } => Node::Fill {
            name: name.clone(),
            body: inline_includes(body, loader, depth)?,
        },
        other => other.clone(),
    })
}

/// Walk the parent's tree and replace `<block name="X">` with the
/// matching `<fill name="X">` body, or the block's own default body
/// when no fill exists. Recurses into element/component children
/// and branch bodies so blocks anywhere in the layout are honored.
fn replace_blocks_in(nodes: Vec<Node>, fills: &HashMap<String, Vec<Node>>) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::with_capacity(nodes.len());
    for n in nodes {
        match n {
            Node::Block { name, default } => {
                let body = fills.get(&name).cloned().unwrap_or(default);
                out.extend(replace_blocks_in(body, fills));
            }
            Node::Element { tag, attrs, children } => out.push(Node::Element {
                tag,
                attrs,
                children: replace_blocks_in(children, fills),
            }),
            Node::Component { tag, props, events, slots, children } => out.push(Node::Component {
                tag,
                props,
                events,
                slots,
                children: replace_blocks_in(children, fills),
            }),
            Node::If { cond, then_, elifs, else_ } => out.push(Node::If {
                cond,
                then_: replace_blocks_in(then_, fills),
                elifs: elifs
                    .into_iter()
                    .map(|(e, body)| (e, replace_blocks_in(body, fills)))
                    .collect(),
                else_: else_.map(|b| replace_blocks_in(b, fills)),
            }),
            Node::For { item, iter, key, body, empty } => out.push(Node::For {
                item,
                iter,
                key,
                body: replace_blocks_in(body, fills),
                empty: empty.map(|b| replace_blocks_in(b, fills)),
            }),
            other => out.push(other),
        }
    }
    out
}

/// In a standalone (non-extending) file, strip orphan `Block` and
/// `Fill` wrappers — render the block's default content directly,
/// drop fills (they only made sense in extending files).
fn strip_blocks(nodes: Vec<Node>) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::with_capacity(nodes.len());
    for n in nodes {
        match n {
            Node::Block { default, .. } => out.extend(strip_blocks(default)),
            Node::Fill { .. } => {} // orphan fill — drop
            Node::Element { tag, attrs, children } => out.push(Node::Element {
                tag,
                attrs,
                children: strip_blocks(children),
            }),
            Node::Component { tag, props, events, slots, children } => out.push(Node::Component {
                tag,
                props,
                events,
                slots,
                children: strip_blocks(children),
            }),
            Node::If { cond, then_, elifs, else_ } => out.push(Node::If {
                cond,
                then_: strip_blocks(then_),
                elifs: elifs
                    .into_iter()
                    .map(|(e, body)| (e, strip_blocks(body)))
                    .collect(),
                else_: else_.map(|b| strip_blocks(b)),
            }),
            Node::For { item, iter, key, body, empty } => out.push(Node::For {
                item,
                iter,
                key,
                body: strip_blocks(body),
                empty: empty.map(|b| strip_blocks(b)),
            }),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loader_with(files: &[(&str, &str)]) -> MapLoader {
        let mut map = HashMap::new();
        for (k, v) in files {
            map.insert((*k).to_string(), (*v).to_string());
        }
        MapLoader { files: map }
    }

    fn template_of(s: &str) -> Template {
        parse::parse(s).unwrap().template.unwrap()
    }

    #[test]
    fn no_extends_strips_blocks_to_defaults() {
        let t = template_of(
            "<template><div><block name=\"head\">DEF</block></div></template>",
        );
        let r = resolve(&t, &MapLoader { files: HashMap::new() }).unwrap();
        // Block is stripped; "DEF" text remains inside the div.
        let html = format!("{:?}", r.root);
        assert!(html.contains("DEF"));
        assert!(!html.contains("Block"));
    }

    #[test]
    fn extends_replaces_blocks_with_fills() {
        let t = template_of(
            "<template>\
<extends src=\"layout.set\" />\
<fill name=\"content\"><p>HELLO</p></fill>\
</template>",
        );
        let loader = loader_with(&[(
            "layout.set",
            "<template><main><block name=\"content\">DEFAULT</block></main></template>",
        )]);
        let r = resolve(&t, &loader).unwrap();
        let html = format!("{:?}", r.root);
        assert!(html.contains("HELLO"));
        assert!(!html.contains("DEFAULT"));
        assert!(!html.contains("Block"));
    }

    #[test]
    fn missing_fill_falls_back_to_block_default() {
        let t = template_of(
            "<template><extends src=\"layout.set\" /></template>",
        );
        let loader = loader_with(&[(
            "layout.set",
            "<template><main><block name=\"content\">DEFAULT</block></main></template>",
        )]);
        let r = resolve(&t, &loader).unwrap();
        let html = format!("{:?}", r.root);
        assert!(html.contains("DEFAULT"));
    }

    #[test]
    fn extends_chain_resolves_recursively() {
        let t = template_of(
            "<template>\
<extends src=\"middle.set\" />\
<fill name=\"body\"><p>FROM-CHILD</p></fill>\
</template>",
        );
        let loader = loader_with(&[
            (
                "middle.set",
                "<template>\
<extends src=\"base.set\" />\
<fill name=\"main\"><section><block name=\"body\">middle-default</block></section></fill>\
</template>",
            ),
            (
                "base.set",
                "<template><html-shell><block name=\"main\">base-default</block></html-shell></template>",
            ),
        ]);
        let r = resolve(&t, &loader).unwrap();
        let html = format!("{:?}", r.root);
        assert!(html.contains("FROM-CHILD"));
        assert!(html.contains("html-shell"));
        assert!(html.contains("section"));
        assert!(!html.contains("middle-default"));
        assert!(!html.contains("base-default"));
    }

    #[test]
    fn include_inlines_partial() {
        let t = template_of(
            "<template><div><include src=\"_partial.set\" /></div></template>",
        );
        let loader = loader_with(&[(
            "_partial.set",
            "<template><span>PARTIAL</span></template>",
        )]);
        let r = resolve(&t, &loader).unwrap();
        let html = format!("{:?}", r.root);
        assert!(html.contains("PARTIAL"));
        assert!(html.contains("span"));
    }

    #[test]
    fn circular_extends_errors() {
        let t = template_of("<template><extends src=\"a.set\" /></template>");
        let loader = loader_with(&[
            ("a.set", "<template><extends src=\"b.set\" /></template>"),
            ("b.set", "<template><extends src=\"a.set\" /></template>"),
        ]);
        let err = resolve(&t, &loader).unwrap_err();
        assert!(err.to_string().contains("circular"));
    }

    #[test]
    fn missing_extends_source_errors() {
        let t = template_of("<template><extends src=\"missing.set\" /></template>");
        let loader = MapLoader { files: HashMap::new() };
        let err = resolve(&t, &loader).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }
}
