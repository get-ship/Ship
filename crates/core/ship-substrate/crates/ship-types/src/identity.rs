/// Actor identity — unique instance IDs and stable labels.
#[derive(Debug, Clone)]
pub struct ActorIdentity {
    pub instance_id: String,
    pub label: String,
    pub actor_type: ActorType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorType {
    Agent,
    Service,
    App,
    Cli,
}
