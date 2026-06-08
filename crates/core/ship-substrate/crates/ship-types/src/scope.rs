//! Authoring scopes — where a customization-vector directory is rooted.

use serde::{Deserialize, Serialize};

/// The scope at which a vector directory is rooted.
///
/// - `Project`: project-local `.ship/<vector>/` at a given project root.
/// - `User`: user-library `~/.ship/library/<vector>/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scope {
    Project,
    User,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_are_distinct() {
        assert_ne!(Scope::Project, Scope::User);
    }
}
