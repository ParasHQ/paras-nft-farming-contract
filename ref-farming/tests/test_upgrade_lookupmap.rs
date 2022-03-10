use std::convert::TryFrom;

use near_sdk::json_types::ValidAccountId;
use near_sdk_sim::{deploy, init_simulator, to_yocto, call, view, DEFAULT_GAS};

use ref_farming::ContractContract as Farming;
use std::collections::HashMap;
use near_sdk::json_types::{U128};

use ref_farming::{HRSimpleFarmTerms};
use crate::common::utils::*;
use crate::common::init::{deploy_farming, deploy_nft_contract};
use crate::common::views::*;
use crate::common::actions::*;
mod common;
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    PREV_FARMING_WASM_BYTES => "../res/ref_farming_101.wasm",
    FARMING_WASM_BYTES => "../res/ref_farming_release.wasm",
}


#[test]
fn test_upgrade_lookupmap() {
    let root = init_simulator(None);
    let test_user = root.create_user("test".to_string(), to_yocto("100"));

    println!("----->> Deploy farming");
    let farming = deploy!(
        contract: Farming,
        contract_id: "farming".to_string(),
        bytes: &PREV_FARMING_WASM_BYTES,
        signer_account: root,
        init_method: new(ValidAccountId::try_from(root.account_id.clone()).unwrap())
    );

    // create farm
    println!("----->> Create farm.");
    let farm_id = "random.near".to_string();
    let mut nft_balance: HashMap<String, U128> = HashMap::new();

    for i in 0..2000 {
        nft_balance.insert(format!("{}@{}", "nft-contract", i.to_string()), U128(3000000000000000000000000));
    }

    let out_come = call!(
        root,
        farming.create_simple_farm(
            HRSimpleFarmTerms{
                seed_id: "random.near".to_string(),
                reward_token: (ValidAccountId::try_from("token.near").unwrap()),
                start_at: 0,
                reward_per_session: to_yocto("1").into(),
                session_interval: 60,
            },
            None,
            Some(nft_balance),
            None
        ),
        to_yocto("1"),
        DEFAULT_GAS
    );

    out_come.assert_success();
    println!("<<----- Farm {} created at #{}, ts:{}.",
             farm_id,
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    let seed_info = show_seedsinfo(&farming, false);
    println!("seed_info {:?}", seed_info);

    root.call(
        farming.user_account.account_id.clone(),
        "upgrade",
        &FARMING_WASM_BYTES,
        near_sdk_sim::DEFAULT_GAS,
        0,
    )
        .assert_success();

    let out_come = call!(
        root,
        farming.force_upgrade_seed(
            "random.near".to_string()
        ),
        0,
        DEFAULT_GAS
    );

    out_come.assert_success();
    let seed_info = show_seedsinfo(&farming, false);

    let nft_balance_1: Option<U128> = view!(
        farming.get_nft_balance_equivalent("random.near".to_string(), "nft-contract@1".to_string())
    ).unwrap_json();

    println!("nft-contract@1: {:#?}", nft_balance_1);

    for i in 0..2000/100 {
        let mut nft_balance: HashMap<String, U128> = HashMap::new();
        for j in i..i+100 {
            nft_balance.insert(format!("{}@{}", "nft-contract", j.to_string()), U128(3000000000000000000000000));
        }

        println!("{}", i);
        let out_come = call!(
            root,
            farming.upgrade_lookup_map(
                "random.near".to_string(),
                nft_balance
            ),
            0,
            DEFAULT_GAS
        );
        out_come.assert_success();
    }


    let nft_balance_1: Option<U128> = view!(
        farming.get_nft_balance_equivalent("random.near".to_string(), "nft-contract@1".to_string())
    ).unwrap_json();

    println!("nft-contract@1: {:#?}", nft_balance_1);
}