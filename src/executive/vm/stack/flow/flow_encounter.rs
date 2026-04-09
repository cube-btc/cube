use super::flow_status::FlowStatus;

/// Tells whether the current execution is in an `if_notif`/`else` block.
#[derive(Debug, Clone)]
pub enum FlowEncounter {
    IfNotif(FlowStatus),
    Else(FlowStatus),
}
