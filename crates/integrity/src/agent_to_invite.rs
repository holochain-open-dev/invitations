use hdi::prelude::*;
use crate::Invite;

pub fn validate_create_link_agent_to_invite(
    _action: CreateLink,
    _base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    let action_hash = target_address.into_action_hash().ok_or(
        wasm_error!(
            WasmErrorInner::Guest(String::from("target address is not a compatible link hash"))
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
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_delete_link_agent_to_invite(
    _action: DeleteLink,
    _original_action: CreateLink,
    _base: AnyLinkableHash,
    _target: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid("Deleting links is not allowed".into()))
}
