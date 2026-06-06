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
        crate::views::themes::resolve_theme_css()
    }

    async fn publish_annotation(&self, view_name: &str, payload: serde_json::Value) {
        use crate::views::sui_view_edit_sse::{AnnotationPayload, publish_annotation};
        // The domain layer builds the JSON object with these exact fields.
        let p = AnnotationPayload {
            annotation_id: payload["annotation_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            view_name: payload["view_name"].as_str().unwrap_or(view_name).to_string(),
            selector: payload["selector"].as_str().unwrap_or_default().to_string(),
            note: payload["note"].as_str().unwrap_or_default().to_string(),
            kind: payload["kind"].as_str().unwrap_or("info").to_string(),
            turn_id: payload["turn_id"].as_str().unwrap_or_default().to_string(),
            timestamp: payload["timestamp"].as_str().unwrap_or_default().to_string(),
        };
        let _ = publish_annotation(view_name, &p).await;
    }

    async fn publish_annotation_clear(&self, view_name: &str, payload: serde_json::Value) {
        use crate::views::sui_view_edit_sse::{AnnotationClearPayload, publish_annotation_clear};
        let p = AnnotationClearPayload {
            annotation_id: payload["annotation_id"].as_str().map(|s| s.to_string()),
            view_name: payload["view_name"].as_str().unwrap_or(view_name).to_string(),
        };
        let _ = publish_annotation_clear(view_name, &p).await;
    }

    fn schedule_annotation_clear(&self, view_name: String) {
        use crate::views::sui_view_edit_sse::{AnnotationClearPayload, publish_annotation_clear};
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let _ = publish_annotation_clear(
                &view_name,
                &AnnotationClearPayload { annotation_id: None, view_name: view_name.clone() },
            )
            .await;
        });
    }
}
