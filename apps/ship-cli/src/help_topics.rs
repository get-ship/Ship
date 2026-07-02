//! `ship docs <topic>` — extended help for common workflows.

use anyhow::Result;

pub fn run(topic: Option<&str>) -> Result<()> {
    match topic {
        None | Some("topics") | Some("list") => {
            print_topic_list();
            Ok(())
        }
        Some(name) => match lookup(name) {
            Some(body) => {
                println!("{body}");
                Ok(())
            }
            None => {
                eprintln!("Unknown topic: {name}");
                eprintln!();
                print_topic_list();
                Err(anyhow::anyhow!("unknown help topic: {name}"))
            }
        },
    }
}

fn print_topic_list() {
    println!("Available help topics:");
    println!();
    for (name, summary) in TOPICS {
        println!("  {:<12} {}", name, summary);
    }
    println!();
    println!("Run `ship docs <topic>` for details.");
}

const TOPICS: &[(&str, &str)] = &[
    ("agents", "Creating and managing agent definitions"),
    ("compile", "How compilation works and harness output"),
    ("config", "User preferences and environment variables"),
    ("harnesses", "Supported harnesses and their output formats"),
    ("skills", "Adding and authoring agent skills"),
    ("workflow", "Typical day-to-day workflow with Ship"),
];

fn lookup(topic: &str) -> Option<&'static str> {
    match topic {
        "agents" => Some(TOPIC_AGENTS),
        "compile" => Some(TOPIC_COMPILE),
        "config" => Some(TOPIC_CONFIG),
        "harnesses" => Some(TOPIC_HARNESSES),
        "skills" => Some(TOPIC_SKILLS),
        "workflow" => Some(TOPIC_WORKFLOW),
        _ => None,
    }
}

const TOPIC_AGENTS: &str = "\
Agents

An agent is a JSONC file in .ship/agents/ that declares an AI assistant's
identity, skills, permissions, and harness targets.

  Create:   Add a file at .ship/agents/<id>.jsonc
  Activate: ship use <id>
  Recompile: ship compile

To add more agents, copy dev.jsonc and change the id, name, and skill refs.

Agent files live in .ship/agents/<id>.jsonc. Global agents live in
~/.ship/agents/<id>.jsonc.

After editing an agent, run `ship compile` or `ship use <id>` to regenerate
harness-native config files.
";

const TOPIC_COMPILE: &str = "\
Compilation

`ship compile` reads the active agent from .ship/ and writes harness-native
config files to the project root (e.g. CLAUDE.md, .cursor/, .mcp.json).

  Compile all harnesses:    ship compile
  Single harness:          ship compile --harness claude
  Preview without writing:  ship compile --dry-run

The compiler resolves skills, permissions, MCP servers, and rules into a
single output per harness. Output files are build artifacts and should be
gitignored.
";

const TOPIC_CONFIG: &str = "\
Configuration

Settings are stored by shipd per scope — account (portable, syncs across
your devices) and runtime (machine-local). There is no config file; set
and read them through the running daemon:

  ship config get <key>       Read a value
  ship config set <key> <val> Write a value
  ship config list            Show all set values

Available keys:

  terminal.program    Terminal for dispatch: wt, iterm, tmux, gnome, vscode, manual
  dispatch.confirm    Show spec and ask y/n before launching agent (true/false)
  defaults.harness   Default compilation harness (claude, codex, cursor, opencode)
  defaults.mode       Default agent permission mode
  identity.name       Your display name
  identity.email      Your email

Environment variables override config values when set:

  SHIP_DEFAULT_TERMINAL   Overrides terminal.program
  SHIP_DISPATCH_CONFIRM   Set to 1 to override dispatch.confirm
  SHIP_WORKTREE_DIR       Overrides worktrees.dir

Resolution order: command flag > env var > ship config > default.
";

const TOPIC_HARNESSES: &str = "\
Harnesses

Ship compiles agent config to harness-native formats:

  claude    CLAUDE.md + .mcp.json
  cursor    .cursor/ directory
  codex     .codex/ directory + AGENTS.md

Set harnesses in your agent JSONC:
  \"agent\": { \"harnesses\": [\"claude\", \"codex\"] }

Or compile for one harness at a time:
  ship compile --harness claude
";

const TOPIC_SKILLS: &str = "\
Skills

Skills are markdown files that add domain knowledge or workflow instructions
to an agent. They live in .ship/skills/<skill-id>/SKILL.md.

  Create a new skill:    mkdir .ship/skills/my-skill && edit SKILL.md

Skills are referenced in agent JSONC files:
  \"skills\": { \"refs\": [\"my-skill\", \"code-review\"] }

A skill directory contains SKILL.md (required), and optionally
assets/vars.json (typed variables) and references/docs/ (reference material).
";

const TOPIC_WORKFLOW: &str = "\
Typical Workflow

1. Scaffold .ship/ in a new project (or copy from an existing one):
   mkdir -p .ship/agents && cat > .ship/agents/dev.jsonc <<'JSON'
   { \"agent\": { \"id\": \"dev\", \"name\": \"Dev\", \"harnesses\": [\"claude\"] } }
   JSON

2. Edit the agent to add skills, permissions, harnesses:
   Edit .ship/agents/dev.jsonc directly

3. Activate and compile:
   ship use dev
   (re-compile after changes: ship compile)

4. Add more agents by copying dev.jsonc:
   cp .ship/agents/dev.jsonc .ship/agents/backend.jsonc
   (change the id and name, then: ship use backend)

The compiled output (CLAUDE.md, .cursor/, etc.) is gitignored.
Only .ship/ is committed to version control.
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_topics_resolve() {
        for (name, _) in TOPICS {
            assert!(
                lookup(name).is_some(),
                "topic '{}' listed but not found",
                name
            );
        }
    }

    #[test]
    fn unknown_topic_returns_none() {
        assert!(lookup("nonexistent").is_none());
    }

    #[test]
    fn run_topics_list_succeeds() {
        assert!(run(Some("topics")).is_ok());
    }

    #[test]
    fn run_known_topic_succeeds() {
        assert!(run(Some("agents")).is_ok());
    }

    #[test]
    fn run_unknown_topic_fails() {
        assert!(run(Some("nonexistent")).is_err());
    }

    #[test]
    fn run_none_shows_list() {
        assert!(run(None).is_ok());
    }
}
