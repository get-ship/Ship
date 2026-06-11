//! `RuntimeService` handler — runtime introspection over the `process.*`
//! projection, proto-as-SSOT (authored in `runtime.proto`). Thin glue: reads the
//! projection via `ship_substrate::runtime_reads::list_processes_at`, applies the
//! same kind-filter/sort/cap the retired REST handler did, and maps records into
//! the generated proto types. C0 read; gate reads `ship.runtime.read` from the FDS.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use ship_substrate::projections::process::{ProcessRecord, ProcessStatus};

use ship_ops_rpc::{
    ListProcsRequest, ListProcsResponse, RuntimeProcess, RuntimeService, RuntimeSignalEntry,
};

use crate::rpc::ops::OpsImpl;

/// Session class → the `executor_kind` strings the orchestrator emits for it.
/// `None` for an unrecognized kind (the handler returns invalid-argument).
fn kind_to_executor_kinds(kind: &str) -> Option<&'static [&'static str]> {
    match kind {
        "agent" => Some(&["pty", "tmux", "tmux_window"]),
        "pipeline-step" => Some(&["process"]),
        "shell" => Some(&["shell"]),
        _ => None,
    }
}

fn status_str(s: ProcessStatus) -> String {
    match s {
        ProcessStatus::Running => "running",
        ProcessStatus::Exited => "exited",
        ProcessStatus::Killed => "killed",
        ProcessStatus::Orphaned => "orphaned",
    }
    .to_string()
}

fn proc_to_proto(r: ProcessRecord) -> RuntimeProcess {
    RuntimeProcess {
        pid: r.pid,
        slug: r.slug,
        pipeline_run_id: r.pipeline_run_id,
        step_id: r.step_id,
        cmd: r.cmd,
        executor_kind: r.executor_kind,
        spawned_at: r.spawned_at.to_rfc3339(),
        exited_at: r.exited_at.map(|d| d.to_rfc3339()),
        exit_code: r.exit_code,
        exit_signal: r.exit_signal,
        signals_sent: r
            .signals_sent
            .into_iter()
            .map(|s| RuntimeSignalEntry {
                signum: s.signum,
                reason: s.reason,
                actor: s.actor,
                at: s.at.to_rfc3339(),
                ..Default::default()
            })
            .collect(),
        status: status_str(r.status),
        ..Default::default()
    }
}

#[allow(refining_impl_trait)]
impl RuntimeService for OpsImpl {
    async fn list_procs(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedListProcsRequestView,
    ) -> ServiceResult<ListProcsResponse> {
        let limit = request.limit.unwrap_or(100) as usize;
        let kind_filter = match request.kind {
            Some(k) => match kind_to_executor_kinds(k) {
                Some(set) => Some(set),
                None => {
                    return Err(ConnectError::invalid_argument(format!(
                        "invalid kind '{k}'. Valid values: agent, pipeline-step, shell"
                    )));
                }
            },
            None => None,
        };
        let mut records =
            ship_substrate::runtime_reads::list_processes_at(self.state().ship_dir(), request.slug)
                .map_err(|e| ConnectError::internal(format!("procs: {e}")))?;
        if let Some(set) = kind_filter {
            records.retain(|r| set.contains(&r.executor_kind.as_str()));
        }
        records.sort_by(|a, b| b.spawned_at.cmp(&a.spawned_at));
        records.truncate(limit);
        Ok(Response::new(ListProcsResponse {
            processes: records.into_iter().map(proc_to_proto).collect(),
            ..Default::default()
        }))
    }
}

// Part of the public surface (clients build it); the handler reads the view.
const _: () = {
    fn _assert(_r: ListProcsRequest) {}
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use ship_substrate::projections::process::SignalEntry;

    #[test]
    fn process_row_maps_enum_datetime_and_signals() {
        let at = chrono::Utc.with_ymd_and_hms(2026, 5, 24, 0, 0, 0).unwrap();
        let row = ProcessRecord {
            pid: Some(4242),
            slug: "run-1".into(),
            pipeline_run_id: Some("pr-1".into()),
            step_id: None,
            cmd: "claude".into(),
            executor_kind: "pty".into(),
            spawned_at: at,
            exited_at: None,
            exit_code: None,
            exit_signal: None,
            signals_sent: vec![SignalEntry {
                signum: "SIGTERM".into(),
                reason: "cancel".into(),
                actor: "operator".into(),
                at,
            }],
            status: ProcessStatus::Killed,
        };
        let p = proc_to_proto(row);
        assert_eq!(p.pid, Some(4242));
        assert_eq!(p.status, "killed"); // enum → snake string
        assert_eq!(p.exited_at, None); // absent stays absent
        assert_eq!(p.spawned_at, at.to_rfc3339()); // DateTime → rfc3339
        assert_eq!(p.signals_sent.len(), 1);
        assert_eq!(p.signals_sent[0].signum, "SIGTERM");
    }

    #[test]
    fn unknown_kind_is_rejected() {
        assert!(kind_to_executor_kinds("bogus").is_none());
        assert!(kind_to_executor_kinds("agent").is_some());
    }
}
