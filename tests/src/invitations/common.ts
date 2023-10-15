import { CallableCell } from '@holochain/tryorama';
import { ActionHash, AgentPubKey, Timestamp } from '@holochain/client';

export type Invite = {
  inviter: AgentPubKey,
  invitees: AgentPubKey[],
  location?: string,
  start_time?: Timestamp,
  end_time?: Timestamp,
  details?: Record<string, string>;
  timestamp: Timestamp
}

export type InviteInfo = {
  invitation: Invite,
  creation_hash: ActionHash,
  author: AgentPubKey
  timestamp: Timestamp
  invitees_who_accepted: AgentPubKey[],
  invitees_who_rejected: AgentPubKey[],
  invitees_pending: AgentPubKey[]

}

export type InviteInput = {
  invitees: AgentPubKey[],
  location?: string,
  start_time?: Timestamp,
  end_time?: Timestamp,
  details?: Record<string, string>;
  creation_hash?: ActionHash
}

export function getSampleInviteInput(inviteesInput: AgentPubKey[]): InviteInput {
  return { invitees: inviteesInput, location: "London" }
} 

export function getSampleInviteInputUpdate(inviteesInput: AgentPubKey[], first_hash:ActionHash): InviteInput {
  return { invitees: inviteesInput, location: "Amsterdam", start_time: Date.now(), end_time: Date.now()+86400, creation_hash: first_hash}
} 

export async function sendInvitations(cell: CallableCell, invitation:InviteInput): Promise<InviteInfo> {
  return cell.callZome({
    zome_name: "invitations",
    fn_name: "create_invitation",
    payload: invitation,
  });
}

export async function updateInvitation(cell: CallableCell, invitation:InviteInput): Promise<InviteInfo> {
  return cell.callZome({
    zome_name: "invitations",
    fn_name: "update_invitation",
    payload: invitation,
  });
}

export async function getPendingInvites(cell: CallableCell): Promise<InviteInfo[]> {
  return cell.callZome({
    zome_name: "invitations",
    fn_name: "get_my_pending_invitations",
    payload: null
  });
}

export async function getAllInvites(cell: CallableCell): Promise<InviteInfo[]> {
  return cell.callZome({
    zome_name: "invitations",
    fn_name: "get_all_my_invitations",
    payload: null
  });
}

export async function acceptInvite(cell: CallableCell, creationHash:ActionHash): Promise<ActionHash> {
  return cell.callZome({
    zome_name: "invitations",
    fn_name: "accept_invitation",
    payload: creationHash
  });
}

export async function rejectInvite(cell:CallableCell, creationHash: ActionHash): Promise<ActionHash> {
  return cell.callZome({
    zome_name: "invitations",
    fn_name: "reject_invitation",
    payload: creationHash
  });
}

export async function clearInvite(cell:CallableCell, creationHash: ActionHash): Promise<void> {
  return cell.callZome({
    zome_name: "invitations",
    fn_name: "clear_invitation",
    payload: creationHash
  });
}

