#![cfg(test)]

use crate::{HelloContract, HelloContractClient};
use soroban_sdk::{testutils::{Address as _}, Address, Env};
use crate::risk_management::RiskManagementError;

fn setup_test() -> (Env, HelloContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.initialize(&admin);
    
    (env, client, admin)
}

#[test]
fn test_initialize_sets_default_params() {
    let (_env, client, _admin) = setup_test();
    
    assert_eq!(client.get_min_collateral_ratio(), 11_000); // 110%
    assert_eq!(client.get_liquidation_threshold(), 10_500); // 105%
    assert_eq!(client.get_close_factor(), 5_000); // 50%
    assert_eq!(client.get_liquidation_incentive(), 1_000); // 10%
}

#[test]
fn test_set_risk_params_success() {
    let (_env, client, admin) = setup_test();
    
    // Change parameters within allowed limit (e.g. 1% or less)
    // Default 11_000, 1% change is 110. Let's use 11_100.
    client.set_risk_params(&admin, &Some(11_100), &Some(10_600), &Some(5_100), &Some(1_050));
    
    assert_eq!(client.get_min_collateral_ratio(), 11_100);
    assert_eq!(client.get_liquidation_threshold(), 10_600);
    assert_eq!(client.get_close_factor(), 5_100);
    assert_eq!(client.get_liquidation_incentive(), 1_050);
}

#[test]
fn test_set_risk_params_unauthorized() {
    let (env, client, _admin) = setup_test();
    let not_admin = Address::generate(&env);
    
    let result = client.try_set_risk_params(&not_admin, &Some(11_100), &None, &None, &None);
    match result {
        Err(Ok(RiskManagementError::Unauthorized)) => {},
        _ => panic!("Expected Unauthorized error, got {:?}", result),
    }
}

#[test]
fn test_set_risk_params_exceeds_change_limit() {
    let (_env, client, admin) = setup_test();
    
    // Default is 11_000, 10% change max is 1_100, so new value <= 12_100
    // Try setting to 12_200, should fail with ParameterChangeTooLarge
    let result = client.try_set_risk_params(&admin, &Some(12_200), &None, &None, &None);
    match result {
        Err(Ok(RiskManagementError::ParameterChangeTooLarge)) => {},
        _ => panic!("Expected ParameterChangeTooLarge error, got {:?}", result),
    }
}

#[test]
fn test_set_risk_params_invalid_collateral_ratio() {
    let (_env, client, admin) = setup_test();
    
    // Current min_collateral_ratio is 11_000
    // Try to set liquidation_threshold to 11_500, which is over min_cr
    // Fail with InvalidCollateralRatio
    // Note: 11_500 is within 10% change limit from 10_500 (1050 max change)
    let result = client.try_set_risk_params(&admin, &None, &Some(11_500), &None, &None);
    match result {
        Err(Ok(RiskManagementError::InvalidCollateralRatio)) => {},
        _ => panic!("Expected InvalidCollateralRatio error, got {:?}", result),
    }
}

#[test]
fn test_get_max_liquidatable_amount() {
    let (_env, client, _admin) = setup_test();
    let debt = 1_000_000;
    // default close factor is 5_000 (50%)
    assert_eq!(client.get_max_liquidatable_amount(&debt), 500_000);
}

#[test]
fn test_get_liquidation_incentive_amount() {
    let (_env, client, _admin) = setup_test();
    let liquidated_amount = 500_000;
    // default incentive is 1_000 (10%)
    assert_eq!(client.get_liquidation_incentive_amount(&liquidated_amount), 50_000);
}
