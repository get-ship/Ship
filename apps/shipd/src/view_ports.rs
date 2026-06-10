//! Shipd implementations of the ship-views port traits.
//!
//! Bridges `ApiState` (Arc<ShipRuntime>) to the abstract ports declared in
//! `ship_views::view_ports`. Follows the same pattern as
//! `agent_event_ports.rs` and `pipeline_ports.rs`.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use ship_substrate::ports::SpaceContext;
use ship_views::view_ports::{BundlerBuildError, ViewToolPort};

use crate::ship_runtime::ApiState;

// ── ViewToolPort ──────────────────────────────────────────────────────────────

pub struct ViewToolAdapter {
    state: ApiState,
}

impl ViewToolAdapter {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl ViewToolPort for ViewToolAdapter {
    async fn build_view(&self, target: &str) -> Result<(), BundlerBuildError> {
        self.state
            .bundler()
            .build(target)
            .await
            .map(|_| ())
            .map_err(|e| match e {
                deck::bundler::BundlerError::NoCompiler => BundlerBuildError::NoCompiler,
                deck::bundler::BundlerError::UnknownTarget(t) => {
                    BundlerBuildError::UnknownTarget(t)
                }
                deck::bundler::BundlerError::Bundle { message, .. } => {
                    BundlerBuildError::Bundle {
                        message,
                        file: None,
                        line: None,
                        col: None,
                    }
                }
            })
    }

    fn space_context(&self, space: &str) -> Arc<SpaceContext> {
        ship_views::view_authoring::space_context_for(
            self.state.kernel().clone(),
            self.state.ship_dir(),
            space,
        )
    }

    fn ship_dir(&self) -> PathBuf {
        self.state.ship_dir().to_path_buf()
    }

    fn collections(&self) -> anyhow::Result<ship_collection::service::CollectionService> {
        self.state.collections()
    }

    fn project_slug(&self) -> Option<String> {
        self.state.project_slug()
    }

    fn resolve_theme_css(&self) -> String {
        // Theme resolution now lives in deck-core (the single owner of the
        // CSS/theme serving surface). shipd delegates — the non-serving
        // consumer is `ship_views::view_authoring::tool_dispatch`.
        deck_core::resolve_theme_css()
    }

    fn resolve_view_source(&self, name: &str) -> Result<(String, std::path::PathBuf), String> {
        let resolved = deck::resolver::resolve(name, None)
            .map_err(|e| e.message.clone())?;
        let path = resolved
            .source_path
            .ok_or_else(|| "view has no on-disk source".to_string())?;
        Ok((resolved.source, path))
    }

    fn preview_view_html(&self, name: &str) -> Result<String, ship_views::view_ports::PreviewError> {
        use ship_views::view_ports::PreviewError;
        let resolved = deck::resolver::resolve(name, None)
            .map_err(|e| PreviewError::ViewNotFound(e.message.clone()))?;
        let rendered = deck::resolver::render_resolved(&resolved, name)
            .map_err(PreviewError::RenderFailed)?;
        Ok(rendered.html)
    }
}
