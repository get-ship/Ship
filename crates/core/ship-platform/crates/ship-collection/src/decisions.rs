//! DecisionService — business logic over the decisions built-in collection.
//!
//! All writes produce collection.item_* events via CollectionService::create_item
//! / update_item — no new event families. Both memento integration paths (floki
//! MementoTool and harness adapter) call this service; the substrate piece is
//! path-independent.

use ship_substrate::ports::{Predicate, Query};

use crate::builtin::decisions_schema;
use crate::error::CollectionError;
use crate::projection::ensure_collection_table;
use crate::service::CollectionService;
use crate::types::ItemRecord;

/// The collection id for decisions.
pub const COLLECTION: &str = "decisions";

/// Ordered conviction tiers: index 0 = lowest, index 2 = highest.
const CONVICTIONS: &[&str] = &["low", "medium", "high"];

/// Wraps `CollectionService` with decisions-domain business logic.
pub struct DecisionService {
    pub inner: CollectionService,
}

/// Input for recording a decision. `conviction` defaults to "medium" when None.
pub struct RecordInput<'a> {
    pub kind: &'a str,
    pub title: &'a str,
    pub rationale: Option<&'a str>,
    /// "low" | "medium" | "high". Defaults to "medium" if None.
    pub conviction: Option<&'a str>,
    pub tags: &'a [String],
}

/// Outcome of a `record` call.
#[derive(Debug)]
pub enum RecordResult {
    Created { id: String },
    Reinforced { id: String, confirmations: u64 },
}

/// Outcome of a `supersede` call.
#[derive(Debug)]
pub struct SupersedeResult {
    pub superseded_id: String,
    pub new_id: String,
}

impl DecisionService {
    /// Ensure the `collection_decisions` table exists with properly typed columns.
    ///
    /// Called at the start of every public method — idempotent (CREATE TABLE IF
    /// NOT EXISTS + additive ALTER TABLE ADD COLUMN).
    async fn ensure_table(&self) -> Result<(), CollectionError> {
        ensure_collection_table(self.inner.ctx.data(), &decisions_schema()).await
    }

    /// Record a decision — dedup+reinforce or create.
    ///
    /// Dedup key: `(kind, lower(title), status='active')`.
    /// On match: `confirmations + 1`, conviction advanced one tier (capped at "high").
    /// On miss: new item, `status='active'`, `confirmations=1`.
    pub async fn record(&self, input: RecordInput<'_>) -> Result<RecordResult, CollectionError> {
        self.ensure_table().await?;

        // Empty title is not useful as a decision statement.
        if input.title.trim().is_empty() {
            return Err(CollectionError::Validation(
                "title is required and must not be empty".to_string(),
            ));
        }

        let conviction = input.conviction.unwrap_or("medium");

        // Server-side dedup query: kind + status='active'.
        let q = Query {
            table: format!("collection_{COLLECTION}"),
            filters: vec![
                Predicate::eq("kind", serde_json::Value::String(input.kind.to_string())),
                Predicate::eq("status", serde_json::Value::String("active".to_string())),
            ],
            limit: 500,
            ..Default::default()
        };
        let (candidates, _) = self.inner.query_items_by_query(q).await?;

        // Client-side: case-insensitive title match.
        let title_lower = input.title.to_lowercase();
        let existing = candidates.into_iter().find(|item| {
            item.data
                .get("title")
                .and_then(|v| v.as_str())
                .map(|t| t.to_lowercase() == title_lower)
                .unwrap_or(false)
        });

        if let Some(existing_item) = existing {
            // Reinforce: increment confirmations, advance conviction one tier.
            let old_conf = get_confirmations(&existing_item);
            let new_conf = old_conf + 1;
            let old_conviction = existing_item
                .data
                .get("conviction")
                .and_then(|v| v.as_str())
                .unwrap_or("medium");
            let next_conviction = advance_conviction(old_conviction);

            let patch = serde_json::json!({
                "confirmations": new_conf,
                "conviction": next_conviction,
            });
            let patch_bytes = serde_json::to_vec(&patch)
                .map_err(|e| CollectionError::Internal(e.to_string()))?;
            self.inner.update_item(COLLECTION, &existing_item.id, &patch_bytes).await?;

            Ok(RecordResult::Reinforced { id: existing_item.id, confirmations: new_conf })
        } else {
            // Create new decision.
            let mut data_map = serde_json::Map::new();
            data_map.insert("kind".into(), serde_json::Value::String(input.kind.to_string()));
            data_map.insert("status".into(), serde_json::Value::String("active".to_string()));
            data_map.insert("title".into(), serde_json::Value::String(input.title.to_string()));
            data_map.insert("conviction".into(), serde_json::Value::String(conviction.to_string()));
            data_map.insert("confirmations".into(), serde_json::json!(1u64));
            if let Some(rationale) = input.rationale {
                data_map.insert(
                    "rationale".into(),
                    serde_json::Value::String(rationale.to_string()),
                );
            }
            if !input.tags.is_empty() {
                data_map.insert(
                    "tags".into(),
                    serde_json::Value::Array(
                        input.tags.iter().map(|t| serde_json::Value::String(t.clone())).collect(),
                    ),
                );
            }

            let data = serde_json::Value::Object(data_map);
            let data_bytes = serde_json::to_vec(&data)
                .map_err(|e| CollectionError::Internal(e.to_string()))?;
            let item = self.inner.create_item(COLLECTION, &data_bytes).await?;
            Ok(RecordResult::Created { id: item.id })
        }
    }

    /// Supersede an existing active decision with a new one.
    ///
    /// Emits two collection events:
    /// - `collection.item_created` for the new record (with `supersedes=old_id`)
    /// - `collection.item_updated` for the old record (sets `status='superseded'`,
    ///   `superseded_by=new_id`)
    ///
    /// Rejects if `old_id` is not found or is already superseded.
    pub async fn supersede(
        &self,
        old_id: &str,
        input: RecordInput<'_>,
    ) -> Result<SupersedeResult, CollectionError> {
        self.ensure_table().await?;

        // Fetch and validate the old record.
        let old_item = self.inner.get_item(COLLECTION, old_id).await?;
        let old_status = old_item.data.get("status").and_then(|v| v.as_str()).unwrap_or("active");
        if old_status == "superseded" {
            return Err(CollectionError::Validation(format!(
                "decision '{old_id}' is already superseded"
            )));
        }

        let conviction = input.conviction.unwrap_or("medium");

        // Create the new record with supersedes=old_id.
        let mut data_map = serde_json::Map::new();
        data_map.insert("kind".into(), serde_json::Value::String(input.kind.to_string()));
        data_map.insert("status".into(), serde_json::Value::String("active".to_string()));
        data_map.insert("title".into(), serde_json::Value::String(input.title.to_string()));
        data_map.insert("conviction".into(), serde_json::Value::String(conviction.to_string()));
        data_map.insert("confirmations".into(), serde_json::json!(1u64));
        data_map.insert("supersedes".into(), serde_json::Value::String(old_id.to_string()));
        if let Some(rationale) = input.rationale {
            data_map.insert(
                "rationale".into(),
                serde_json::Value::String(rationale.to_string()),
            );
        }
        if !input.tags.is_empty() {
            data_map.insert(
                "tags".into(),
                serde_json::Value::Array(
                    input.tags.iter().map(|t| serde_json::Value::String(t.clone())).collect(),
                ),
            );
        }

        let data = serde_json::Value::Object(data_map);
        let data_bytes = serde_json::to_vec(&data)
            .map_err(|e| CollectionError::Internal(e.to_string()))?;
        let new_item = self.inner.create_item(COLLECTION, &data_bytes).await?;
        let new_id = new_item.id.clone();

        // Mark the old record as superseded.
        let patch = serde_json::json!({
            "status": "superseded",
            "superseded_by": new_id,
        });
        let patch_bytes = serde_json::to_vec(&patch)
            .map_err(|e| CollectionError::Internal(e.to_string()))?;
        self.inner.update_item(COLLECTION, old_id, &patch_bytes).await?;

        Ok(SupersedeResult { superseded_id: old_id.to_string(), new_id })
    }

    /// Query decisions with optional filters.
    ///
    /// `status` defaults to `"active"`. `min_conviction`, `tag` are client-side
    /// filters. Results are sorted by `conviction_tier * confirmations` descending.
    pub async fn query(
        &self,
        kind: Option<&str>,
        status: Option<&str>,
        min_conviction: Option<&str>,
        tag: Option<&str>,
        limit: u32,
    ) -> Result<Vec<ItemRecord>, CollectionError> {
        self.ensure_table().await?;

        let status = status.unwrap_or("active");
        let mut filters = vec![Predicate::eq(
            "status",
            serde_json::Value::String(status.to_string()),
        )];
        if let Some(k) = kind {
            filters.push(Predicate::eq("kind", serde_json::Value::String(k.to_string())));
        }

        let q = Query {
            table: format!("collection_{COLLECTION}"),
            filters,
            limit: 500,
            ..Default::default()
        };
        let (items, _) = self.inner.query_items_by_query(q).await?;

        // Client-side filters.
        let min_tier = min_conviction
            .and_then(|c| CONVICTIONS.iter().position(|&v| v == c));

        let mut filtered: Vec<ItemRecord> = items
            .into_iter()
            .filter(|item| {
                // min_conviction filter.
                if let Some(min) = min_tier {
                    let conv = item.data.get("conviction").and_then(|v| v.as_str()).unwrap_or("medium");
                    let tier = CONVICTIONS.iter().position(|&v| v == conv).unwrap_or(1);
                    if tier < min {
                        return false;
                    }
                }
                // tag filter.
                if let Some(t) = tag {
                    let has_tag = match item.data.get("tags") {
                        Some(serde_json::Value::Array(arr)) => {
                            arr.iter().any(|e| e.as_str() == Some(t))
                        }
                        Some(serde_json::Value::String(s)) => {
                            let arr: Vec<serde_json::Value> =
                                serde_json::from_str(s).unwrap_or_default();
                            arr.iter().any(|e| e.as_str() == Some(t))
                        }
                        _ => false,
                    };
                    if !has_tag {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Sort by conviction_tier * confirmations descending.
        filtered.sort_by(|a, b| {
            let score_a = conviction_tier_f64(a) * get_confirmations_f64(a);
            let score_b = conviction_tier_f64(b) * get_confirmations_f64(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        filtered.truncate(limit as usize);
        Ok(filtered)
    }

    /// Get a single decision by id.
    pub async fn get(&self, id: &str) -> Result<ItemRecord, CollectionError> {
        self.ensure_table().await?;
        self.inner.get_item(COLLECTION, id).await
    }

    /// Return active decisions ranked by confirmations descending, then conviction
    /// descending.
    pub async fn audit(&self, limit: u32) -> Result<Vec<ItemRecord>, CollectionError> {
        self.ensure_table().await?;

        let q = Query {
            table: format!("collection_{COLLECTION}"),
            filters: vec![Predicate::eq(
                "status",
                serde_json::Value::String("active".to_string()),
            )],
            limit: 500,
            ..Default::default()
        };
        let (mut items, _) = self.inner.query_items_by_query(q).await?;

        // Sort by confirmations desc, then conviction tier desc.
        items.sort_by(|a, b| {
            let conf_cmp = get_confirmations_f64(b)
                .partial_cmp(&get_confirmations_f64(a))
                .unwrap_or(std::cmp::Ordering::Equal);
            if conf_cmp != std::cmp::Ordering::Equal {
                return conf_cmp;
            }
            conviction_tier(b).cmp(&conviction_tier(a))
        });

        items.truncate(limit as usize);
        Ok(items)
    }
}

// ── private helpers ───────────────────────────────────────────────────────────

/// Advance conviction one tier, capped at the highest tier ("high").
fn advance_conviction(current: &str) -> &'static str {
    let idx = CONVICTIONS.iter().position(|&v| v == current).unwrap_or(1);
    CONVICTIONS[(idx + 1).min(CONVICTIONS.len() - 1)]
}

/// Conviction tier index (0=low, 1=medium, 2=high). Defaults to 1 on unknown.
fn conviction_tier(item: &ItemRecord) -> usize {
    let conv = item.data.get("conviction").and_then(|v| v.as_str()).unwrap_or("medium");
    CONVICTIONS.iter().position(|&v| v == conv).unwrap_or(1)
}

fn conviction_tier_f64(item: &ItemRecord) -> f64 {
    conviction_tier(item) as f64
}

/// Read confirmations, handling both Number and (unlikely) String storage forms.
fn get_confirmations(item: &ItemRecord) -> u64 {
    match item.data.get("confirmations") {
        Some(serde_json::Value::Number(n)) => {
            n.as_u64().or_else(|| n.as_f64().map(|f| f as u64)).unwrap_or(1)
        }
        Some(serde_json::Value::String(s)) => s.parse().unwrap_or(1),
        _ => 1,
    }
}

fn get_confirmations_f64(item: &ItemRecord) -> f64 {
    get_confirmations(item) as f64
}
