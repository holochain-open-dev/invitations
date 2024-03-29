pub mod signals;
pub mod invite;

use hdk::prelude::{*, holo_hash::hash_type};
use hc_integrity_zome_invitations::*;
use signals::Signal;

#[hdk_extern]
pub fn init(_: ()) -> ExternResult<InitCallbackResult> {
    let mut functions = BTreeSet::new();
    functions.insert((zome_info()?.name, "recv_remote_signal".into()));

    let grant = ZomeCallCapGrant {
        access: CapAccess::Unrestricted, // Unrestricted access means any external agent can call the extern
        functions: GrantedFunctions::Listed(functions),
        tag: "recv_remote_signal_cap_grant".into()
    };
    create_cap_grant(grant)?;
    Ok(InitCallbackResult::Pass)
}

#[hdk_extern] 
fn recv_remote_signal(signal: Signal) -> ExternResult<()> {
    emit_signal(signal)?;
    Ok(())
}

#[hdk_extern(infallible)]
pub fn post_commit(committed_actions: Vec<SignedActionHashed>) {
    for action in committed_actions {
        if let Err(err) = signal_action(action) {
            error!("Error signaling new action: {:?}", err);
        }
    }
}
fn signal_action(action: SignedActionHashed) -> ExternResult<()> {
    match action.hashed.content.clone() {
        Action::Create(_create) => {
            if let Ok(Some(invite_entry_info)) = get_invitation_detail(&action.hashed.hash) {
                signals::invitation_received(action, invite_entry_info.clone())?;
            }
            Ok(())
        }
        Action::Update(_update) => {
            if let Ok(Some(invite_entry_info)) = get_invitation_detail_update(&action.hashed.hash) {
                signals::invitation_updated(action, invite_entry_info.clone())?;
            }
            Ok(())
        }
        Action::Delete(_delete) => {
            /*if let Ok(Some(original_app_entry))
                = get_entry_for_action(&delete.deletes_address) {
                emit_signal(Signal::EntryDeleted {
                    action,
                    original_app_entry,
                })?;
            }*/
            Ok(())
        }
        Action::CreateLink(create_link) => {
            if let Ok(Some(link_type)) = LinkTypes::from_type(create_link.zome_index, create_link.link_type) {
                if link_type == LinkTypes::InviteToAgent {
                    let invite_entry_info = get_invitation_detail_by_link_target(create_link.base_address)?;
                    if create_link.tag == LinkTag::new("accepted") {
                        signals::invitation_accepted(action, invite_entry_info)?;
                    }
                    else if create_link.tag == LinkTag::new("rejected") {
                        signals::invitation_rejected(action, invite_entry_info)?;
                    } 
                }
            }
            Ok(())
        }
        Action::DeleteLink(_delete_link) => {
            Ok(())
        }
        _ => Ok(()),
    }
}


fn get_invitation_detail(creation_action_hash: &ActionHash) -> ExternResult<Option<InviteInfo>> {
    let invite_entry_info = invite::get_invitation_info(&creation_action_hash)?;
    return Ok(Some(invite_entry_info));
}

fn get_invitation_detail_update(update_action_hash: &ActionHash) -> ExternResult<Option<InviteInfo>> {
    let invite_entry_info = invite::get_invitation_update_info(update_action_hash)?;
    return Ok(Some(invite_entry_info));
}

fn get_invitation_detail_by_link_target(link_target:HoloHash<hash_type::AnyLinkable>) -> ExternResult<InviteInfo> {
    if let Ok(action_hash) = ActionHash::try_from(link_target){
        let invite_entry_info = invite::get_invitation_info(&action_hash)?;
        return Ok(invite_entry_info)
    }
    return Err(wasm_error!("Invite_entry_info not found"))

}
