import { assert, test } from "vitest";

import { runScenario, pause, CallableCell, dhtSync, runLocalServices, createConductor, enableAndGetAgentApp, stopLocalServices, cleanAllConductors } from '@holochain/tryorama';
import { NewEntryAction, ActionHash, Record, AppBundleSource, fakeDnaHash, fakeActionHash, fakeAgentPubKey, fakeEntryHash, AppSignalCb, AppSignal, RecordEntry, AppWebsocket } from '@holochain/client';
import { decode } from '@msgpack/msgpack';

import { acceptInvite, clearInvite, getAllInvites, getPendingInvites, getSampleInviteInput, getSampleInviteInputUpdate, InviteInfo, rejectInvite, sendInvitations, updateInvitation } from './common.js';

const path_to_happ = '/../workdir/happ/invitations.happ'

test('6. try to update an invite without being the author', async () => {
  await runScenario(async scenario => {
    // Construct proper paths for your app.
    // This assumes app bundle created by the `hc app pack` command.
    const testAppPath = process.cwd() + path_to_happ;

    // Set up the app to be installed 
    const appSource_alice = { appBundleSource: { path: testAppPath }}
    const appSource_bob = { appBundleSource: { path: testAppPath }}

     // Add 2 players with the test app to the Scenario. The returned players
    // can be destructured.
    const [alice,bob] = await scenario.addPlayersWithApps([appSource_alice,appSource_bob]);

    // Shortcut peer discovery through gossip and register all agents in every
    // conductor of the scenario.
    await scenario.shareAllAgents();
    console.log("\n************************* START TEST ****************************\n")

    console.log("\nAlice creates an Invite")
    const invite_detail: InviteInfo = await sendInvitations(alice.cells[0], getSampleInviteInput([bob.agentPubKey,alice.agentPubKey]));
    //console.log(invite_detail)//decode((record.entry as any).Present.entry as any))
    assert.ok(invite_detail);

    // Wait for the created entry to be propagated to the other node.
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    console.log("Bob gets his pending invites")
    const invite_list_bob: InviteInfo[] = await getPendingInvites(bob.cells[0])
    console.log(invite_list_bob)
    assert.isNotEmpty(invite_list_bob)

    console.log("Bob trys to update the invite")
    const invite_update = getSampleInviteInputUpdate([bob.agentPubKey,alice.agentPubKey],invite_detail.creation_hash)
    var invite_update_list_bob : null | InviteInfo = null
    try {
       var invite_update_list_bob: InviteInfo = await updateInvitation(bob.cells[0],invite_update)
    } catch (e:any){
      console.log(e)
    }
    assert.isNull(invite_update_list_bob)
  
  });
});

test('7. try to accept an invite without being an invitee', async () => {
  await runScenario(async scenario => {
    // Construct proper paths for your app.
    // This assumes app bundle created by the `hc app pack` command.
    const testAppPath = process.cwd() + path_to_happ;

    // Set up the app to be installed 
    const appSource_alice = { appBundleSource: { path: testAppPath }}
    const appSource_bob = { appBundleSource: { path: testAppPath }}

     // Add 2 players with the test app to the Scenario. The returned players
    // can be destructured.
    const [alice,bob] = await scenario.addPlayersWithApps([appSource_alice,appSource_bob]);

    // Shortcut peer discovery through gossip and register all agents in every
    // conductor of the scenario.
    await scenario.shareAllAgents();
    console.log("\n************************* START TEST ****************************\n")

    console.log("\nAlice creates an Invite without being an invitee")
    const invite_detail: InviteInfo = await sendInvitations(alice.cells[0], getSampleInviteInput([bob.agentPubKey]));
    //console.log(invite_detail)//decode((record.entry as any).Present.entry as any))
    assert.ok(invite_detail);

    console.log("Alice trys to accept the invite")
    var result : null | ActionHash = null
    try {
      var result: ActionHash = await acceptInvite(alice.cells[0],invite_detail.creation_hash)
    } catch (e:any){
      console.log(e)
    }
    assert.isNull(result)
  
  });
});

