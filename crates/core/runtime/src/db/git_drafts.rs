//! Git commit draft store — platform DB backing for agent-written commit
//! messages awaiting human sign-off.
//!
//! One draft per branch. Agents write drafts when they finish work; the
//! git view displays the draft pre-filled in the commit composer; on
//! successful commit the draft is cleared.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::db::{block_on, open_db};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommitDraft {
    pub branch: String,
    pub subject: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub by_agent: Option<String>,
    #[serde(default)]
    pub by_pipeline: Option<String>,
    pub drafted_at: DateTime<Utc>,
}

fn row_to_draft(row: sqlx::sqlite::SqliteRow) -> Result<CommitDraft> {
    let drafted_at: String = row.try_get("drafted_at")?;
    Ok(CommitDraft {
        branch: row.try_get("branch")?,
        subject: row.try_get("subject")?,
        body: row.try_get("body")?,
        by_agent: row.try_get("by_agent").ok(),
        by_pipeline: row.try_get("by_pipeline").ok(),
        drafted_at: DateTime::parse_from_rfc3339(&drafted_at)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

pub fn get_draft(branch: &str) -> Result<Option<CommitDraft>> {
    let mut conn = open_db()?;
    let row = block_on(async {
        sqlx::query(
            "SELECT branch, subject, body, by_agent, by_pipeline, drafted_at
             FROM git_commit_drafts WHERE branch = ?",
        )
        .bind(branch)
        .fetch_optional(&mut conn)
        .await
    })?;
    match row {
        Some(r) => Ok(Some(row_to_draft(r)?)),
        None => Ok(None),
    }
}

pub fn upsert_draft(d: &CommitDraft) -> Result<()> {
    let mut conn = open_db()?;
    let drafted = d.drafted_at.to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO git_commit_drafts (branch, subject, body, by_agent, by_pipeline, drafted_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(branch) DO UPDATE SET
               subject = excluded.subject,
               body = excluded.body,
               by_agent = excluded.by_agent,
               by_pipeline = excluded.by_pipeline,
               drafted_at = excluded.drafted_at",
        )
        .bind(&d.branch)
        .bind(&d.subject)
        .bind(&d.body)
        .bind(&d.by_agent)
        .bind(&d.by_pipeline)
        .bind(&drafted)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn delete_draft(branch: &str) -> Result<()> {
    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("DELETE FROM git_commit_drafts WHERE branch = ?")
            .bind(branch)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::init_project;
    use tempfile::tempdir;

    fn sample(branch: &str) -> CommitDraft {
        CommitDraft {
            branch: branch.into(),
            subject: "feat: thing".into(),
            body: "details".into(),
            by_agent: Some("rust-lane".into()),
            by_pipeline: Some("tdd".into()),
            drafted_at: Utc::now(),
        }
    }

    #[test]
    fn crud_roundtrip() {
        let tmp = tempdir().unwrap();
        let _ = init_project(tmp.path().to_path_buf()).unwrap();
        crate::db::ensure_db().unwrap();

        assert!(get_draft("feat/x").unwrap().is_none());

        let d = sample("feat/x");
        upsert_draft(&d).unwrap();

        let got = get_draft("feat/x").unwrap().expect("draft present");
        assert_eq!(got.subject, "feat: thing");
        assert_eq!(got.body, "details");
        assert_eq!(got.by_agent.as_deref(), Some("rust-lane"));
        assert_eq!(got.by_pipeline.as_deref(), Some("tdd"));

        // Update path — overwrites subject/body.
        let mut updated = got.clone();
        updated.subject = "feat: thing v2".into();
        upsert_draft(&updated).unwrap();
        assert_eq!(get_draft("feat/x").unwrap().unwrap().subject, "feat: thing v2");

        delete_draft("feat/x").unwrap();
        assert!(get_draft("feat/x").unwrap().is_none());
    }

    #[test]
    fn delete_missing_is_noop() {
        let tmp = tempdir().unwrap();
        let _ = init_project(tmp.path().to_path_buf()).unwrap();
        crate::db::ensure_db().unwrap();
        delete_draft("nope").unwrap();
    }
}
