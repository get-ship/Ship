//! `agents.wire(id)` — graph/canvas detail payload.
//!
//! Server owns layout math so the template stays declarative. The
//! ship-set canvas component consumes the `nodes` + `edges` arrays and
//! renders SVG.

use serde::Serialize;
use std::path::Path;

use super::view_state::{AgentCatalogs, AgentDetail, list_agent_catalogs, load_agent};

/// A positioned node on the wire canvas. `kind` drives color/shape in
/// the template.
#[derive(Serialize)]
pub struct WireNode {
    pub kind: &'static str, // agent | skill | mcp | rule | perm | prov
    pub id: String,
    pub title: String,
    pub sub: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
}

/// A bezier edge between the center agent node and a peripheral.
#[derive(Serialize)]
pub struct WireEdge {
    pub kind: &'static str,
    pub id: String, // matches the peripheral node's id
    pub path_d: String,
    pub dashed: bool,
}

#[derive(Serialize)]
pub struct AgentWireCtx {
    pub agent: AgentDetail,
    pub nodes: Vec<WireNode>,
    pub edges: Vec<WireEdge>,
    /// Catalogs for the picker dialog (skills/mcp/rules not already
    /// attached to this agent). Shared with the settings view.
    pub catalogs: AgentCatalogs,
}

/// Build the full wire payload for an agent by id. Returns `None` when
/// no agent with that id exists in either scope.
pub fn build(id: &str, ship_dir: &Path) -> Option<AgentWireCtx> {
    let agent = load_agent(ship_dir, id)?;
    let (nodes, edges) = compute_wire_layout(&agent);
    let mut catalogs = list_agent_catalogs(ship_dir);
    catalogs.skills.retain(|e| !agent.skills.contains(&e.id));
    catalogs.mcp.retain(|e| !agent.mcp_servers.contains(&e.id));
    catalogs.rules.retain(|e| !agent.rules.contains(&e.id));
    Some(AgentWireCtx { agent, nodes, edges, catalogs })
}

/// Compute node positions + bezier edges. Canvas is a 1000x600 viewBox
/// with the agent anchored center-left.
pub fn compute_wire_layout(agent: &AgentDetail) -> (Vec<WireNode>, Vec<WireEdge>) {
    let agent_x = 420.0_f32;
    let agent_y = 240.0_f32;
    let agent_w = 220.0_f32;
    let agent_h = 80.0_f32;

    let mut nodes: Vec<WireNode> = Vec::new();
    nodes.push(WireNode {
        kind: "agent",
        id: agent.id.clone(),
        title: agent.name.clone(),
        sub: agent.id.clone(),
        x: agent_x,
        y: agent_y,
        w: agent_w,
    });

    fn column(n: usize, y0: f32, y1: f32) -> Vec<f32> {
        if n == 0 { return vec![] }
        if n == 1 { return vec![(y0 + y1) / 2.0] }
        let step = (y1 - y0) / (n as f32 - 1.0).max(1.0);
        (0..n).map(|i| y0 + step * i as f32).collect()
    }

    let node_w: f32 = 160.0;

    // Skills — left column.
    for (i, y) in column(agent.skills.len(), 60.0, 420.0).into_iter().enumerate() {
        let s = &agent.skills[i];
        nodes.push(WireNode {
            kind: "skill",
            id: s.clone(),
            title: s.clone(),
            sub: "project skill".into(),
            x: 70.0,
            y,
            w: node_w,
        });
    }

    // MCP — top row centered above agent.
    for (i, s) in agent.mcp_servers.iter().enumerate() {
        let x = 280.0 + (i as f32) * 180.0;
        nodes.push(WireNode {
            kind: "mcp",
            id: s.clone(),
            title: s.clone(),
            sub: "mcp server".into(),
            x,
            y: 90.0,
            w: node_w,
        });
    }

    // Rules — bottom-center.
    if !agent.rules.is_empty() {
        nodes.push(WireNode {
            kind: "rule",
            id: "rules".into(),
            title: format!(
                "{} rule{}",
                agent.rules.len(),
                if agent.rules.len() == 1 { "" } else { "s" }
            ),
            sub: "always-on + inline".into(),
            x: 470.0,
            y: 470.0,
            w: node_w,
        });
    }

    // Permissions — bottom-left.
    nodes.push(WireNode {
        kind: "perm",
        id: "permissions".into(),
        title: capitalize(&agent.permissions_tier),
        sub: "permission preset".into(),
        x: 220.0,
        y: 470.0,
        w: node_w,
    });

    // Providers — right column.
    for (i, y) in column(agent.providers.len(), 100.0, 420.0).into_iter().enumerate() {
        let p = &agent.providers[i];
        nodes.push(WireNode {
            kind: "prov",
            id: p.clone(),
            title: capitalize(p),
            sub: "compiled target".into(),
            x: 830.0,
            y,
            w: node_w,
        });
    }

    // Edges: from agent to each peripheral.
    let agent_left = (agent_x, agent_y + agent_h / 2.0);
    let agent_right = (agent_x + agent_w, agent_y + agent_h / 2.0);
    let agent_top = (agent_x + agent_w / 2.0, agent_y);
    let agent_bottom = (agent_x + agent_w / 2.0, agent_y + agent_h);

    let mut edges: Vec<WireEdge> = Vec::new();
    for n in nodes.iter().skip(1) {
        let (fx, fy);
        let tx;
        let ty = n.y + 30.0;
        if n.x < agent_x {
            fx = agent_left.0; fy = agent_left.1;
            tx = n.x + node_w;
        } else if n.x > agent_x + agent_w {
            fx = agent_right.0; fy = agent_right.1;
            tx = n.x;
        } else if n.y < agent_y {
            fx = agent_top.0; fy = agent_top.1;
            tx = n.x + node_w / 2.0;
        } else {
            fx = agent_bottom.0; fy = agent_bottom.1;
            tx = n.x + node_w / 2.0;
        }
        let mx = (fx + tx) / 2.0;
        let path_d = format!("M{fx:.0},{fy:.0} C{mx:.0},{fy:.0} {mx:.0},{ty:.0} {tx:.0},{ty:.0}");
        edges.push(WireEdge {
            kind: n.kind,
            id: n.id.clone(),
            path_d,
            dashed: n.kind == "rule",
        });
    }

    (nodes, edges)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(c).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::view_state::Permissions;

    fn agent() -> AgentDetail {
        AgentDetail {
            id: "a".into(),
            name: "A".into(),
            description: String::new(),
            letter: "A".into(),
            skills: vec!["s1".into(), "s2".into()],
            mcp_servers: vec!["m1".into()],
            rules: vec!["r1".into()],
            providers: vec!["claude".into(), "codex".into()],
            default_provider: None,
            permissions_tier: "standard".into(),
            permissions: Permissions::default(),
            source: "PROJECT AGENTS".into(),
        }
    }

    #[test]
    fn layout_produces_center_plus_peripherals() {
        let (nodes, edges) = compute_wire_layout(&agent());
        // agent + 2 skills + 1 mcp + 1 rule + 1 perm + 2 providers = 8
        assert_eq!(nodes.len(), 8);
        // edges = nodes - 1
        assert_eq!(edges.len(), 7);
        // agent node is first
        assert_eq!(nodes[0].kind, "agent");
    }

    #[test]
    fn rule_edge_is_dashed() {
        let (_n, edges) = compute_wire_layout(&agent());
        assert!(edges.iter().any(|e| e.kind == "rule" && e.dashed));
    }

    #[test]
    fn empty_rules_skip_rule_node() {
        let mut a = agent();
        a.rules.clear();
        let (nodes, _e) = compute_wire_layout(&a);
        assert!(nodes.iter().all(|n| n.kind != "rule"));
    }
}
