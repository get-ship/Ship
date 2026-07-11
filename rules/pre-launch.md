---
alwaysApply: true
---

# Pre-launch (delete this file at launch)

No users and no persisted data to protect yet. Until launch:

- No migration code, compat shims, or deprecation paths. Change the thing directly.
- Breaking old behavior in a refactor is expected, not a regression. A test that asserts removed behavior is deleted, not satisfied — the bar is green on the new design, not green on the old.
- Replace, don't layer. No parallel old/new paths kept "for safety." Delete the old.
