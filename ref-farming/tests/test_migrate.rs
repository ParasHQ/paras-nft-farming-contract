use std::convert::TryFrom;

use near_sdk::json_types::ValidAccountId;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto};
use near_sdk::serde_json::json;
use near_sdk::json_types::{U128};
use ref_farming::{HRSimpleFarmTerms};
use near_sdk::serde_json::Value;


use ref_farming::ContractContract as Farming;
use near_sdk::{AccountId, Duration};
use near_sdk_sim::account::Account;
use crate::common::utils::*;
use crate::common::init::{deploy_farming, deploy_nft_contract};
use crate::common::views::*;
use crate::common::actions::*;
use std::collections::HashMap;

mod common;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    PREV_FARMING_WASM_BYTES => "../res/ref_farming_101.wasm",
    FARMING_WASM_BYTES => "../res/ref_farming_release.wasm",
}


#[test]
fn test_upgrade() {
    let root = init_simulator(None);
    let test_user = root.create_user("test".to_string(), to_yocto("100"));
    let farming = deploy!(
        contract: Farming,
        contract_id: "farming".to_string(),
        bytes: &PREV_FARMING_WASM_BYTES,
        signer_account: root,
        init_method: new(ValidAccountId::try_from("farming").unwrap(), None, None, None)
    );

    // Failed upgrade with no permissions.
    let result = test_user
        .call(
            farming.user_account.account_id.clone(),
            "migrate",
            &json!({}).to_string().into_bytes(),
            near_sdk_sim::DEFAULT_GAS,
            0,
        )
        .status();
    assert!(format!("{:?}", result).contains("Method migrate is private"));

    // ==== STAKE BEFORE UPGRADE ====

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    let farmer2 = root.create_user("farmer2".to_string(), to_yocto("100"));

    let (_, token1, _) = prepair_pool_and_liquidity(
        &root, &owner, farming_id(), vec![&farmer1, &farmer2]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmers.");
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(farmer2, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmers registered.");

    // create farm
    println!("----->> Create farm.");
    let farm_id = "dai#0".to_string();
    let out_come = call!(
        farming.user_account,
        farming.create_simple_farm(HRSimpleFarmTerms{
            seed_id: token1.account_id(),
            reward_token: token1.valid_account_id(),
            start_at: 0,
            reward_per_session: to_yocto("1").into(),
            session_interval: 60,
        }, None, None, None),
        deposit = to_yocto("1")
    );
    out_come.assert_success();
    assert_eq!(Value::String(farm_id.clone()), out_come.unwrap_json_value());

    // register to token1
    println!("Register to token1 farming_id");
    call!(
        root,
        token1.storage_deposit(Some(to_va(farming_id())), None),
        deposit = to_yocto("1")
    );

    mint_token(&token1, &root, to_yocto("10"));

    println!("----->> Deposit reward to turn farm Running.");
    call!(
        root,
        token1.ft_transfer_call(to_va(farming_id()), U128(to_yocto("10")), None, farm_id.clone()),
        deposit = 1
    ).assert_success();

    show_farminfo(&farming, farm_id.clone(), true);

    // farmer1 staking lpt
    println!("----->> Farmer1 staking lpt.");
    let out_come = call!(
        farmer1,
        token1.ft_transfer_call(to_va(farming_id()), U128(to_yocto("1")), None, "".to_string()),
        deposit = 1
    );
    out_come.assert_success();

    // ==== AFTER STAKE ====

    farming.user_account
        .create_transaction(farming.user_account.account_id.clone())
        .deploy_contract(FARMING_WASM_BYTES.to_vec())
        .submit()
        .assert_success();

    let one_day = 24*60*60*10u64.pow(9);
    let result = farming.user_account.call(
         farming.user_account.account_id.clone(),
         "migrate",
         &json!({"dao_contract_id": "dao", "dao_utility_token": "token", "unstake_period": one_day }).to_string().into_bytes(),
         near_sdk_sim::DEFAULT_GAS,
         0,
     ).assert_success();

    let (dao_contract_id, dao_utility_token, unstake_period): (AccountId, AccountId, Duration) = farming.user_account.view(farming.user_account.account_id.clone(), "get_dao_info", &[]).unwrap_json();

    assert_eq!(dao_contract_id, "dao");
    assert_eq!(dao_utility_token, "token");
    assert_eq!(unstake_period, one_day);

    // pub fn list_user_seeds(&self, account_id: ValidAccountId) -> HashMap<SeedId, U128> {

    let user_seeds: HashMap<String, U128> = farming.user_account.view(
        farming.user_account.account_id.clone(),
        "list_rewards",
        &json!({"account_id": farmer1.account_id}).to_string().as_bytes()
    ).unwrap_json();

    println!("{:?}", user_seeds);

    let delegation_info: Value = farming.user_account.view(
        farming.user_account.account_id.clone(),
        "get_user_delegations",
        &json!({"account_id": farmer1.account_id}).to_string().as_bytes()
    ).unwrap_json();
    println!("{:?}", delegation_info);
}
