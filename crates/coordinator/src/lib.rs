pub mod signals;
pub mod invite;

use hdk::prelude::{*, holo_hash::hash_type};
use hc_integrity_zome_invitations::*;
use invite::InvitationEntryInfo;
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
    //let signal_detail: SignalDetails = signal.decode().map_err(|err| wasm_error!(WasmErrorInner::Guest(err.into())))?;
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
        Action::Update(update) => {
            if let Ok(Some(app_entry)) = get_entry_for_action(&action.hashed.hash) {
                if let Ok(Some(original_app_entry))
                    = get_entry_for_action(&update.original_action_address) {
                    emit_signal(Signal::EntryUpdated {
                        action,
                        app_entry,
                        original_app_entry,
                    })?;
                }
            }
            Ok(())
        }
        Action::Delete(delete) => {
            if let Ok(Some(original_app_entry))
                = get_entry_for_action(&delete.deletes_address) {
                emit_signal(Signal::EntryDeleted {
                    action,
                    original_app_entry,
                })?;
            }
            Ok(())
        }
        Action::CreateLink(create_link) => {
            if let Ok(Some(link_type)) = LinkTypes::from_type(create_link.zome_index, create_link.link_type) {
                if link_type == LinkTypes::InviteToMembers {
                    let invite_entry_info = get_invitation_detail_by_link_target(create_link.base_address)?;
                    if create_link.tag == LinkTag::new("Accepted") {
                        signals::invitation_accepted(action, invite_entry_info)?;
                    }
                    else if create_link.tag == LinkTag::new("Rejected") {
                        signals::invitation_rejected(action, invite_entry_info)?;
                    } 
                }
                    //emit_signal(Signal::LinkCreated {
                     //   action,
                     //   link_type,
                   // })?;
                }
          //  }
            Ok(())
        }
        Action::DeleteLink(delete_link) => {
            let record = get(
                    delete_link.link_add_address.clone(),
                    GetOptions::default(),
                )?
                .ok_or(
                    wasm_error!(
                        WasmErrorInner::Guest("Failed to fetch CreateLink action"
                        .to_string())
                    ),
                )?;
            match record.action() {
                Action::CreateLink(create_link) => {
                    if let Ok(Some(link_type))
                        = LinkTypes::from_type(
                            create_link.zome_index,
                            create_link.link_type,
                        ) {
                        emit_signal(Signal::LinkDeleted {
                            action,
                            link_type,
                        })?;
                    }
                    Ok(())
                }
                _ => {
                    return Err(
                        wasm_error!(
                            WasmErrorInner::Guest("Create Link should exist".to_string())
                        ),
                    );
                }
            }
        }
        _ => Ok(()),
    }
}
fn get_entry_for_action(action_hash: &ActionHash) -> ExternResult<Option<EntryTypes>> {
    let record = match get_details(action_hash.clone(), GetOptions::default())? {
        Some(Details::Record(record_details)) => record_details.record,
        _ => {
            return Ok(None);
        }
    };
    let entry = match record.entry().as_option() {
        Some(entry) => entry,
        None => {
            return Ok(None);
        }
    };
    let (zome_index, entry_index) = match record.action().entry_type() {
        Some(EntryType::App(AppEntryDef { zome_index, entry_index, .. })) => {
            (zome_index, entry_index)
        }
        _ => {
            return Ok(None);
        }
    };
    Ok(
        EntryTypes::deserialize_from_type(
            zome_index.clone(),
            entry_index.clone(),
            entry,
        )?,
    )
}

fn get_record_for_action(action_hash: &ActionHash) -> ExternResult<Option<Record>> {
    let record = match get_details(action_hash.clone(), GetOptions::default())? {
        Some(Details::Record(record_details)) => record_details.record,
        _ => {
            return Ok(None);
        }
    };
    return Ok(Some(record))
}

fn get_invitation_detail(action_hash:&ActionHash) -> ExternResult<Option<InvitationEntryInfo>> {
    if let Ok(Some(invite_record)) = get_record_for_action(action_hash){
        let invite_entry_info = invite::get_invitation_entry_info(invite_record)?;
            return Ok(Some(invite_entry_info));
    };
    return Ok(None)
}

fn get_invitation_detail_by_link_target(link_target:HoloHash<hash_type::AnyLinkable>) -> ExternResult<InvitationEntryInfo> {
    let input = GetInput::new(ActionHash::from(link_target).into(),GetOptions::default());
    if let Some(invite_record) = get(input.any_dht_hash, GetOptions::content())? {
        let invite_entry_info = invite::get_invitation_entry_info(invite_record)?;
        return Ok(invite_entry_info)
    }
    return Err(wasm_error!("Invite_entry_info not found"))

}
