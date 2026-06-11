//! `ViewService` policy + gate conformance tests.
//!
//! The `impl ViewService for OpsImpl` is deleted — `ViewWire` (ship-views)
//! is now the sole handler, mounted via `ViewServiceExt::register` in
//! `rpc/ops`.

#[cfg(test)]
mod tests {
    /// Every `view.*` write declares `ship.views.write` / C1 in the compiled
    /// descriptor — the field-level policy the gate reads at runtime. Matching
    /// the generated CommandOps `view.*` ops is the proto-as-SSOT contract.
    #[test]
    fn view_rpc_op_policy_in_descriptor() {
        for method in [
            "PatchSession",
            "AppendSession",
            "SignalSession",
            "EnterEditMode",
            "ExitEditMode",
            "Tool",
        ] {
            let path = format!("ship.ops.v1.ViewService/{method}");
            let p = crate::rpc::ops::policy_for(&path)
                .unwrap_or_else(|| panic!("{path} must carry a policy in the descriptor"));
            assert_eq!(p.capability, "ship.views.write");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Space);
        }
    }
}
