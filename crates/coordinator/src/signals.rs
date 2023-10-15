use hdk::prelude::*;
use hc_integrity_zome_invitations::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Signal {
    InvitationAccepted {action: SignedActionHashed, data: InviteInfo },
    InvitationReceived {action: SignedActionHashed, data: InviteInfo},
    InvitationRejected {action: SignedActionHashed, data: InviteInfo},
    InvitationUpdated {action: SignedActionHashed, data: InviteInfo},

}

//broadcast to everyone inviter
pub fn invitation_received(action_data: SignedActionHashed, invite_detail:InviteInfo) -> ExternResult<bool> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;

    let signal: Signal = Signal::InvitationReceived {
        action: action_data, 
        data: invite_detail.clone()
    };

    let send_signal_to: Vec<AgentPubKey> = invite_detail
        .clone()
        .invitation
        .invitees
        .into_iter()
        .filter(|invitee| !AgentPubKey::from(invitee.clone()).eq(&my_pub_key))
        .map(|agent_pub_key| agent_pub_key.into())
        .collect();

    remote_signal(signal, send_signal_to)?;
    Ok(true)
}

//broadcast to everyone except updater - consider an emit_signal for UI
pub fn invitation_updated(action_data: SignedActionHashed, invite_detail:InviteInfo) -> ExternResult<bool> {
    let my_pub_key: AgentPubKey = agent_info()?.agent_latest_pubkey;

    let signal: Signal = Signal::InvitationUpdated {
        action: action_data,
        data: invite_detail.clone()
    };

    let send_signal_to: Vec<AgentPubKey> = invite_detail
        .clone()
        .invitation
        .invitees
        .into_iter()
        .filter(|invitee| !AgentPubKey::from(invitee.clone()).eq(&my_pub_key))
        .map(|agent_pub_key| agent_pub_key.into())
        .collect();

    remote_signal(signal, send_signal_to)?;
    return Ok(true)
}

//signal only from invitee to inviter to avoid group noise
pub fn invitation_accepted(action_data: SignedActionHashed, invite_detail:InviteInfo) -> ExternResult<bool> {
    let signal: Signal = Signal::InvitationAccepted {
        action: action_data,
        data: invite_detail.clone()
    };

    let send_signal_to: Vec<AgentPubKey> = vec![invite_detail.author];
    remote_signal(signal, send_signal_to)?;
    Ok(true)
}

//signal only from invitee to inviter to avoid group noise
pub fn invitation_rejected(action_data: SignedActionHashed, invite_detail:InviteInfo) -> ExternResult<bool> {
    let signal: Signal = Signal::InvitationRejected {
        action: action_data,
        data: invite_detail.clone()
    };

    let send_signal_to: Vec<AgentPubKey> = vec![invite_detail.author];
    remote_signal(signal, send_signal_to)?;
    Ok(true)
}
