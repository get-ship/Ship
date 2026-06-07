use crate::types::{PluginEntry, PluginsManifest};

/// Build the plugins manifest for a given harness.
/// Plugin entries inherit the harness from the manifest scope or default to
/// the compile target when no harness hint is encoded in the plugin ID.
pub(super) fn build_plugins_manifest(
    plugins: &PluginsManifest,
    harness_id: &str,
) -> PluginsManifest {
    if plugins.install.is_empty() {
        return PluginsManifest::default();
    }

    let entries = plugins
        .install
        .iter()
        .map(|entry| PluginEntry {
            id: entry.id.clone(),
            harness: if entry.harness.is_empty() {
                harness_id.to_string()
            } else {
                entry.harness.clone()
            },
        })
        .collect();

    PluginsManifest {
        install: entries,
        scope: if plugins.scope.is_empty() {
            "project".to_string()
        } else {
            plugins.scope.clone()
        },
    }
}
