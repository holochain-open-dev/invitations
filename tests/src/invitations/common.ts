import { CallableCell } from '@holochain/tryorama';
import { NewEntryAction, ActionHash, Record, AppBundleSource, fakeActionHash, fakeAgentPubKey, fakeEntryHash, fakeDnaHash, AgentPubKey, EntryHash, Action, Timestamp } from '@holochain/client';

export type Invite = {
  inviter: AgentPubKey,
  invitees: AgentPubKey[],
  location?: string,
  start_time?: Timestamp,
  end_time?: Timestamp,
  timestamp: Timestamp
}

export type InviteInfo = {
  invitation: Invite,
  invitation_creation_hash: ActionHash,
  invitees_who_accepted: AgentPubKey[],
  invitees_who_rejected: AgentPubKey[]
}

export type InviteInput = {
  invitees: AgentPubKey[],
  location?: string,
  start_time?: Timestamp,
  end_time?: Timestamp,
  original_hash?: ActionHash
}

export function getSampleInviteInput(inviteesInput: AgentPubKey[]): InviteInput {
  return { invitees: inviteesInput, location: "london" }
} 

export function getSampleInviteInputUpdate(inviteesInput: AgentPubKey[], original_hash:ActionHash): InviteInput {
  return { invitees: inviteesInput, location: "Amsterdam", start_time: Date.now(), end_time: Date.now()+86400, original_hash: original_hash}
} 

export async function sendInvitations(cell: CallableCell, invitation:InviteInput): Promise<InviteInfo> {
  return cell.callZome({
    zome_name: "invitations",
    fn_name: "send_invitations",
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

export async function acceptInvite(cell: CallableCell, creationHash:ActionHash): Promise<boolean> {
  return cell.callZome({
    zome_name: "invitations",
    fn_name: "accept_invitation",
    payload: creationHash
  });
}

export async function rejectInvite(cell:CallableCell, creationHash: ActionHash): Promise<boolean> {
  return cell.callZome({
    zome_name: "invitations",
    fn_name: "reject_invitation",
    payload: creationHash
  });
}

export async function clearInvite(cell:CallableCell, creationHash: ActionHash): Promise<boolean> {
  return cell.callZome({
    zome_name: "invitations",
    fn_name: "clear_invitation",
    payload: creationHash
  });
}

