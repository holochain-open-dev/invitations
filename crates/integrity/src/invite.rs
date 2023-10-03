use std::collections::BTreeMap;

use hdi::prelude::*;
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Invite {
    pub invitees: Vec<AgentPubKey>,
    pub location: Option<String>,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
    pub details: Option<BTreeMap<String,String>>,
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
    if _original_action.author() == &_action.author 
    {
        Ok(ValidateCallbackResult::Valid)
    } else {
        Ok(ValidateCallbackResult::Invalid("Only the author of the invitation can make updates".into()))
    }
}
pub fn validate_delete_invite(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_invite: Invite,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
