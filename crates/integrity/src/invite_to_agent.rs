use hdi::prelude::*;
use crate::Invite;

pub fn validate_create_link_invite_to_agent(
    _action: CreateLink,
    base_address: AnyLinkableHash,
    _target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    let action_hash = base_address.into_action_hash().ok_or(
        wasm_error!(
            WasmErrorInner::Guest(String::from("base address is not a compatible link hash"))
        ),
    )?;
    let record = must_get_valid_record(action_hash)?;
    let _invite: Invite = record
        .entry()
        .to_app_option()
        .map_err(|e| wasm_error!(e))?
        .ok_or(
            wasm_error!(
                WasmErrorInner::Guest(String::from("Linked action must reference an entry"))
            ),
        )?;
    if !_invite.invitees.contains(&_action.author) {
        Ok(ValidateCallbackResult::Invalid("only invitees can respond to invites".into()))
    } else {
        Ok(ValidateCallbackResult::Valid)
    }
}
pub fn validate_delete_link_invite_to_agent(
    _action: DeleteLink,
    _original_action: CreateLink,
    _base: AnyLinkableHash,
    _target: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid("Deleting links is not allowed".into()))

}
