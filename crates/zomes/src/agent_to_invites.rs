use hdi::prelude::*;
pub fn validate_create_link_agent_to_invites(
    _action: CreateLink,
    _base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    let entry_hash = EntryHash::from(target_address);
    let entry = must_get_entry(entry_hash)?.content;
    let _invite = crate::Invite::try_from(entry)?;
    Ok(ValidateCallbackResult::Valid)
}
pub fn validate_delete_link_agent_to_invites(
    _action: DeleteLink,
    _original_action: CreateLink,
    _base: AnyLinkableHash,
    _target: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
