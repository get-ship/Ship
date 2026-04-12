//! Specs view — Datastar SSE renderer.
//!
//! Subscribes to `job.*` kernel events so launched-spec status reflects
//! live job progress. On any event (and on a 2s heartbeat) re-renders
//! `#spec-list` and emits it as a PatchElements patch.

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use datastar::prelude::PatchElements;
use runtime::db::specs::{self, Spec};
use runtime::events::ActorConfig;
use std::convert::Infallible;
use std::time::Duration;

/// GET /api/runtime/view/specs/sse
pub async fn specs_view_sse(
    State(state): State<crate::rest_api::ApiState>,
) -> Sse<impl futures::stream::Stream<Item = Result<Event, Infallible>>> {
    let actor_id = format!(
        "specs-sse-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string()],
    };

    let mailbox = state
        .kernel()
        .lock()
        .await
        .spawn_actor(&actor_id, config)
        .ok()
        .map(|(_store, mb)| mb);

    let guard = crate::runtime_api::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    let stream = async_stream::stream! {
        let _guard = guard;

        // Initial render.
        yield Ok::<_, Infallible>(PatchElements::new(render_spec_list()).write_as_axum_sse_event());

        // Re-render on job events OR every 2s heartbeat.
        if let Some(mut mb) = mailbox {
            loop {
                let sleep = tokio::time::sleep(Duration::from_secs(2));
                tokio::pin!(sleep);
                tokio::select! {
                    maybe_ev = mb.recv() => {
                        if maybe_ev.is_none() { break; }
                    }
                    _ = &mut sleep => {}
                }
                yield Ok::<_, Infallible>(PatchElements::new(render_spec_list()).write_as_axum_sse_event());
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// ---- Rendering ----

fn render_spec_list() -> String {
    let specs = specs::list_specs().unwrap_or_default();
    if specs.is_empty() {
        return r#"<div id="spec-list" class="spec-list"><div class="empty">No specs yet. Click New Spec to start.</div></div>"#.to_string();
    }

    let rows: String = specs.iter().map(render_spec_row).collect();
    format!(r#"<div id="spec-list" class="spec-list">{rows}</div>"#)
}

fn render_spec_row(s: &Spec) -> String {
    let status = html_escape(&s.status);
    let title = if s.title.is_empty() { "(untitled)".to_string() } else { html_escape(&s.title) };
    let pipeline = s.pipeline.as_deref().map(html_escape).unwrap_or_else(|| "&mdash;".into());
    format!(
        r##"<button class="spec-row" data-show="$filter === '{status}'" data-on:click="$selectedId = '{id}'; fetch('/api/runtime/specs/{id}', {{headers: {{authorization: 'Bearer ' + window.__shipToken}}}}).then(r => r.json()).then(j => {{ const d = j.data || {{}}; $title = d.title || ''; $body = d.body || ''; $pipeline = d.pipeline || ''; $status = d.status || 'draft'; $criteria = d.criteria || []; $criteriaText = ($criteria).join('\\n'); $scope = d.scope || []; $scopeText = ($scope).join('\\n'); $vars = d.vars || {{}}; $varsText = Object.entries($vars).map(([k,v]) => k+'='+v).join('\\n'); $dirty = false; }})">
  <div class="spec-row-title">{title}</div>
  <div class="spec-row-meta"><span class="badge badge-{status}">{status}</span><span class="pipeline-tag">{pipeline}</span></div>
</button>"##,
        id = html_escape(&s.id),
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_html() {
        assert_eq!(html_escape("<a>&\""), "&lt;a&gt;&amp;&quot;");
    }

    #[test]
    fn empty_list_renders_empty_state() {
        let out = render_spec_list();
        assert!(out.contains("No specs yet"));
        assert!(out.starts_with(r#"<div id="spec-list""#));
    }
}
