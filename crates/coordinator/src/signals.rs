use hdk::prelude::*;
use hc_integrity_zome_invitations::{EntryTypes, LinkTypes};
use crate::invite::InvitationEntryInfo;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Signal {
    EntryCreated { action: SignedActionHashed, app_entry: EntryTypes },
    EntryUpdated {
        action: SignedActionHashed,
        app_entry: EntryTypes,
        original_app_entry: EntryTypes,
    },
    EntryDeleted { action: SignedActionHashed, original_app_entry: EntryTypes },
    LinkCreated { action: SignedActionHashed, link_type: LinkTypes },
    LinkDeleted { action: SignedActionHashed, link_type: LinkTypes },
    InvitationAccepted {action: SignedActionHashed, data: InvitationEntryInfo },
    InvitationReceived {action: SignedActionHashed, data: InvitationEntryInfo},
    InvitationRejected {action: SignedActionHashed, data: InvitationEntryInfo},
}

//Not currently used.. 'type' is used from enum Signal
#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct SignalName;
impl SignalName {
    pub const INVITATION_RECEIVED: &'static str = "invitation received";
    pub const INVITATION_ACCEPTED: &'static str = "invitation accepted";
    pub const INVITATION_UPDATED: &'static str = "invitation updated";
    pub const INVITATION_REJECTED: &'static str = "invitation rejected";
}

pub fn invitation_received(action_data: SignedActionHashed, invite_detail:InvitationEntryInfo) -> ExternResult<bool> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;

    let signal: Signal = Signal::InvitationReceived {
        action: action_data,// SignalName::INVITATION_ACCEPTED.to_owned(), 
        data: invite_detail.clone()
    };

    let send_signal_to: Vec<AgentPubKey> = invite_detail
        .clone()
        .invitation
        .invitees
        .into_iter()
        .filter(|invitee| !AgentPubKey::from(invitee.clone()).eq(&my_pub_key))
        .map(|wrapped_agent_pub_key| wrapped_agent_pub_key.into())
        .collect();

    //let bump = ExternIO::encode(signal).unwrap();
    remote_signal(signal, send_signal_to)?;
    return Ok(true)
}

pub fn invitation_accepted(action_data: SignedActionHashed, invite_detail:InvitationEntryInfo) -> ExternResult<bool> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;

    let signal: Signal = Signal::InvitationAccepted {
        action: action_data,
        data: invite_detail.clone()
    };

    let mut send_signal_to: Vec<AgentPubKey> = invite_detail
        .clone()
        .invitation
        .invitees
        .into_iter()
        .filter(|invitee| !AgentPubKey::from(invitee.clone()).eq(&my_pub_key))
        .map(|agent_pub_key| agent_pub_key.into())
        .collect();

    send_signal_to.push(invite_detail.clone().invitation.inviter.into());
    //let bump = ExternIO::encode(signal).unwrap();
    remote_signal(signal, send_signal_to)?;
    return Ok(true)
}

pub fn invitation_rejected(action_data: SignedActionHashed, invite_detail:InvitationEntryInfo) -> ExternResult<bool> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;

    let signal: Signal = Signal::InvitationRejected {
        action: action_data,
        data: invite_detail.clone()
    };

    let mut send_signal_to: Vec<AgentPubKey> = invite_detail
        .clone()
        .invitation
        .invitees
        .into_iter()
        .filter(|invitee| !AgentPubKey::from(invitee.clone()).eq(&my_pub_key))
        .map(|agent_pub_key| agent_pub_key.into())
        .collect();

    send_signal_to.push(invite_detail.clone().invitation.inviter.into());
    remote_signal(signal, send_signal_to)?;
    return Ok(true)
}
