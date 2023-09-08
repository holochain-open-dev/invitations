import { assert, test } from "vitest";

import { runScenario, pause, CallableCell } from '@holochain/tryorama';
import { NewEntryAction, ActionHash, Record, AppBundleSource, fakeDnaHash, fakeActionHash, fakeAgentPubKey, fakeEntryHash, AppSignalCb, AppSignal, RecordEntry } from '@holochain/client';
import { decode } from '@msgpack/msgpack';

import { acceptInvite, clearInvite, getPendingInvites, getSampleInviteInput, getSampleInviteInputUpdate, InviteInfo, rejectInvite, sendInvitations, updateInvitation } from './common.js';

const path_to_happ = '/../workdir/happ/invitations-test.happ'

test('1. create and compare invitation lists', async () => {
  await runScenario(async scenario => {
    // Construct proper paths for your app.
    // This assumes app bundle created by the `hc app pack` command.
    const testAppPath = process.cwd() + path_to_happ;

    // Set up the app to be installed 
    const appSource_alice = { appBundleSource: { path: testAppPath }}
    const appSource_bob = { appBundleSource: { path: testAppPath }}

     // Add 2 players with the test app to the Scenario. The returned players
    // can be destructured.
    const [alice] = await scenario.addPlayersWithApps([appSource_alice]);
    const [bob] = await scenario.addPlayersWithApps([appSource_bob]);

    // Shortcut peer discovery through gossip and register all agents in every
    // conductor of the scenario.
    await scenario.shareAllAgents();
    console.log("\n************************* START TEST ****************************\n")

    console.log("\nAlice creates an Invite")
    const invite_detail: InviteInfo = await sendInvitations(alice.cells[0], getSampleInviteInput([bob.agentPubKey]));
    //console.log(invite_detail)//decode((record.entry as any).Present.entry as any))
    assert.ok(invite_detail);

    // Wait for the created entry to be propagated to the other node.
    await pause(1200);

    console.log("Bob gets his pending invites")
    const invite_list_bob: InviteInfo[] = await getPendingInvites(bob.cells[0])
    console.log(invite_list_bob)
    assert.isNotEmpty(invite_list_bob)
  
    console.log("Alice gets her pending Invites")
    const invite_list_alice: InviteInfo[] = await getPendingInvites(alice.cells[0])
    console.log(invite_list_alice)
    assert.deepEqual(invite_list_bob,invite_list_alice)
  
  });
});

test('2. create and accept Invite', async () => {
  await runScenario(async scenario => {
    
    // setup signal receivers
    let processSignal_alice: AppSignalCb | undefined;
    let signalReceived_alice = new Promise<AppSignal>((resolve) => {
      processSignal_alice = (signal) => {
        console.log("signal found for Alice:",signal)
        resolve(signal);
      };
    });

    let processSignal_bob: AppSignalCb | undefined;
    let signalReceived_bob = new Promise<AppSignal>((resolve) => {
      processSignal_bob = (signal) => {
        console.log("signal found for bob:",signal)
        resolve(signal);
      };
    });
    
    // Construct proper paths for your app.
    // This assumes app bundle created by the `hc app pack` command.
    const testAppPath = process.cwd() + path_to_happ;

    // Set up the app to be installed 
    const appSource_alice = { appBundleSource: { path: testAppPath }}//, options: {signalHandler: processSignal_alice} };
    const appSource_bob = { appBundleSource: { path: testAppPath }}


    // Add 2 players with the test app to the Scenario. The returned players
    // can be destructured.
    const [alice] = await scenario.addPlayersWithApps([appSource_alice]);
    const [bob] = await scenario.addPlayersWithApps([appSource_bob]);

    alice.conductor.appWs().on("signal",processSignal_alice)
    bob.conductor.appWs().on("signal",processSignal_bob)

    // Shortcut peer discovery through gossip and register all agents in every
    // conductor of the scenario.
    await scenario.shareAllAgents();

    console.log("\n************************* START TEST ****************************\n")
    console.log("\nAlice creates an Invite to Bob\n")
    const invite_detail: InviteInfo = await sendInvitations(alice.cells[0],getSampleInviteInput([bob.agentPubKey]));
    assert.ok(invite_detail);

    await pause(1200);
    const bob_signal = await signalReceived_bob
    
    console.log("Bob sees he has been signalled a new Invite:\n",bob_signal.payload['data'])
    assert.equal(bob_signal.payload['type'], 'InvitationReceived')

    console.log("\nBob accepts the invite\n")  //this would be better to get back the action hash from the createlink
    const result: boolean = await acceptInvite(bob.cells[0],bob_signal.payload['data'].invitation_original_hash)
    console.log(result)
    assert.isTrue(result)

    await pause(1200);
    let alice_signal = await signalReceived_alice

    assert.equal(alice_signal.payload['type'],'InvitationAccepted',"message should be of type accepted")
    let invitees = alice_signal.payload['data'].invitees_who_accepted
    assert.deepEqual(invitees[0], bob.agentPubKey, "Bob was not found in the accepted invitees")

    console.log("Alice sees Bob has accepted the invite via a signal and checks the invite status\n") //todo react to accept signal
    const invite_list_alice: InviteInfo[] = await getPendingInvites(alice.cells[0])
    console.log(invite_list_alice)
    assert.deepEqual(invite_list_alice[0].invitees_who_accepted[0],bob.agentPubKey)
  });
});

test('3. create and update Invite', async () => {
  await runScenario(async scenario => {
   
    // setup signal handlers
    let processSignal_alice: AppSignalCb | undefined;
    let signalReceived_alice = new Promise<AppSignal>((resolve) => {
      processSignal_alice = (signal) => {
        console.log("signal found for Alice:",signal)
        resolve(signal);
      };
    });

    //todo change this to rxjs observable subscription to playback multiple signals

    let processSignal_bob: AppSignalCb | undefined;
    let signalReceived_bob = new Promise<AppSignal>((resolve) => {
      processSignal_bob = (signal) => {
        console.log("signal found for bob:",signal)
        resolve(signal);
      };
    });

    // Construct proper paths for your app.
    // This assumes app bundle created by the `hc app pack` command.
    const testAppPath = process.cwd() + path_to_happ;

    // Set up the app to be installed 
    const appSource_alice = { appBundleSource: { path: testAppPath }}
    const appSource_bob = { appBundleSource: { path: testAppPath }}

    // Add 2 players with the test app to the Scenario. The returned players
    // can be destructured.
    const [alice] = await scenario.addPlayersWithApps([appSource_alice]);
    const [bob] = await scenario.addPlayersWithApps([appSource_bob]);

    //setup signal handling
    alice.conductor.appWs().on("signal",processSignal_alice)
    bob.conductor.appWs().on("signal",processSignal_bob)

    // Shortcut peer discovery through gossip and register all agents in every
    // conductor of the scenario.
    await scenario.shareAllAgents();

    console.log("\n************************* START TEST ****************************\n")

    console.log("\nAlice creates an Invite to Bob\n")
    const invite_detail: InviteInfo = await sendInvitations(alice.cells[0],getSampleInviteInput([bob.agentPubKey]));
    assert.ok(invite_detail);

    await pause(1200);
    let bob_signal = await signalReceived_bob

    console.log("Bob sees he has been signalled a new Invite:\n",bob_signal.payload['data'])
    assert.equal(bob_signal.payload['type'], 'InvitationReceived')

    console.log("\nBob accepts the invitation\n")
    const accept: boolean = await acceptInvite(bob.cells[0],bob_signal.payload['data'].invitation_original_hash)
    console.log(accept)
    
    await pause(1200);
    let alice_signal = await signalReceived_alice

    assert.equal(alice_signal.payload['type'],'InvitationAccepted',"message should be of type accepted")
    let invitees = alice_signal.payload['data'].invitees_who_accepted
    assert.deepEqual(invitees[0], bob.agentPubKey, "Bob was not found in the accepted invitees")

    console.log("Alice sees Bob has accepted the invite via a signal and decides to update the invite location and start_time\n")
    let inviteUpdate = getSampleInviteInputUpdate([bob.agentPubKey], alice_signal.payload['data'].invitation_original_hash)
    const invite_list_alice: InviteInfo = await updateInvitation(alice.cells[0],inviteUpdate)
    console.log(invite_list_alice)
    
    await pause(1200);
    let bob_signal2 = await signalReceived_bob

    //console.log("Bob sees he has been signalled an updated Invite:\n",bob_signal2.payload['data'])
    //console.log(bob_signal2.payload['type'])
    //assert.equal(bob_signal2.payload['type'], 'InvitationUpdated')
  });
});


test('4. create and reject Invite', async () => {
  await runScenario(async scenario => {
   
    // setup signal handlers
    let processSignal_alice: AppSignalCb | undefined;
    let signalReceived_alice = new Promise<AppSignal>((resolve) => {
      processSignal_alice = (signal) => {
        console.log("signal found for Alice:",signal)
        resolve(signal);
      };
    });


    let processSignal_bob: AppSignalCb | undefined;
    let signalReceived_bob = new Promise<AppSignal>((resolve) => {
      processSignal_bob = (signal) => {
        console.log("signal found for bob:",signal)
        resolve(signal);
      };
    });
    // Construct proper paths for your app.
    // This assumes app bundle created by the `hc app pack` command.
    const testAppPath = process.cwd() + path_to_happ;

    // Set up the app to be installed 
    const appSource_alice = { appBundleSource: { path: testAppPath }}
    const appSource_bob = { appBundleSource: { path: testAppPath }}

    // Add 2 players with the test app to the Scenario. The returned players
    // can be destructured.
    const [alice] = await scenario.addPlayersWithApps([appSource_alice]);
    const [bob] = await scenario.addPlayersWithApps([appSource_bob]);

    //setup signal handling
    alice.conductor.appWs().on("signal",processSignal_alice)
    bob.conductor.appWs().on("signal",processSignal_bob)

    // Shortcut peer discovery through gossip and register all agents in every
    // conductor of the scenario.
    await scenario.shareAllAgents();

    console.log("\n************************* START TEST ****************************\n")

    console.log("\nAlice creates an Invite to Bob\n")
    const invite_detail: InviteInfo = await sendInvitations(alice.cells[0],getSampleInviteInput([bob.agentPubKey]));
    assert.ok(invite_detail);

    await pause(1200);
    let bob_signal = await signalReceived_bob

    console.log("Bob sees he has been signalled a new Invite:\n",bob_signal.payload['data'])
    assert.equal(bob_signal.payload['type'], 'InvitationReceived')

    console.log("\nBob rejects the invitation\n")
    const reject: boolean = await rejectInvite(bob.cells[0],bob_signal.payload['data'].invitation_original_hash)
    console.log(reject)
    
    await pause(1200);
    let alice_signal = await signalReceived_alice

    assert.equal(alice_signal.payload['type'],'InvitationRejected',"message should be of type rejected")
    let invitees = alice_signal.payload['data'].invitees_who_rejected
    assert.deepEqual(invitees[0], bob.agentPubKey, "Bob was not found in the rejected invitees")

    console.log("Alice sees Bob has rejected the invite via a signal and checks the invite status\n")
    const invite_list_alice: InviteInfo[] = await getPendingInvites(alice.cells[0])
    console.log(invite_list_alice)
    assert.deepEqual(invite_list_alice[0].invitees_who_rejected[0],bob.agentPubKey)
  });
});


test('5. create, reject and clear Invite', async () => {
  await runScenario(async scenario => {
        
    // setup signal receiver
    let processSignal_bob: AppSignalCb | undefined;
    let signalReceived_bob = new Promise<AppSignal>((resolve) => {
      processSignal_bob = (signal) => {
        console.log("signal found for bob:",signal)
        resolve(signal);
      };
    });

    // Construct proper paths for your app.
    // This assumes app bundle created by the `hc app pack` command.
    const testAppPath = process.cwd() + path_to_happ;

    // Set up the app to be installed 
    const appSource_alice = { appBundleSource: { path: testAppPath }}
    const appSource_bob = { appBundleSource: { path: testAppPath }}

    // Add 2 players with the test app to the Scenario. The returned players
    // can be destructured.
    const [alice] = await scenario.addPlayersWithApps([appSource_alice]);
    const [bob] = await scenario.addPlayersWithApps([appSource_bob]);

    //setup signal handling
    bob.conductor.appWs().on("signal",processSignal_bob)

    // Shortcut peer discovery through gossip and register all agents in every
    // conductor of the scenario.
    await scenario.shareAllAgents();

    console.log("\n************************* START TEST ****************************\n")

    console.log("\nAlice creates an Invite to Bob\n")
    const invite_detail: InviteInfo = await sendInvitations(alice.cells[0],getSampleInviteInput([bob.agentPubKey]));
    assert.ok(invite_detail);
    
    await pause(1200);
    let bob_signal = await signalReceived_bob

    console.log("Bob sees he has been signalled a new Invite:\n",bob_signal.payload['data'])
    assert.equal(bob_signal.payload['type'], 'InvitationReceived')

    console.log("\nBob rejects the invitation\n")
    const reject: boolean = await rejectInvite(bob.cells[0],bob_signal.payload['data'].invitation_original_hash)
    console.log(reject)
    
    console.log("Bob clears the invitation")
    const result: boolean = await clearInvite(bob.cells[0], bob_signal.payload['data'].invitation_original_hash)
    console.log(result)

    console.log("Bob checks that he has deleted the invitation from his list by seeing he has no invitations")
    const invite_list_bob  = await getPendingInvites(bob.cells[0])
    console.log(invite_list_bob)
    assert.isNull(invite_list_bob)
  });
});
