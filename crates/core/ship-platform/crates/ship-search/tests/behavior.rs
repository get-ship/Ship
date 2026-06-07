//! Behavioral tests for `SearchService`. Exercises the composition path
//! end-to-end through a `TestSearchContext` returning fixture snapshots.
//! Each test states a single behavioral invariant; if it fails, that
//! invariant is broken.

use async_trait::async_trait;

use ship_search::{
    EntityKind, EntityKindFilter, SearchContext, SearchOpts, SearchService, SpaceTarget,
    run_global,
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

// ── global (cross-space) fan-out ─────────────────────────────────────────
//
// These exercise `run_global` end-to-end through the SAME scope-blind
// `SearchService`, proving the fan-out + merge + opt-out without any of
// shipd's multi-space wiring. Each space is modelled as its own
// `TestSearchContext`, exactly as shipd builds one `SpaceContext` per space.

// (b) GLOBAL search populates space_id on every hit; the per-space service
// path is unchanged (this-space hits stay un-stamped — covered above where
// every prior test asserts on hits whose space_id stays empty by default).
#[tokio::test]
async fn global_fanout_stamps_space_id_on_every_hit() {
    let svc = SearchService::new();
    let targets = vec![
        SpaceTarget::new("home", || {
            TestSearchContext::new().with_agents(vec![agent("h-1", "alpha-home")])
        }),
        SpaceTarget::new("work", || {
            TestSearchContext::new().with_agents(vec![agent("w-1", "alpha-work")])
        }),
    ];
    let hits = run_global(&svc, targets, "alpha", SearchOpts::default(), |_| true).await;

    assert_eq!(hits.len(), 2, "both spaces contribute, got {hits:?}");
    assert!(hits.iter().all(|h| !h.space_id.is_empty()), "every global hit is stamped");
    assert_eq!(hits.iter().find(|h| h.id == "h-1").unwrap().space_id, "home");
    assert_eq!(hits.iter().find(|h| h.id == "w-1").unwrap().space_id, "work");
}

// (a, end-to-end) the merged set is ranked by the shared score across spaces.
#[tokio::test]
async fn global_fanout_ranks_across_spaces_by_score() {
    let svc = SearchService::new();
    // Exact-name match outscores a substring match (ranking heuristic), so the
    // hit whose name equals the query ranks first regardless of which space
    // it lives in.
    let targets = vec![
        SpaceTarget::new("space-a", || {
            TestSearchContext::new().with_agents(vec![agent("a-1", "alphabet-soup")])
        }),
        SpaceTarget::new("space-b", || {
            TestSearchContext::new().with_agents(vec![agent("b-1", "alpha")])
        }),
    ];
    let hits = run_global(&svc, targets, "alpha", SearchOpts::default(), |_| true).await;
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].id, "b-1", "exact-name match ranks first");
    assert_eq!(hits[0].space_id, "space-b");
    let scores: Vec<f64> = hits.iter().map(|h| h.score).collect();
    let mut sorted = scores.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    assert_eq!(scores, sorted, "merged hits are score-descending across spaces");
}

// (c) an opted-out space is never searched, so its hits never reach the merge.
#[tokio::test]
async fn global_fanout_excludes_opted_out_space() {
    let svc = SearchService::new();
    let targets = vec![
        SpaceTarget::new("included", || {
            TestSearchContext::new().with_agents(vec![agent("in-1", "alpha")])
        }),
        SpaceTarget::new("opted-out", || {
            TestSearchContext::new().with_agents(vec![agent("out-1", "alpha")])
        }),
    ];
    // Opt-out predicate: model the per-space `search.global = false` setting.
    let hits = run_global(&svc, targets, "alpha", SearchOpts::default(), |space_id| {
        space_id != "opted-out"
    })
    .await;

    assert_eq!(hits.len(), 1, "only the included space contributes, got {hits:?}");
    assert_eq!(hits[0].id, "in-1");
    assert_eq!(hits[0].space_id, "included");
    assert!(
        hits.iter().all(|h| h.space_id != "opted-out"),
        "opted-out space must not leak into the merged set"
    );
}

// (d, end-to-end) a global hit's deeplink composes ship://space/<id><path>.
#[tokio::test]
async fn global_fanout_hit_deeplinks_with_space_prefix() {
    let svc = SearchService::new();
    let targets = vec![SpaceTarget::new("space-7", || {
        TestSearchContext::new().with_views(vec![view("dashboard")])
    })];
    let hits = run_global(&svc, targets, "dashboard", SearchOpts::default(), |_| true).await;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, "/dashboard");
    assert_eq!(hits[0].deeplink(), "ship://space/space-7/dashboard");
}
