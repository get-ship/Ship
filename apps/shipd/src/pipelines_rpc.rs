//! `PipelinesService` policy + gate conformance tests.
//!
//! The `impl PipelinesService for OpsImpl` is deleted — `PipelinesWire`
//! (ship-pipeline-runtime) is now the sole handler, mounted via
//! `PipelinesServiceExt::register` in `ops_rpc.rs`.

#[cfg(test)]
mod tests {
    use ship_ops_rpc::BlastRadius;

    use crate::ops_rpc::policy_for;

    #[test]
    fn pipelines_list_is_c0_read() {
        let p = policy_for("ship.ops.v1.PipelinesService/List")
            .expect("PipelinesService/List must be in the descriptor");
        assert_eq!(p.capability, "ship.pipelines.read");
        assert_eq!(p.blast_radius, BlastRadius::C0Read);
    }

    #[test]
    fn update_status_is_c1_runtime_write() {
        let p = policy_for("ship.ops.v1.PipelinesService/UpdateStatus")
            .expect("PipelinesService/UpdateStatus must be in the descriptor");
        assert_eq!(p.capability, "ship.runtime.write");
        assert_eq!(p.blast_radius, BlastRadius::C1Space);
    }

    #[test]
    fn pipelines_put_is_c1_config_write() {
        let p = policy_for("ship.ops.v1.PipelinesService/Put")
            .expect("PipelinesService/Put must be in the descriptor");
        assert_eq!(p.capability, "ship.pipelines.write");
        assert_eq!(p.blast_radius, BlastRadius::C1Space);
    }
}
