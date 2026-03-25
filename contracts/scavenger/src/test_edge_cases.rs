#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};
use soroban_sdk::token::StellarAssetClient;

use crate::{ScavengerContract, ScavengerContractClient};
use crate::types::{Role, WasteType};

fn setup(env: &Env) -> (ScavengerContractClient<'_>, Address, Address) {
    let contract_id = env.register(ScavengerContract, ());
    let client = ScavengerContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract(admin.clone());
    let charity = Address::generate(env);
    env.mock_all_auths();
    // collector_pct=10, owner_pct=20 → recycler gets remaining 70%
    client.initialize(&admin, &token, &charity, &10, &20);
    (client, admin, token)
}

fn name(env: &Env, s: &str) -> String {
    String::from_str(env, s)
}

// ── Edge case 1: zero-weight waste with an active incentive ──────────────────
//
// weight_kg = 0 / 1000 = 0, so total_reward = reward_points * 0 = 0.
// The budget check (0 <= remaining_budget) passes, distribute_rewards returns 0,
// and the incentive budget is unchanged (subtracting 0).
// This documents that zero-weight submissions silently produce no reward.
#[test]
fn test_zero_weight_waste_yields_zero_reward() {
    let env = Env::default();
    let (client, admin, token) = setup(&env);

    let manufacturer = Address::generate(&env);
    let recycler = Address::generate(&env);

    client.register_participant(&manufacturer, &Role::Manufacturer, &name(&env, "M"), &0, &0);
    client.register_participant(&recycler, &Role::Recycler, &name(&env, "R"), &0, &0);

    StellarAssetClient::new(&env, &token).mint(&manufacturer, &1_000_000);

    // Budget of 500; reward_points = 100 pts/kg
    let incentive = client.create_incentive(&manufacturer, &WasteType::Plastic, &100, &500);

    // Submit waste with weight < 1000 g → weight_kg rounds to 0
    let material = client.submit_material(&recycler, &WasteType::Plastic, &999);
    client.confirm_waste(&material.id, &recycler);

    let total = client.distribute_rewards(&material.id, &incentive.id, &manufacturer);

    // Zero reward distributed
    assert_eq!(total, 0);

    // Budget must be untouched — incentive stays active
    let updated = client.get_incentive_by_id(&incentive.id).unwrap();
    assert_eq!(updated.remaining_budget, 500);
    assert!(updated.active);
}

// ── Edge case 2: single-participant chain (submitter == current owner, no collectors) ──
//
// When the recycler submits and never transfers, there are no collector entries
// in the transfer history. The owner share goes to the submitter, and the
// recycler (current_owner == submitter) receives the remainder.
// Both amounts land on the same address, so total_earned = total_reward.
#[test]
fn test_single_participant_chain_all_reward_to_recycler() {
    let env = Env::default();
    let (client, _admin, token) = setup(&env);

    let manufacturer = Address::generate(&env);
    let recycler = Address::generate(&env);

    client.register_participant(&manufacturer, &Role::Manufacturer, &name(&env, "M"), &0, &0);
    client.register_participant(&recycler, &Role::Recycler, &name(&env, "R"), &0, &0);

    StellarAssetClient::new(&env, &token).mint(&manufacturer, &1_000_000);

    // 10 pts/kg, budget 10_000
    let incentive = client.create_incentive(&manufacturer, &WasteType::Plastic, &10, &10_000);

    // 5 kg, no transfer — recycler stays current_owner
    let material = client.submit_material(&recycler, &WasteType::Plastic, &5_000);
    client.confirm_waste(&material.id, &recycler);

    // total_reward = 10 * 5 = 50
    let total = client.distribute_rewards(&material.id, &incentive.id, &manufacturer);
    assert_eq!(total, 50);

    // submitter (owner_pct=20%) gets 10, recycler (remainder=80%) gets 40
    // both are the same address → total_earned = 50
    let stats = client.get_participant_stats(&recycler);
    assert_eq!(stats.total_earned, 50);
}

// ── Edge case 3: incentive with exact budget match ───────────────────────────
//
// When total_reward == remaining_budget exactly, the budget hits 0 after
// distribution and the contract must automatically deactivate the incentive.
// Any subsequent distribute_rewards call must fail with "Incentive not active".
#[test]
fn test_exact_budget_match_deactivates_incentive() {
    let env = Env::default();
    let (client, _admin, token) = setup(&env);

    let manufacturer = Address::generate(&env);
    let recycler = Address::generate(&env);

    client.register_participant(&manufacturer, &Role::Manufacturer, &name(&env, "M"), &0, &0);
    client.register_participant(&recycler, &Role::Recycler, &name(&env, "R"), &0, &0);

    StellarAssetClient::new(&env, &token).mint(&manufacturer, &1_000_000);

    // 10 pts/kg, budget exactly 50 → one 5 kg submission exhausts it
    let incentive = client.create_incentive(&manufacturer, &WasteType::Plastic, &10, &50);

    let material = client.submit_material(&recycler, &WasteType::Plastic, &5_000);
    client.confirm_waste(&material.id, &recycler);

    let total = client.distribute_rewards(&material.id, &incentive.id, &manufacturer);
    assert_eq!(total, 50);

    // Budget is now 0 → incentive must be deactivated automatically
    let updated = client.get_incentive_by_id(&incentive.id).unwrap();
    assert_eq!(updated.remaining_budget, 0);
    assert!(!updated.active, "Incentive should be deactivated when budget hits zero");
}

#[test]
#[should_panic(expected = "Incentive not active")]
fn test_exact_budget_match_blocks_further_distribution() {
    let env = Env::default();
    let (client, _admin, token) = setup(&env);

    let manufacturer = Address::generate(&env);
    let recycler = Address::generate(&env);

    client.register_participant(&manufacturer, &Role::Manufacturer, &name(&env, "M"), &0, &0);
    client.register_participant(&recycler, &Role::Recycler, &name(&env, "R"), &0, &0);

    StellarAssetClient::new(&env, &token).mint(&manufacturer, &1_000_000);

    let incentive = client.create_incentive(&manufacturer, &WasteType::Plastic, &10, &50);

    let m1 = client.submit_material(&recycler, &WasteType::Plastic, &5_000);
    client.confirm_waste(&m1.id, &recycler);
    client.distribute_rewards(&m1.id, &incentive.id, &manufacturer);

    // Second submission — incentive is now inactive, must panic
    let m2 = client.submit_material(&recycler, &WasteType::Plastic, &5_000);
    client.confirm_waste(&m2.id, &recycler);
    client.distribute_rewards(&m2.id, &incentive.id, &manufacturer);
}

// ── Edge case 4: transfer to self ────────────────────────────────────────────
//
// Allowing self-transfer would let a participant inject themselves into the
// collector history multiple times to inflate their reward share.
// The contract must reject it.
#[test]
#[should_panic(expected = "Cannot transfer waste to self")]
fn test_transfer_to_self_is_rejected() {
    let env = Env::default();
    let (client, _admin, _token) = setup(&env);

    let recycler = Address::generate(&env);
    client.register_participant(&recycler, &Role::Recycler, &name(&env, "R"), &0, &0);

    let material = client.submit_material(&recycler, &WasteType::Plastic, &5_000);

    // Transferring to yourself must be rejected
    client.transfer_waste(&material.id, &recycler, &recycler);
}
