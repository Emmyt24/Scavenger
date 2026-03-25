#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, vec};
use stellar_scavngr_contract::{ScavengerContract, ScavengerContractClient};

fn setup(env: &Env) -> (ScavengerContractClient<'_>, Address) {
    let client = ScavengerContractClient::new(env, &env.register_contract(None, ScavengerContract));
    let admin = Address::generate(env);
    client.initialize_admin(&admin);
    (client, admin)
}

#[test]
#[should_panic(expected = "Unauthorized: caller is not admin")]
fn test_transfer_admin_non_admin_cannot_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);
    let non_admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let new_admins = vec![&env, new_admin];
    client.transfer_admin(&non_admin, &new_admins);
}

#[test]
fn test_transfer_admin_new_admin_can_call_admin_functions() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let new_admin = Address::generate(&env);
    let new_admins = vec![&env, new_admin.clone()];
    client.transfer_admin(&admin, &new_admins);
    assert_eq!(client.get_admin(), new_admin);
}

#[test]
#[should_panic(expected = "Unauthorized: caller is not admin")]
fn test_transfer_admin_old_admin_loses_privileges() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let new_admin = Address::generate(&env);
    let new_admins = vec![&env, new_admin];
    client.transfer_admin(&admin, &new_admins);
    // old admin should no longer have privileges
    let another_admin = Address::generate(&env);
    let another_admins = vec![&env, another_admin];
    client.transfer_admin(&admin, &another_admins);
}
