//! `UiService` policy + gate conformance tests.
//!
//! The `impl UiService for OpsImpl` is gone — `UiWire` (ship-ui) is now the
//! sole handler, mounted via `UiServiceExt::register` in `rpc/ops/ui_wire`.
//!
//! The unified `ship.ui` service (proto/ship/ops/v1/ui.proto) carries
//! twelve RPCs: five C0 reads (`Packs`, `Read`, `Diagnostics`, `Hover`,
//! `Dependencies`) and seven C1 writes (`PatchSession`, `AppendSession`,
//! `SignalSession`, `EnterEditMode`, `ExitEditMode`, `Tool`, `Scaffold`).
//! These tests assert the proto-as-SSOT contract for the writes: every C1
//! op must carry `ship.ui.write` and the C1Space blast radius.

#[cfg(test)]
mod tests {
    /// Every C1 write on the unified `ship.ui` service declares
    /// `ship.ui.write` / C1Space in the compiled descriptor — the field-level
    /// policy the gate reads at runtime. Matching the generated CommandOps
    /// `ui.*` ops is the proto-as-SSOT contract.
    #[test]
    fn ui_write_ops_policy_in_descriptor() {
        for method in [
            "PatchSession",
            "AppendSession",
            "SignalSession",
            "EnterEditMode",
            "ExitEditMode",
            "Tool",
            "Scaffold",
        ] {
            let path = format!("ship.ops.v1.UiService/{method}");
            let p = crate::rpc::ops::policy_for(&path)
                .unwrap_or_else(|| panic!("{path} must carry a policy in the descriptor"));
            assert_eq!(p.capability, "ship.ui.write", "{path} capability");
            assert_eq!(
                p.blast_radius,
                ship_ops_rpc::BlastRadius::C1Space,
                "{path} blast radius"
            );
        }
    }

    /// Every C0 read on `ship.ui` declares `ship.ui.read` / C0Safe.
    #[test]
    fn ui_read_ops_policy_in_descriptor() {
        for method in ["Packs", "Read", "Diagnostics", "Hover", "Dependencies"] {
            let path = format!("ship.ops.v1.UiService/{method}");
            let p = crate::rpc::ops::policy_for(&path)
                .unwrap_or_else(|| panic!("{path} must carry a policy in the descriptor"));
            assert_eq!(p.capability, "ship.ui.read", "{path} capability");
            assert_eq!(
                p.blast_radius,
                ship_ops_rpc::BlastRadius::C0Safe,
                "{path} blast radius"
            );
        }
    }
}
