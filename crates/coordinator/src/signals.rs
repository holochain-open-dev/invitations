use hdk::prelude::*;
use crate::invite::InviteInfo;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Signal {
    InvitationAccepted {action: SignedActionHashed, data: InviteInfo },
    InvitationReceived {action: SignedActionHashed, data: InviteInfo},
    InvitationRejected {action: SignedActionHashed, data: InviteInfo},
    InvitationUpdated {action: SignedActionHashed, data: InviteInfo},

}

//broadcast
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

//broadcast
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

//only from invitee to inviter
pub fn invitation_accepted(action_data: SignedActionHashed, invite_detail:InviteInfo) -> ExternResult<bool> {
    let signal: Signal = Signal::InvitationAccepted {
        action: action_data,
        data: invite_detail.clone()
    };

    let send_signal_to: Vec<AgentPubKey> = vec![invite_detail.author];
    remote_signal(signal, send_signal_to)?;
    Ok(true)
}

//only from invitee to inviter
pub fn invitation_rejected(action_data: SignedActionHashed, invite_detail:InviteInfo) -> ExternResult<bool> {
    let signal: Signal = Signal::InvitationRejected {
        action: action_data,
        data: invite_detail.clone()
    };

    let send_signal_to: Vec<AgentPubKey> = vec![invite_detail.author];
    remote_signal(signal, send_signal_to)?;
    Ok(true)
}
