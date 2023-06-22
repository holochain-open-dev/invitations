use hdi::prelude::*;
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Invite {
    pub inviter: AgentPubKey,
    pub invitees: Vec<AgentPubKey>,
    pub timestamp: Timestamp,
}
pub fn validate_create_invite(
    _action: EntryCreationAction,
    _invite: Invite,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
pub fn validate_update_invite(
    _action: Update,
    _invite: Invite,
    _original_action: EntryCreationAction,
    _original_invite: Invite,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
pub fn validate_delete_invite(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_invite: Invite,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
