use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

/// Helper function to create a test environment
fn create_test_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

#[test]
fn test_governance_initialization() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(&admin);

    // Verify initialization succeeded by checking a simple getter
    let min_cr = client.get_min_collateral_ratio();
    assert!(min_cr.is_ok());
    
    // Verify default min collateral ratio is set
    assert_eq!(min_cr.unwrap(), 11_000); // 110% in basis points
}
