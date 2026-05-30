//! Behavioral tests for `SearchService`. Exercises the composition path
//! end-to-end through a `TestSearchContext` returning fixture snapshots.
//! Each test states a single behavioral invariant; if it fails, that
//! invariant is broken.

use async_trait::async_trait;

use ship_search::{
    EntityKind, EntityKindFilter, SearchContext, SearchOpts, SearchService,
    adapters::{
        agents::AgentSnapshot,
        artifacts::ArtifactSnapshot,
        collection_items::CollectionItemSnapshot,
        collections::CollectionSnapshot,
        sessions::SessionSnapshot,
        skills::SkillSnapshot,
        views::ViewSnapshot,
    },
};

// ── fixture builder ────────────────────────────────────────────────────

#[derive(Default)]
struct TestSearchContext {
    views: Vec<ViewSnapshot>,
    collections: Vec<CollectionSnapshot>,
    collection_items: Vec<CollectionItemSnapshot>,
    agents: Vec<AgentSnapshot>,
    skills: Vec<SkillSnapshot>,
    sessions: Vec<SessionSnapshot>,
    artifacts: Vec<ArtifactSnapshot>,
}

impl TestSearchContext {
    fn new() -> Self { Self::default() }
    fn with_agents(mut self, agents: Vec<AgentSnapshot>) -> Self { self.agents = agents; self }
    fn with_collections(mut self, c: Vec<CollectionSnapshot>) -> Self { self.collections = c; self }
    fn with_skills(mut self, s: Vec<SkillSnapshot>) -> Self { self.skills = s; self }
    fn with_views(mut self, v: Vec<ViewSnapshot>) -> Self { self.views = v; self }
}

#[async_trait]
impl SearchContext for TestSearchContext {
    fn views(&self) -> Vec<ViewSnapshot> { self.views.clone() }
    fn collections(&self) -> Vec<CollectionSnapshot> { self.collections.clone() }
    fn agents(&self) -> Vec<AgentSnapshot> { self.agents.clone() }
    fn skills(&self) -> Vec<SkillSnapshot> { self.skills.clone() }
    fn sessions(&self) -> Vec<SessionSnapshot> { self.sessions.clone() }
    async fn collection_items(&self) -> Vec<CollectionItemSnapshot> { self.collection_items.clone() }
    async fn artifacts(&self) -> Vec<ArtifactSnapshot> { self.artifacts.clone() }
}

// ── snapshot constructors ──────────────────────────────────────────────

fn agent(id: &str, name: &str) -> AgentSnapshot {
    AgentSnapshot { id: id.into(), name: Some(name.into()), description: None }
}
fn collection(id: &str, name: &str) -> CollectionSnapshot {
    CollectionSnapshot { id: id.into(), name: Some(name.into()) }
}
fn skill(id: &str, name: &str, desc: &str) -> SkillSnapshot {
    SkillSnapshot {
        id: id.into(),
        name: Some(name.into()),
        description: Some(desc.into()),
        body: None,
    }
}
fn view(name: &str) -> ViewSnapshot {
    ViewSnapshot { name: name.into(), description: None, icon: None }
}

// ── tests ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn query_matches_name_returns_hit() {
    let ctx = TestSearchContext::new()
        .with_agents(vec![agent("a-1", "alpha-agent"), agent("b-1", "beta-agent")]);
    let resp = SearchService::new().run(&ctx, "alpha", SearchOpts::default()).await;
    assert_eq!(resp.hits.len(), 1, "expected 1 hit, got {:?}", resp.hits);
    assert_eq!(resp.hits[0].id, "a-1");
}

#[tokio::test]
async fn name_match_outranks_description_match() {
    let ctx = TestSearchContext::new().with_skills(vec![
        skill("name-match", "alpha", "boring description"),
        skill("desc-match", "boring-name", "alpha is the description"),
    ]);
    let resp = SearchService::new().run(&ctx, "alpha", SearchOpts::default()).await;
    assert!(resp.hits.len() >= 2, "expected 2 hits, got {:?}", resp.hits);
    let name_hit = resp.hits.iter().position(|h| h.id == "name-match").expect("name-match present");
    let desc_hit = resp.hits.iter().position(|h| h.id == "desc-match").expect("desc-match present");
    assert!(name_hit < desc_hit, "name-match (idx {name_hit}) should outrank desc-match (idx {desc_hit})");
}

#[tokio::test]
async fn kind_filter_excludes_other_kinds() {
    let ctx = TestSearchContext::new()
        .with_agents(vec![agent("a-1", "alpha")])
        .with_collections(vec![collection("c-1", "alpha")])
        .with_skills(vec![skill("s-1", "alpha", "")]);
    let opts = SearchOpts { types: Some(vec![EntityKindFilter::Agent]), ..Default::default() };
    let resp = SearchService::new().run(&ctx, "alpha", opts).await;
    for hit in &resp.hits {
        assert!(matches!(hit.kind, EntityKind::Agent), "non-agent leaked: {:?}", hit.kind);
    }
}

#[tokio::test]
async fn cross_kind_ranking_orders_by_score() {
    let ctx = TestSearchContext::new()
        .with_agents(vec![agent("a-1", "alpha"), agent("a-2", "alphabet-soup")])
        .with_collections(vec![collection("c-1", "alpha-collection")]);
    let resp = SearchService::new().run(&ctx, "alpha", SearchOpts::default()).await;
    let scores: Vec<f64> = resp.hits.iter().map(|h| h.score).collect();
    let sorted: Vec<f64> = {
        let mut s = scores.clone();
        s.sort_by(|a, b| b.partial_cmp(a).unwrap());
        s
    };
    assert_eq!(scores, sorted, "hits should be score-descending");
}

#[tokio::test]
async fn empty_kinds_filter_searches_all_sources() {
    let ctx = TestSearchContext::new()
        .with_agents(vec![agent("a-1", "alpha")])
        .with_collections(vec![collection("c-1", "alpha")])
        .with_skills(vec![skill("s-1", "alpha", "")])
        .with_views(vec![view("alpha-view")]);
    let resp = SearchService::new().run(&ctx, "alpha", SearchOpts::default()).await;
    let kinds: std::collections::HashSet<_> =
        resp.hits.iter().map(|h| std::mem::discriminant(&h.kind)).collect();
    assert!(kinds.len() >= 3, "expected hits across ≥3 kinds, got {:?}", resp.hits);
}

#[tokio::test]
async fn limit_caps_result_count() {
    let agents: Vec<_> = (0..50).map(|i| agent(&format!("a-{i}"), &format!("alpha-{i}"))).collect();
    let ctx = TestSearchContext::new().with_agents(agents);
    let opts = SearchOpts { limit: 10, ..Default::default() };
    let resp = SearchService::new().run(&ctx, "alpha", opts).await;
    assert!(resp.hits.len() <= 10, "limit 10 honored, got {}", resp.hits.len());
}

#[tokio::test]
async fn scope_filter_at_adapter_boundary() {
    // Demonstrates that scope-filtering happens IN the context adapter,
    // not in the service. The "hub" context returns broader snapshots;
    // the "space" context returns a subset. Search service is identical.
    let hub_ctx = TestSearchContext::new()
        .with_agents(vec![agent("global-1", "alpha"), agent("space-A-1", "alpha")]);
    let space_a_ctx = TestSearchContext::new()
        .with_agents(vec![agent("space-A-1", "alpha")]);

    let svc = SearchService::new();
    let hub_resp = svc.run(&hub_ctx, "alpha", SearchOpts::default()).await;
    let space_resp = svc.run(&space_a_ctx, "alpha", SearchOpts::default()).await;

    assert_eq!(hub_resp.hits.len(), 2, "hub sees both");
    assert_eq!(space_resp.hits.len(), 1, "space sees only its own");
    assert_eq!(space_resp.hits[0].id, "space-A-1");
}

#[tokio::test]
async fn empty_context_returns_no_hits() {
    let ctx = TestSearchContext::new();
    let resp = SearchService::new().run(&ctx, "anything", SearchOpts::default()).await;
    assert!(resp.hits.is_empty());
}
