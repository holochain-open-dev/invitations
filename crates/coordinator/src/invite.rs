use hdk::prelude::*;
use hc_integrity_zome_invitations::*;

#[hdk_extern]
fn create_invitation(input: InviteInput) -> ExternResult<InviteInfo> {
  let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;

  let invitation = Invite {
      invitees: input.invitees.clone(),
      location: input.location,
      start_time: input.start_time,
      end_time: input.end_time,
      details: input.details,
    };

    let action_hash = create_entry(&EntryTypes::Invite(invitation.clone()))?;

   for agent in input.invitees.clone().into_iter(){
        create_link(
            agent,
            action_hash.clone(),
            LinkTypes::AgentToInvite,
            LinkTag::new(String::from("pending")),
        )?;
    }

    //creator of the invitation is not an invitee
    if !input.invitees.contains(&my_pub_key){
        create_link(
            my_pub_key.clone(),
            action_hash.clone(),
            LinkTypes::AgentToInvite,
            LinkTag::new(String::from("inviter")),
        )?;
    }
    return Ok(get_invitation_info(&action_hash)?);
}


#[hdk_extern]
pub fn update_invitation(invitation: InviteInput) -> ExternResult<ActionHash> {

    let hash_result = match invitation.creation_hash {
        Some(hash) => hash,
        None => return Err(wasm_error!(WasmErrorInner::Guest(
            "Cannot find original action hash to update Invite entry".to_string()
        )))?
    };
    let last_invite_record = get_latest_record(hash_result)?;

    let updated_invite = Invite {
        invitees: invitation.invitees.clone(), //change invitees?
        location: invitation.location, 
        start_time: invitation.start_time,
        end_time: invitation.end_time,
        details: invitation.details,
      };
    let update_hash = update_entry(last_invite_record.action_address().clone(), &updated_invite)?;
    return Ok(update_hash);
}


#[hdk_extern]
pub fn get_my_pending_invitations(_: ()) -> ExternResult<Option<Vec<InviteInfo>>> {
    let agent: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let links = get_links(agent, LinkTypes::AgentToInvite, Some(LinkTag::new("pending")))?;
    Ok(get_invite_info_from_links(links)?)
}


#[hdk_extern]
pub fn get_all_my_invitations(_: ()) -> ExternResult<Option<Vec<InviteInfo>>> {
    let agent: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let links = get_links(agent, LinkTypes::AgentToInvite,None)?;
    Ok(get_invite_info_from_links(links)?)
}


#[hdk_extern]
pub fn accept_invitation(original_action_hash: ActionHash) -> ExternResult<ActionHash> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let entry_info = get_invitation_info( &original_action_hash)?;

    create_link(
        entry_info.creation_hash.clone(), //action hash
        my_pub_key.clone(),
        LinkTypes::InviteToAgent,
        LinkTag::new(String::from("accepted")),
    )?;
    let committed_link_hash = commit_invitation(entry_info.creation_hash)?;
    return Ok(committed_link_hash)
}


#[hdk_extern]
pub fn reject_invitation(original_action_hash: ActionHash) -> ExternResult<ActionHash> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let entry_info = get_invitation_info(&original_action_hash)?;
    create_link(
        entry_info.creation_hash.clone(),
        my_pub_key,
        LinkTypes::InviteToAgent,
        LinkTag::new(String::from("rejected")),
    )?;
    let committed_link_hash = commit_invitation(entry_info.creation_hash)?;
    return Ok(committed_link_hash)
}

#[hdk_extern]
pub fn clear_invitation(original_action_hash: ActionHash) -> ExternResult<()> {
    let links = get_links(
        agent_info()?.agent_latest_pubkey, 
        LinkTypes::AgentToInvite,
        None,
    )?;

    links
        .into_iter()
        .filter(|link| link.target == HoloHash::from(original_action_hash.clone()))
        .map(|link_to_invitation| -> ExternResult<()> {
            delete_link(link_to_invitation.create_link_hash)?;
            Ok(())
        })
        .collect::<ExternResult<Vec<()>>>()?;
    return Ok(())
}




//************ Helpers **************************

fn get_invite_info_from_links(links: Vec<Link>) -> ExternResult<Option<Vec<InviteInfo>>> {
    if !links.is_empty(){
        let mut invitations: Vec<InviteInfo> = vec![];
        for link in links.into_iter() {
            let original_action_hash = ActionHash::try_from(link.target).map_err(|err| wasm_error!(err))?;
            let invitation_info = get_invitation_info(&original_action_hash);
            invitations.push(invitation_info?); 
        }
        return Ok(Some(invitations))
    }
    Ok(None)
}

//no option to update link tags, so we delete and create a new link
fn commit_invitation(original_action_hash: ActionHash) -> ExternResult<ActionHash> {
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
    
    let committed_link = create_link(
        my_pub_key,
        original_action_hash.clone(),
        LinkTypes::AgentToInvite,
        LinkTag::new(String::from("commited")),
    )?;
    return Ok(committed_link);
}


// from the details of an Action we cycle up the chain of Record updates to get the latest one
// with some reference to the latest Action (link or field entry) this function could be avoided 
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

//we cycle down the chain of records to find the first/genesis record's Create action.
//if all the Action::Updates retained a copy of the genesis create action .. we could remove this function and just get that
fn get_creation_action_hash(invite_record:&Record) -> ExternResult<ActionHash> {
    if let ActionType::Create = invite_record.action().action_type(){
        return Ok(invite_record.action_address().clone())
    }
    else {
        if let Action::Update(update) = invite_record.action(){
            let previous_record = get(update.original_action_address.clone(),GetOptions::default())?.ok_or(
                wasm_error!(
                WasmErrorInner::Guest(String::from("Could not find the Invitation record"))),
            )?;
        return get_creation_action_hash(&previous_record);
    } else {
        Err(wasm_error!(
            WasmErrorInner::Guest(String::from("something is seriously wrong "))
        ))
        }
    }
}

pub fn get_invitation_info(original_action_hash: &ActionHash) -> ExternResult<InviteInfo> {
    let invite_record = get(original_action_hash.clone(), GetOptions::default())?
    .ok_or(wasm_error!(WasmErrorInner::Guest(String::from("Could not find the Invitation action"))))?;

    let invitation_entry: Invite = invite_record.entry.clone().to_app_option().map_err(|e| wasm_error!(e))?
    .ok_or(wasm_error!(WasmErrorInner::Guest(String::from("Could not find Invitation for hash in invitation details "))))?;
    
    return get_invitation_info_details(invitation_entry, invite_record, original_action_hash)
}

//different path for updates
pub fn get_invitation_update_info(update_action_hash: &ActionHash) -> ExternResult<InviteInfo> {
    let invite_record = get(update_action_hash.clone(), GetOptions::default())?
    .ok_or(wasm_error!(WasmErrorInner::Guest(String::from("Could not find the Invitation action"))))?;

    let invite_entry: Invite = invite_record.entry.clone().to_app_option().map_err(|e| wasm_error!(e))?
    .ok_or(wasm_error!(WasmErrorInner::Guest(String::from("Could not find Invitation for hash in invitation details "))))?;

    //get creation hash from cycling update chain...
    let creation_action_hash = get_creation_action_hash(&invite_record)?;
    return get_invitation_info_details(invite_entry, invite_record, &creation_action_hash)
}

//DTO for all returns and signals
pub fn get_invitation_info_details(invite:Invite, invite_record: Record, first_action_hash: &ActionHash)-> ExternResult<InviteInfo> {
    let invitees_who_accepted: Vec<AgentPubKey> = get_links(
        first_action_hash.clone(),
        LinkTypes::InviteToAgent,
        Some(LinkTag::new("accepted")),
    )?.into_iter()
    .map(|link| AgentPubKey::try_from(link.target).unwrap())
    .collect();

    let invitees_who_rejected: Vec<AgentPubKey> = get_links(
        first_action_hash.clone(),
        LinkTypes::InviteToAgent,
        Some(LinkTag::new("rejected")),
    )?.into_iter()
    .map(|link| AgentPubKey::try_from(link.target).unwrap())
    .collect();

    let mut invitees_pending: Vec<AgentPubKey> = invite.invitees.clone();
    invitees_pending.retain(|x| !invitees_who_accepted.contains(&x) && !invitees_who_rejected.contains(&x));
   
    return Ok(InviteInfo {
        invitation: invite.clone(),
        creation_hash: first_action_hash.clone(),
        author: invite_record.action().author().clone(),
        timestamp: invite_record.action().timestamp(),
        invitees_who_accepted,
        invitees_who_rejected,
        invitees_pending
    })
}

//when to use get_details or get for record retrival?

/*fn get_record_for_action(action_hash: &ActionHash) -> ExternResult<Option<Record>> {
    let record = match get_details(action_hash.clone(), GetOptions::default())? {
        Some(Details::Record(record_details)) => record_details.record,
        _ => {
            return Ok(None);
        }
    };
    return Ok(Some(record))
}*/