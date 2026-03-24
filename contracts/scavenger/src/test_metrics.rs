#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};
use crate::{ScavengerContract, ScavengerContractClient, Role, WasteType};

fn create_test_contract(env: &Env) -> (ScavengerContractClient<'_>, Address, Address, Address) {
    let contract_id = env.register(ScavengerContract, ());
    let client = ScavengerContractClient::new(env, &contract_id);
    
    let admin = Address::generate(env);
    let token_address = env.register_stellar_asset_contract(admin.clone());
    let charity_address = Address::generate(env);
    
    (client, admin, token_address, charity_address)
}

#[test]
fn test_get_metrics() {
    let env = Env::default();
    let (client, admin, token_address, charity_address) = create_test_contract(&env);
    
    env.mock_all_auths();
    client.initialize(&admin, &token_address, &charity_address, &30, &20);
    
    // Check initial metrics
    let metrics = client.get_metrics();
    assert_eq!(metrics.total_wastes_count, 0);
    assert_eq!(metrics.total_tokens_earned, 0);
    
    // Register participant and submit material to increment count
    let recycler = Address::generate(&env);
    let name = String::from_str(&env, "Recycler");
    client.register_participant(&recycler, &Role::Recycler, &name, &100, &200);
    
    client.submit_material(&recycler, &WasteType::PetPlastic, &5000);
    
    let metrics2 = client.get_metrics();
    assert_eq!(metrics2.total_wastes_count, 1);
    
    // Submit another
    client.submit_material(&recycler, &WasteType::Glass, &2000);
    let metrics3 = client.get_metrics();
    assert_eq!(metrics3.total_wastes_count, 2);
}

#[test]
fn test_get_supply_chain_stats() {
    let env = Env::default();
    let (client, admin, token_address, charity_address) = create_test_contract(&env);
    
    env.mock_all_auths();
    client.initialize(&admin, &token_address, &charity_address, &30, &20);
    
    // Initial stats
    let (count, weight, earned) = client.get_supply_chain_stats();
    assert_eq!(count, 0);
    assert_eq!(weight, 0);
    assert_eq!(earned, 0);
    
    // Submit some waste
    let recycler = Address::generate(&env);
    let name = String::from_str(&env, "Recycler");
    client.register_participant(&recycler, &Role::Recycler, &name, &100, &200);
    
    client.submit_material(&recycler, &WasteType::Metal, &3000);
    
    let (count2, weight2, earned2) = client.get_supply_chain_stats();
    assert_eq!(count2, 1);
    assert_eq!(weight2, 3000);
    assert_eq!(earned2, 0);
}
