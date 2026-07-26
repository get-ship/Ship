//! Approval-handshake types — the survivor of the `Ask`-stance interaction.
//!
//! `UserInput` is the decision a human (or scripted test driver) returns when
//! the gate raises an `Ask` for a protected op.  It is NOT a session event;
//! it is the resolution of an approval gate and lives outside the canonical
//! event stream.

/// The decision a human (or scripted approver) returns for an `Ask`-stance
/// effect.  `Approve` lets the effect proceed through the one door; `Deny`
/// surfaces as a normal denial result — never an exception.
#[derive(Debug, Clone, PartialEq)]
pub enum UserInput {
    Approve { request_id: String },
    Deny { request_id: String, reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_input_variants_carry_request_id() {
        let a = UserInput::Approve {
            request_id: "r".into(),
        };
        let d = UserInput::Deny {
            request_id: "r".into(),
            reason: "no".into(),
        };
        match a {
            UserInput::Approve { request_id } => assert_eq!(request_id, "r"),
            _ => panic!("expected Approve"),
        }
        match d {
            UserInput::Deny { request_id, reason } => {
                assert_eq!(request_id, "r");
                assert_eq!(reason, "no");
            }
            _ => panic!("expected Deny"),
        }
    }
}
