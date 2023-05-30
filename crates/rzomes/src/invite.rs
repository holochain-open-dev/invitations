use hdk::prelude::*;
use hc_zome_invitations::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct InviteesListInput(pub Vec<AgentPubKey>);

//This struct is an output object and contains helpfull information for the ui
#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct InvitationEntryInfo {
    pub invitation: Invite,
    pub invitation_entry_hash: EntryHash,
    pub invitation_creation_hash: ActionHash,
    pub invitees_who_accepted: Vec<AgentPubKey>,
    pub invitees_who_rejected: Vec<AgentPubKey>,
}

#[hdk_extern]
fn send_invitations(invitees_list: InviteesListInput) -> ExternResult<InvitationEntryInfo> {
  let agent_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;

  let invited_agents: Vec<AgentPubKey> = invitees_list
      .0
      .clone()
      .into_iter()
      .map(|agent_pub_key| AgentPubKey::from(agent_pub_key))
      .collect();
    
  //let now = sys_time()?.as_seconds_and_nanos();
  //let date_time = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(now.0, now.1).unwrap(), Utc);

  let invitation = Invite {
      invitees: invited_agents,
      inviter: AgentPubKey::from(agent_pub_key.clone()),
      timestamp: sys_time()?
    };
    let invitation_entry_hash: EntryHash = hash_entry(invitation.clone())?;

    let new_entry_hash = create_entry(&EntryTypes::Invite(invitation.clone()))?;
    let record: Record = get(new_entry_hash.clone(), GetOptions::default())?
        .ok_or(
            wasm_error!(
                WasmErrorInner::Guest(String::from("Could not find the newly created Invitation"))
            ),
        )?;

    create_link(
        agent_pub_key.clone(),
        invitation_entry_hash.clone(),
        LinkTypes::AgentToInvites,
        LinkTag::new(String::from("Invitee")),
    )?;

   for agent in invitees_list.0.clone().into_iter() {
        create_link(
            agent,
            invitation_entry_hash.clone(),
            LinkTypes::AgentToInvites,
            LinkTag::new(String::from("Invitee")),
        )?;
    }
    return Ok(get_invitation_entry_info(record)?);
}

#[hdk_extern] //TODO: too many network calls..link from Action not entry
pub fn get_my_pending_invitations(_: ()) -> ExternResult<Vec<InvitationEntryInfo>> {
    let agent: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let mut pending_invitations: Vec<InvitationEntryInfo> = vec![];

    let links = get_links(agent, LinkTypes::AgentToInvites, Some(LinkTag::new("Invitee")))?;
    let get_input: Vec<GetInput> = links
        .into_iter()
        .map(|link| GetInput::new(
            EntryHash::from(link.target).into(),
            GetOptions::default(),
        ))
        .collect();
    let records: Vec<Record> = HDK
        .with(|hdk| hdk.borrow().get(get_input))?
        .into_iter()
        .filter_map(|r| r)
        .collect();
    //Ok(records)
    for record in records.into_iter() {
            let invitation_info = get_invitation_entry_info(record);
            pending_invitations.push(invitation_info?); 
    }
    Ok(pending_invitations)
}


#[hdk_extern] //link to the actionhash not the entryhash?
pub fn accept_invitation(invitation_creation_hash: ActionHash) -> ExternResult<bool> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let record = get(invitation_creation_hash, GetOptions::default())?
    .ok_or(
        wasm_error!(
            WasmErrorInner::Guest(String::from("Could not find the Invitation action"))
        ),
    )?;
    let entry_info =
        get_invitation_entry_info(record.clone())?;

    // we will check if the agent attempting to accept this invitation is an invitee
    if entry_info
        .invitation
        .invitees
        .contains(&AgentPubKey::from(my_pub_key.clone()))
    {
        create_link(
            entry_info.invitation_entry_hash.clone(), //action hash
            my_pub_key.clone(),
            LinkTypes::InviteToMembers,
            LinkTag::new(String::from("Accepted")),
        )?;
        return Ok(true)
    }

    return Ok(false)
}

#[hdk_extern]
pub fn reject_invitation(invitation_action_hash: ActionHash) -> ExternResult<bool> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;
    let record = get(invitation_action_hash, GetOptions::default())?
    .ok_or(
        wasm_error!(
            WasmErrorInner::Guest(String::from("Could not find the Invitation action"))
        ),
    )?;
    let entry_info =
        get_invitation_entry_info(record.clone())?;

    // we will check if the agent attempting to accept this invitation is an invitee
    if entry_info
        .invitation
        .invitees
        .contains(&AgentPubKey::from(my_pub_key.clone()))
    {
        create_link(
            entry_info.invitation_entry_hash,
            my_pub_key,
            LinkTypes::InviteToMembers,
            LinkTag::new(String::from("Rejected")),
        )?;
        return Ok(true)
    }

    return Ok(false)
}


#[hdk_extern]
pub fn clear_invitation(invitation_entry_hash: EntryHash) -> ExternResult<bool> {
    let links = get_links(
        agent_info()?.agent_latest_pubkey, 
        LinkTypes::AgentToInvites,
        Some(LinkTag::new("Invitee")),
    )?;

    links
        .into_iter()
        .filter(|link| link.target == HoloHash::from(invitation_entry_hash.clone()))
        .map(|link_to_invitation| -> ExternResult<()> {
            delete_link(link_to_invitation.create_link_hash)?;
            Ok(())
        })
        .collect::<ExternResult<Vec<()>>>()?;

    return Ok(true);
}

//HELPERS - we expect the Record to have an entry
pub fn get_invitation_entry_info(invite: Record) -> ExternResult<InvitationEntryInfo> {
    let invite_action_hash = invite.signed_action.action_address();
       //debug!("invite record: {:?}",invite.entry);
    let invitation: Invite = invite.entry.clone().to_app_option().map_err(|e| wasm_error!(e))?.ok_or(
        wasm_error!(
            WasmErrorInner::Guest(String::from("Could not find Invitation for hash in invitation details "))
        ),
    )?;
    let invite_entry_hash = hash_entry(invitation.clone())?;
//    .ok_or_else(|| {
 //       WasmError::Guest("we dont found the invitation entry for the given hash".into())
 //   })?;
    let invitees_who_accepted: Vec<AgentPubKey> = get_links(
        invite_entry_hash.clone(),
        LinkTypes::InviteToMembers,
        Some(LinkTag::new("Accepted")),
    )?.into_iter()
    .map(|link| AgentPubKey::from(EntryHash::from(link.target)))
    .collect();

    let invitees_who_rejected: Vec<AgentPubKey> = get_links(
        invite_entry_hash.clone(),
        LinkTypes::InviteToMembers,
        Some(LinkTag::new("Rejected")),
    )?.into_iter()
    .map(|link| AgentPubKey::from(EntryHash::from(link.target)))
    .collect();
        
    return Ok(InvitationEntryInfo {
        invitation: invitation.clone(),
        invitation_entry_hash: hash_entry(invitation)?,
        invitation_creation_hash: invite_action_hash.clone(),
        invitees_who_accepted,
        invitees_who_rejected
    })
}
