use near_sdk_sim::{call, init_simulator, to_yocto, view, DEFAULT_GAS};
use near_sdk::json_types::{U128};
use near_sdk::serde_json::Value;

use ref_farming::HRSimpleFarmTerms;

use crate::common::utils::*;
use crate::common::init::{deploy_farming, deploy_nft_contract};
use crate::common::views::*;
use crate::common::actions::*;
use near_sdk::serde_json::json;
use std::collections::HashMap;

mod common;

#[test]
fn single_paras_farm() {
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    let farmer2 = root.create_user("farmer2".to_string(), to_yocto("100"));
    println!("<<----- owner and 2 farmers prepared.");

    // println!("----->> Prepare ref-exchange and swap pool.");
    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1, &farmer2]);
    // println!("<<----- The pool prepaired.");

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmers.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(farmer2, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmers registered.");

    // create farm
    println!("----->> Create farm.");
    let farm_id = "dai#0".to_string();
    let out_come = call!(
        owner,
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
    println!("<<----- Farm {} created at #{}, ts:{}.",
             farm_id,
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

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

    println!("<<----- Farm {} deposit reward at #{}, ts:{}.",
             farm_id,
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    // farmer1 staking lpt
    println!("----->> Farmer1 staking lpt.");
    let out_come = call!(
        farmer1,
        token1.ft_transfer_call(to_va(farming_id()), U128(to_yocto("1")), None, "".to_string()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 staked liquidity at #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 0, 0, 0, 0, 0);
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, 0_u128);

    println!("----->> move to 60 secs later.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("<<----- Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 1, 0, 0, to_yocto("1"), 0);
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1"));

    // farmer2 staking lpt
    println!("----->> Farmer2 staking lpt.");
    let out_come = call!(
        farmer2,
        token1.ft_transfer_call(to_va(farming_id()), U128(to_yocto("1")), None, "".to_string()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer2 staked liquidity at #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 1, 1, 0, to_yocto("1"), 0);
    let user_seeds = show_userseeds(&farming, farmer2.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, 0_u128);

    println!("----->> move to 60 secs later.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("<<----- Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 2, 1, 0, to_yocto("2"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1.5"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0.5"));

    println!("----->> move to 60 secs later and farmer1 claim reward by farm_id.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("        Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 3, 1, 0, to_yocto("3"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("2"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1"));
    let out_come = call!(
        farmer1,
        farming.claim_reward_by_farm(farm_id.clone()),
        deposit = 0
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 3, 3, to_yocto("2"), to_yocto("1"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, 0_u128);
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1"));
    let reward = show_reward(&farming, farmer1.account_id(), token1.account_id(), false);
    assert_eq!(reward.0, to_yocto("2"));
    println!("<<----- Farmer1 claimed reward by farmid, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    println!("----->> move to 60 secs later and farmer2 claim reward by seed_id.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("        Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 4, 3, to_yocto("2"), to_yocto("2"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0.5"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1.5"));
    let out_come = call!(
        farmer2,
        farming.claim_reward_by_seed(farm_info.seed_id.clone()),
        deposit = 0
    );
    out_come.assert_success();
    // println!("{:#?}", out_come.promise_results());
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 4, 4, to_yocto("3.5"), to_yocto("0.5"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0.5"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));
    let reward = show_reward(&farming, farmer2.account_id(), token1.account_id(), false);
    assert_eq!(reward.0, to_yocto("1.5"));
    println!("<<----- Farmer2 claimed reward by seedid, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    println!("----->> move to 60 secs later and farmer1 unstake half lpt.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("        Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 5, 4, to_yocto("3.5"), to_yocto("1.5"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0.5"));
    let out_come = call!(
        farmer1,
        farming.withdraw_seed(farm_info.seed_id.clone(), to_yocto("0.4").into()),
        deposit = 1
    );
    out_come.assert_success();
    // println!("{:#?}", out_come.promise_results());
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 5, 5, to_yocto("4.5"), to_yocto("0.5"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0.5"));
    let reward = show_reward(&farming, farmer1.account_id(), token1.account_id(), false);
    assert_eq!(reward.0, to_yocto("0"));
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&farm_info.seed_id.clone()).unwrap().0, to_yocto("0.6"));
    println!("<<----- Farmer1 unstake half lpt, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    println!("----->> move to 60 secs later and farmer2 unstake all his lpt.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("        Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 6, 5, to_yocto("4.5"), to_yocto("1.5"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0.375"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1.125"));
    let out_come = call!(
        farmer2,
        farming.withdraw_seed(farm_info.seed_id.clone(), to_yocto("1").into()),
        deposit = 1
    );
    out_come.assert_success();
    // println!("{:#?}", out_come.promise_results());
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 6, 6, to_yocto("5.625"), to_yocto("0.375"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0.375"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));
    let reward = show_reward(&farming, farmer2.account_id(), token1.account_id(), false);
    assert_eq!(reward.0, to_yocto("0"));
    let user_seeds = show_userseeds(&farming, farmer2.account_id(), false);
    assert!(user_seeds.get(&farm_info.seed_id.clone()).is_none());
    println!("<<----- Farmer2 unstake all his lpt, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    println!("----->> move to 60 secs later and farmer1 unstake the other half lpt.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("        Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 7, 6, to_yocto("5.625"), to_yocto("1.375"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1.374999999999999999999999"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));
    let out_come = call!(
        farmer1,
        farming.withdraw_seed(farm_info.seed_id.clone(), to_yocto("0.6").into()),
        deposit = 1
    );
    out_come.assert_success();
    // println!("{:#?}", out_come.promise_results());
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 7, 7, to_yocto("6.999999999999999999999999"), 1, 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));
    let reward = show_reward(&farming, farmer1.account_id(), token1.account_id(), false);
    assert_eq!(reward.0, to_yocto("0"));
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert!(user_seeds.get(&farm_info.seed_id.clone()).is_none());
    println!("<<----- Farmer1 unstake the other half lpt, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    println!("----->> move to 60 secs later and farmer1 restake lpt.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("        Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 8, 7, to_yocto("6.999999999999999999999999"), 1 + to_yocto("1"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));
    let out_come = call!(
        farmer1,
        token1.ft_transfer_call(to_va(farming_id()), U128(to_yocto("1")), None, "".to_string()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 staked liquidity at #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 8, 8, to_yocto("8"), to_yocto("0"), to_yocto("1") + 1);
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));
    let out_come = call!(
        owner,
        farming.force_clean_farm(farm_id.clone()),
        deposit = 0
    );
    out_come.assert_success();
    assert_eq!(Value::Bool(false), out_come.unwrap_json_value());

    println!("----->> move to 40 secs later and farmer2 restake lpt.");
    assert!(root.borrow_runtime_mut().produce_blocks(40).is_ok());
    println!("        Chain goes 40 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 9, 8, to_yocto("8"), to_yocto("1"), to_yocto("1") + 1);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));
    let out_come = call!(
        farmer2,
        token1.ft_transfer_call(to_va(farming_id()), U128(to_yocto("1")), None, "".to_string()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer2 staked liquidity at #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 9, 9, to_yocto("8"), to_yocto("1"), to_yocto("1") + 1);
    let user_seeds = show_userseeds(&farming, farmer2.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0"));

    println!("----->> move to 60 secs later.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("<<----- Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Ended".to_string(), to_yocto("10"), 10, 9, to_yocto("8"), to_yocto("2"), to_yocto("1") + 1);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1.5"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0.5"));

    println!("----->> move to 60 secs later, and force remove farm");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("<<----- Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Ended".to_string(), to_yocto("10"), 10, 9, to_yocto("8"), to_yocto("2"), to_yocto("1") + 1);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1.5"));
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("0.5"));
    let out_come = call!(
        owner,
        farming.force_clean_farm(farm_id.clone()),
        deposit = 0
    );
    out_come.assert_success();
    assert_eq!(Value::Bool(true), out_come.unwrap_json_value());
    assert_eq!(view!(farming.get_number_of_farms()).unwrap_json::<u64>(), 0);
    assert_eq!(view!(farming.get_number_of_outdated_farms()).unwrap_json::<u64>(), 1);
    let farm_info = show_outdated_farminfo(&farming, farm_id.clone(), true);
    assert_farming(&farm_info, "Cleared".to_string(), to_yocto("10"), 10, 10, to_yocto("10"), to_yocto("0"), to_yocto("3") + 1);
}

#[test]
fn test_farm_with_nft_mappings() {
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    let farmer2 = root.create_user("farmer2".to_string(), to_yocto("100"));
    println!("<<----- owner and 2 farmers prepared.");

    // println!("----->> Prepare ref-exchange and swap pool.");
    let (_, token1, _) = prepair_pool_and_liquidity(
        &root, &owner, farming_id(), vec![&farmer1, &farmer2]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmers.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(farmer2, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmers registered.");

    let nft_contract = deploy_nft_contract(&root, "nft-contract".to_string(), farmer1.account_id.clone());
    // create farm
    println!("----->> Create farm.");
    let farm_id = "dai$1#0".to_string();
    let mut nft_balance: HashMap<String, U128> = HashMap::new();
    nft_balance.insert("nft-contract@1".to_string(), U128(2000000000000000000000000));
    nft_balance.insert("nft-contract@2".to_string(), U128(4000000000000000000000000));
    nft_balance.insert("nft-contract@3".to_string(), U128(6000000000000000000000000));
    nft_balance.insert("nft-contract".to_string(), U128(6000000000000000000000000));

    let out_come = call!(
        owner,
        farming.create_simple_farm(
            HRSimpleFarmTerms{
                seed_id: format!("{}${}", token1.account_id(), "1"),
                reward_token: token1.valid_account_id(),
                start_at: 0,
                reward_per_session: to_yocto("1").into(),
                session_interval: 60,
            },
            None,
            Some(nft_balance),
            None
        ),
        deposit = to_yocto("1")
    );

    out_come.assert_success();
    assert_eq!(Value::String(farm_id.clone()), out_come.unwrap_json_value());
    println!("<<----- Farm {} created at #{}, ts:{}.",
             farm_id,
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

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

    println!("<<----- Farm {} deposit reward at #{}, ts:{}.",
             farm_id,
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    // farmer 1 staking nft
    println!("----->> Farmer1 staking nft.");

    let out_come = call!(
        farmer1,
        nft_contract.nft_transfer_call(
            to_va(farming_id()),
            "1".to_string(),
            None,
            None,
            format!("{}${}", token1.account_id(), "1")
        ),
        deposit = 1
    );

    out_come.assert_success();
    println!("{:?}", out_come.promise_results());
    println!("<<----- Farmer1 staked nft at #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 0, 0, 0, 0, 0);
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    println!("user_seeds {:?}", user_seeds);
    assert_eq!(user_seeds.get(&String::from("dai$1")).unwrap().0, 2000000000000000000000000);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, 0_u128);

    let user_nft_seeds = show_usernftseeds(&farming, farmer1.account_id(), false);
    println!("user_nft_seeds {:?}", user_nft_seeds);

    println!("----->> Farmer1 staking nft.");

    let out_come = call!(
        farmer1,
        nft_contract.nft_transfer_call(
            to_va(farming_id()),
            "2".to_string(),
            None,
            None,
            format!("{}${}", token1.account_id(), "1")
        ),
        1,
        DEFAULT_GAS
    );

    out_come.assert_success();
    // println!("{:?}", out_come.promise_results());
    println!("<<----- Farmer1 staked nft at #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 0, 0, 0, 0, 0);
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    println!("user_seeds {:?}", user_seeds);
    assert_eq!(user_seeds.get(&String::from("dai$1")).unwrap().0, 6000000000000000000000000);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, 0_u128);

    let user_nft_seeds = show_usernftseeds(&farming, farmer1.account_id(), false);
    println!("farmer1 user_nft_seeds {:?}", user_nft_seeds);

    println!("----->> Farmer1 staking nft.");

    let out_come = call!(
        farmer1,
        nft_contract.nft_transfer_call(
            to_va(farming_id()),
            "3".to_string(),
            None,
            None,
            format!("{}${}", token1.account_id(), "1")
        ),
        1,
        DEFAULT_GAS
    );

    out_come.assert_success();
    // println!("{:?}", out_come.promise_results());
    println!("<<----- Farmer1 staked nft at #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 0, 0, 0, 0, 0);
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    println!("user_seeds {:?}", user_seeds);
    assert_eq!(user_seeds.get(&String::from("dai$1")).unwrap().0, 12000000000000000000000000);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, 0_u128);

    let user_nft_seeds = show_usernftseeds(&farming, farmer1.account_id(), false);
    println!("farmer1 user_nft_seeds {:?}", user_nft_seeds);

    println!("----->> Farmer1 unstaking nft.");

    let out_come = call!(
        farmer1,
        farming.withdraw_nft(
            "dai$1".to_string(),
            "nft-contract".to_string(),
            "1".to_string()
        ),
        deposit = 1
    );

    out_come.assert_success();
    // println!("{:?}", out_come.promise_results());
    println!("<<----- Farmer1 unstaked nft at #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 0, 0, 0, 0, 0);
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai$1")).unwrap().0, 10000000000000000000000000);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, 0_u128);

    let user_nft_seeds = show_usernftseeds(&farming, farmer1.account_id(), false);
    println!("farmer1 user_nft_seeds {:?}", user_nft_seeds);

    // check farmer 2 nft seeds
    let user_nft_seeds = show_usernftseeds(&farming, farmer2.account_id(), false);
    println!("farmer2 user_nft_seeds {:?}", user_nft_seeds);

    println!("----->> Farmer1 send nft 1 to Farmer2.");

    let out_come = call!(
        farmer1,
        nft_contract.nft_transfer(
            to_va(farmer2.account_id.clone()),
            "1".to_string(),
            None,
            None
        ),
        1,
        DEFAULT_GAS
    );

    out_come.assert_success();

    println!("----->> Farmer2 staking nft.");

    let out_come = call!(
        farmer2,
        nft_contract.nft_transfer_call(
            to_va(farming_id()),
            "1".to_string(),
            None,
            None,
            format!("{}${}", token1.account_id(), "1")
        ),
        1,
        DEFAULT_GAS
    );

    out_come.assert_success();
    println!("<<----- Farmer2 staked nft at #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let user_nft_seeds = show_usernftseeds(&farming, farmer2.account_id(), false);
    println!("farmer2 user_nft_seeds {:?}", user_nft_seeds);

    println!("----->> move to 60 secs later.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("<<----- Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 1, 0, 0, to_yocto("1"), 0);
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, 833333333333333333333330);
    let unclaim = show_unclaim(&farming, farmer2.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, 166666666666666666666666);

    println!("----->> Farmer1 staking nft 4 which use default contract.");

    let out_come = call!(
        farmer1,
        nft_contract.nft_transfer_call(
            to_va(farming_id()),
            "4".to_string(),
            None,
            None,
            format!("{}${}", token1.account_id(), "1")
        ),
        1,
        DEFAULT_GAS
    );

    out_come.assert_success();
    println!("<<----- Farmer1 staked nft at #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let user_nft_seeds = show_usernftseeds(&farming, farmer1.account_id(), false);
    println!("farmer1 user_nft_seeds {:?}", user_nft_seeds);
}

#[test]
fn test_maximum_nft_mappings() {

    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    let farmer2 = root.create_user("farmer2".to_string(), to_yocto("100"));
    println!("<<----- owner and 2 farmers prepared.");

    // println!("----->> Prepare ref-exchange and swap pool.");
    let (_, token1, _) = prepair_pool_and_liquidity(
        &root, &owner, farming_id(), vec![&farmer1, &farmer2]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmers.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(farmer2, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmers registered.");

    // create farm
    println!("----->> Create farm.");
    let farm_id = "dai$1#0".to_string();
    let mut nft_balance: HashMap<String, U128> = HashMap::new();

    for i in 0..300 {
        nft_balance.insert(format!("{}@{}", "nft-contract", i.to_string()), U128(3000000000000000000000000));
    }

    let out_come = call!(
        owner,
        farming.create_simple_farm(
            HRSimpleFarmTerms{
                seed_id: format!("{}${}", token1.account_id(), "1"),
                reward_token: token1.valid_account_id(),
                start_at: 0,
                reward_per_session: to_yocto("1").into(),
                session_interval: 60,
            },
            None,
            Some(nft_balance),
            None
        ),
        deposit = to_yocto("1")
    );

    out_come.assert_success();
    assert_eq!(Value::String(farm_id.clone()), out_come.unwrap_json_value());
    println!("<<----- Farm {} created at #{}, ts:{}.",
             farm_id,
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    let seed_info = show_seedsinfo(&farming, false);
    println!("seed_info {:?}", seed_info);
}

#[test]
fn test_claim_and_withdraw() {
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    let farmer2 = root.create_user("farmer2".to_string(), to_yocto("100"));
    println!("<<----- owner and 2 farmers prepared.");

    // println!("----->> Prepare ref-exchange and swap pool.");
    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1, &farmer2]);
    // println!("<<----- The pool prepaired.");

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmers.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(farmer2, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmers registered.");

    // create farm
    println!("----->> Create farm.");
    let farm_id = "dai#0".to_string();
    let out_come = call!(
        owner,
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
    println!("<<----- Farm {} created at #{}, ts:{}.",
             farm_id,
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

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

    println!("<<----- Farm {} deposit reward at #{}, ts:{}.",
             farm_id,
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    // farmer1 staking lpt
    println!("----->> Farmer1 staking lpt.");
    let out_come = call!(
        farmer1,
        token1.ft_transfer_call(to_va(farming_id()), U128(to_yocto("1")), None, "".to_string()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 staked liquidity at #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 0, 0, 0, 0, 0);
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, 0_u128);

    println!("----->> move to 60 secs later.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("<<----- Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 1, 0, 0, to_yocto("1"), 0);
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("1"));

    // claim and withdraw
    let out_come = call!(
        farmer1,
        farming.claim_reward_by_farm_and_withdraw(farm_id.clone()),
        deposit = 1
    );

    out_come.assert_success();
    // println!("{:#?}", out_come.promise_results());
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, 0_u128);
    let reward = show_reward(&farming, farmer1.account_id(), token1.account_id(), false);
    assert_eq!(reward.0, 0_u128);
    println!("<<----- Farmer1 claimed reward by farmid, now #{}, ts:{}.", 
    root.borrow_runtime().current_block().block_height, 
    root.borrow_runtime().current_block().block_timestamp);
}
