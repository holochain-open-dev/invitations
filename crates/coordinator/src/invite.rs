use hdk::prelude::*;
use hc_integrity_zome_invitations::*;
use std::collections::BTreeMap;


#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct InviteInput {
    pub invitees: Vec<AgentPubKey>,
    pub location: Option<String>,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
    pub details: Option<BTreeMap<String,String>>,
    pub original_hash: Option<ActionHash>
}

//This struct is an output object and contains helpfull information for the ui
#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct InviteInfo {
    pub invitation: Invite,
    pub invitation_original_hash: ActionHash,
   // pub timestamp: Timestamp,
   // pub author: AgentPubKey,
    pub invitees_who_accepted: Vec<AgentPubKey>,
    pub invitees_who_rejected: Vec<AgentPubKey>,
    pub invitees_pending:Vec<AgentPubKey>,
}

#[hdk_extern]
fn create_invitation(input: InviteInput) -> ExternResult<InviteInfo> {
  let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;

  let invitation = Invite {
      inviter: AgentPubKey::from(my_pub_key.clone()),
      invitees: input.invitees.clone(),
      location: input.location,
      start_time: input.start_time,
      end_time: input.end_time,
      details: input.details,
      timestamp: sys_time()?
    };

    let action_hash = create_entry(&EntryTypes::Invite(invitation.clone()))?;
    let record: Record = get(action_hash.clone(), GetOptions::default())?.ok_or(wasm_error!(WasmErrorInner::Memory))?;

   for agent in input.invitees.clone().into_iter(){
        create_link(
            agent,
            action_hash.clone(),
            LinkTypes::AgentToInvite,
            LinkTag::new(String::from("pending")),
        )?;
    }

    if !input.invitees.contains(&my_pub_key){
        create_link(
            my_pub_key.clone(),
            action_hash.clone(),
            LinkTypes::AgentToInvite,
            LinkTag::new(String::from("inviter")),
        )?;
    }
    return Ok(get_invitation_info(record, &action_hash)?);
}


#[hdk_extern]
pub fn update_invitation(invitation: InviteInput) -> ExternResult<bool> {
    let my_pub_key = agent_info()?.agent_latest_pubkey;

    let hash_result = match invitation.original_hash {
        Some(hash) => hash,
        None => return Err(wasm_error!(WasmErrorInner::Guest(
            "Cannot find original action hash to update Invite entry".to_string()
        )))?
    };
        
    let last_invite_record = get_latest_record(hash_result)?;
    let last_invite: Invite = last_invite_record
        .entry()
        .to_app_option()
        .map_err(|e| wasm_error!(e))?
        .ok_or(wasm_error!(WasmErrorInner::Guest(
            "Previous invite is malformed".to_string()
        )))?;

    if my_pub_key != last_invite.inviter{
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Only the author can update an invite".into(),
        )))?;
    }

    let updated_invite = Invite {
        inviter: last_invite.inviter,
        invitees: invitation.invitees.clone(), //change invitees
        location: invitation.location, 
        start_time: invitation.start_time,
        end_time: invitation.end_time,
        details: invitation.details,
        timestamp: sys_time()?
      };
    update_entry(last_invite_record.action_address().clone(), &updated_invite)?;
    return Ok(true);
}


#[hdk_extern]
pub fn get_my_pending_invitations(_: ()) -> ExternResult<Option<Vec<InviteInfo>>> {
    let agent: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let links = get_links(agent, LinkTypes::AgentToInvite, Some(LinkTag::new("pending")))?;
    if !links.is_empty(){
        let pending_invitations = get_invite_info_from_links(links)?;
        Ok(Some(pending_invitations))
    }else {
        Ok(None)
    }
}


#[hdk_extern]
pub fn get_all_my_invitations(_: ()) -> ExternResult<Option<Vec<InviteInfo>>> {
    let agent: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let links = get_links(agent, LinkTypes::AgentToInvite,None)?;
    if !links.is_empty(){
        let all_invitations = get_invite_info_from_links(links)?;
        Ok(Some(all_invitations))
    }else {
        Ok(None)
    }
}


#[hdk_extern]
pub fn accept_invitation(original_action_hash: ActionHash) -> ExternResult<bool> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let record = get(original_action_hash.clone(), GetOptions::default())?
    .ok_or(
        wasm_error!(
            WasmErrorInner::Guest(String::from("Could not find the Invitation action"))
        ),
    )?;
    let entry_info =
        get_invitation_info(record.clone(), &original_action_hash)?;

    // we will check if the agent attempting to accept this invitation is an invitee
    if entry_info
        .invitation
        .invitees
        .contains(&AgentPubKey::from(my_pub_key.clone()))
    {
         create_link(
            entry_info.invitation_original_hash.clone(), //action hash
            my_pub_key.clone(),
            LinkTypes::InviteToAgent,
            LinkTag::new(String::from("accepted")),
        )?;
        commit_invitation(entry_info.invitation_original_hash)?;
        return Ok(true)
    }

    return Ok(false)
}


#[hdk_extern]
pub fn reject_invitation(original_action_hash: ActionHash) -> ExternResult<bool> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let record = get(original_action_hash.clone(), GetOptions::default())?
    .ok_or(
        wasm_error!(
            WasmErrorInner::Guest(String::from("Could not find the Invitation action"))
        ),
    )?;
    let entry_info =
        get_invitation_info(record.clone(), &original_action_hash)?;

    // we will check if the agent attempting to accept this invitation is an invitee
    if entry_info
        .invitation
        .invitees
        .contains(&AgentPubKey::from(my_pub_key.clone()))
    {
        create_link(
            entry_info.invitation_original_hash.clone(),
            my_pub_key,
            LinkTypes::InviteToAgent,
            LinkTag::new(String::from("rejected")),
        )?;
        commit_invitation(entry_info.invitation_original_hash)?;

        return Ok(true)
    }

    return Ok(false)
}

//helpers

fn get_invite_info_from_links(links: Vec<Link>) -> ExternResult<Vec<InviteInfo>> {
    let original_action_hash = ActionHash::try_from(links[0].clone().target).unwrap();

    let get_input: Vec<GetInput> = links
        .into_iter()
        .map(|link| GetInput::new(ActionHash::try_from(link.target).unwrap().into(),GetOptions::default()))
        .collect();
    let records: Vec<Record> = HDK
        .with(|hdk| hdk.borrow().get(get_input))?
        .into_iter()
        .filter_map(|r| r)
        .collect();

    let mut invitations: Vec<InviteInfo> = vec![];
    for record in records.into_iter() {
        let invitation_info = get_invitation_info(record,&original_action_hash);
        invitations.push(invitation_info?); 
    }
    Ok(invitations)
}

//no option to update link tags, so we delete and create a new link
fn commit_invitation(original_action_hash: ActionHash) -> ExternResult<bool> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let links = get_links(
        agent_info()?.agent_latest_pubkey, 
        LinkTypes::AgentToInvite,
        Some(LinkTag::new("pending")),
    )?;

    links
        .into_iter()
        .filter(|link| link.target == HoloHash::from(original_action_hash.clone()))
        .map(|link_to_invitation| -> ExternResult<()> {
            delete_link(link_to_invitation.create_link_hash)?;
            Ok(())
        })
        .collect::<ExternResult<Vec<()>>>()?;
    
    create_link(
        my_pub_key,
        original_action_hash.clone(),
        LinkTypes::AgentToInvite,
        LinkTag::new(String::from("commited")),
    )?;
    return Ok(true);
}


fn get_latest_record(action_hash: ActionHash) -> ExternResult<Record> {
    let details = get_details(action_hash, GetOptions::default())?.ok_or(wasm_error!(
        WasmErrorInner::Guest("invite not found".into())
    ))?;

    match details {
        Details::Entry(_) => Err(wasm_error!(WasmErrorInner::Guest(
            "Malformed details".into()
        ))),
        Details::Record(element_details) => match element_details.updates.last() {
            Some(update) => get_latest_record(update.action_address().clone()),
            None => Ok(element_details.record),
        },
    }
}

pub fn get_invitation_info(invite: Record, original_action_hash: &ActionHash) -> ExternResult<InviteInfo> {
    let invitation_entry: Invite = invite.entry.clone().to_app_option().map_err(|e| wasm_error!(e))?.ok_or(
        wasm_error!(
            WasmErrorInner::Guest(String::from("Could not find Invitation for hash in invitation details "))
        ),
    )?;
    
    let invitees_who_accepted: Vec<AgentPubKey> = get_links(
        original_action_hash.clone(),
        LinkTypes::InviteToAgent,
        Some(LinkTag::new("accepted")),
    )?.into_iter()
    .map(|link| AgentPubKey::try_from(link.target).unwrap())
    .collect();

    let invitees_who_rejected: Vec<AgentPubKey> = get_links(
        original_action_hash.clone(),
        LinkTypes::InviteToAgent,
        Some(LinkTag::new("rejected")),
    )?.into_iter()
    .map(|link| AgentPubKey::try_from(link.target).unwrap())
    .collect();

    let mut invitees_pending: Vec<AgentPubKey> = invitation_entry.invitees.clone();
    invitees_pending.retain(|x| !invitees_who_accepted.contains(&x) && !invitees_who_rejected.contains(&x));
   
    return Ok(InviteInfo {
        invitation: invitation_entry.clone(),
        invitation_original_hash: original_action_hash.clone(),
        invitees_who_accepted,
        invitees_who_rejected,
        invitees_pending
    })
}

/*fn get_entry_for_action(action_hash: &ActionHash) -> ExternResult<Option<EntryTypes>> {
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
}*/