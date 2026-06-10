# Job S-skills: skill discovery + context injection (lightweight)

## Context
Work in `spikes/ship-agent/` (own Cargo workspace; builds from any location).
Read `README.md` (SSOT). Skills are the **user-customization surface** and a
standard — this is intentionally small: discover skills, inject their
descriptions into context, and load a skill's body on demand. Nothing fancy.

Ship's existing skill format is `<dir>/<id>/SKILL.md` with a YAML frontmatter
block (`--- ... ---`) carrying at least `name` and `description`, then a markdown
body. (See real examples under the repo's `.ship/skills/`.)

## Task — add `src/skills.rs` (+ `mod skills;` in main.rs)
1. `struct Skill { name: String, description: String, path: PathBuf, requires: Vec<String> }`.
   - `requires` is the Ship-native **capability hint**: an optional frontmatter
     `requires: [ship.collections.read, ...]` array of capability ids the skill
     needs. Empty if absent. (Explicit-over-implicit: a skill declares the grants
     it implies rather than hiding them in prose.)
2. `fn discover_skills(root: &Path) -> Vec<Skill>` — scan `root/*/SKILL.md` (one
   level), parse the frontmatter for `name`, `description`, and optional
   `requires`. Skip dirs without a valid `SKILL.md`. Parse the frontmatter by
   hand (find the `---` fenced block; read `key: value` and simple `[a, b]`
   arrays) — do NOT add a YAML dependency.
3. `fn render_catalog(skills: &[Skill]) -> String` — the text injected into the
   system prompt: each skill as `name — description` (one per line), sorted by
   name for determinism.
4. `fn load_body(skill: &Skill) -> std::io::Result<String>` — read `SKILL.md`,
   strip the frontmatter, return the markdown body (loaded on demand when the
   agent activates a skill).

## Acceptance (behavioral tests — NOT brittle)
Use `tempfile` (already a dev-dependency) to build fixture skill dirs:
- `discover_skills` finds skills with valid `SKILL.md`, parses name+description,
  and skips dirs lacking one.
- a skill declaring `requires: [ship.collections.read]` parses that into
  `requires`; one without yields empty `requires`.
- `render_catalog` is sorted and contains each skill's name and description.
- `load_body` returns the body with the frontmatter stripped (body present,
  `---` fence not included).

Assert *semantics* (found/skipped, present/absent, sorted, stripped) — never exact
counts of a specific fixture set or full string equality of generated catalogs.

## Process (IMPORTANT)
Run `cd spikes/ship-agent && cargo test` **in the foreground and WAIT for the
final `test result:` line** before reporting. Do NOT background the build and
return early. First build is slow (deno_core/V8) — wait it out.
When green (existing 67 + new), `git add` + `git commit` on the current branch.

## Constraints
- New file `src/skills.rs` (+ its `mod` line) only. Do NOT touch `agent_loop.rs`,
  enforcement/execution modules, `provider.rs`, `session.rs`, `events.rs`, or
  `Cargo.toml` (no new deps; `tempfile` is already a dev-dependency).
- Behavioral tests only. Update `README.md` module map (add a `skills.rs` row).
