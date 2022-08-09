use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use near_sdk_sim::UserAccount;
use near_sdk_sim::near_crypto::InMemorySigner;
use near_sdk_sim::runtime::{GenesisConfig, RuntimeStandalone};
use near_sdk_sim::{call, init_simulator, to_yocto};
use near_sdk::json_types::U128;
use near_sdk::serde_json::Value;

use ref_farming::HRSimpleFarmTerms;

use crate::common::utils::*;
use crate::common::views::*;
use crate::common::actions::*;
use crate::common::init::deploy_farming;

mod common;

#[test]
fn locked_seed_rewards(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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

    println!("----->> Farmer1 lock ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("1").into(), 50u32),
        deposit = 1
    );
    out_come.assert_success();
    show_userlockedseeds(&farming, farmer1.account_id(), true);
    println!("<<----- Farmer1 lock ft token");

    println!("----->> move to 60 secs later.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("<<----- Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 2, 0, 0, to_yocto("2"), 0);
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("2"));

    println!("----->> Farmer1 unlock ft token");
    let out_come = call!(
        farmer1,
        farming.unlock_ft_balance(token1.account_id(), to_yocto("1").into()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 unlock ft token");

    println!("----->> move to 60 secs later.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("<<----- Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto("10"), 3, 0, 0, to_yocto("3"), 0);
    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("1"));
    let unclaim = show_unclaim(&farming, farmer1.account_id(), farm_id.clone(), false);
    assert_eq!(unclaim.0, to_yocto("3"));
}


#[test]
fn locked_seed_update_locked_balance(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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

    println!("----->> Farmer1 lock 1st ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.1").into(), 10u32),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 lock 1st ft token at ts{}", root.borrow_runtime().current_block().block_timestamp);

    println!("----->> Farmer1 lock 2nd ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.1").into(), 100u32),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 lock 2nd ft token");

    let locked_seeds = show_userlockedseeds(&farming, farmer1.account_id(), true);
    let locked_seed = locked_seeds.get(&token1.account_id()).unwrap();
    assert!(locked_seed.balance == to_yocto("0.2").into());
}

#[test]
fn locked_seed_e36_seed_type_is_not_ft(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

    // create farm
    println!("----->> Create farm.");
    // create fake type NFT
    let farm_id = "dai$1#0".to_string();
    let seed_id = format!("{}${}", token1.account_id(), "1"); 

    let mut nft_balance: HashMap<String, U128> = HashMap::new();
    nft_balance.insert("nft-contract@1".to_string(), U128(2000000000000000000000000));

    let out_come = call!(
        owner,
        farming.create_simple_farm(HRSimpleFarmTerms{
            seed_id: seed_id.clone(),
            reward_token: token1.valid_account_id(),
            start_at: 0,
            reward_per_session: to_yocto("1").into(),
            session_interval: 60,
        }, None, Some(nft_balance), None),
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

    println!("----->> Farmer1 lock ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(seed_id.clone(), to_yocto("1").into(), 50u32),
        deposit = 1
    );
    assert!(!out_come.is_ok());

    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("E36: seed type is not FT"));
}

#[test]
fn locked_seed_e37_balance_is_not_enough(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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

    println!("----->> Farmer1 lock ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("100").into(), 50u32),
        deposit = 1
    );
    assert!(!out_come.is_ok());

    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("E37: balance is not enough"));
}

#[test]
fn locked_seed_e38_end_duration_is_less_than_ended_at(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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

    println!("----->> Farmer1 lock 1st ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.1").into(), 100u32),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 lock 1st ft token");

    println!("----->> Farmer1 lock 2nd ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.1").into(), 10u32),
        deposit = 1
    );
    assert!(!out_come.is_ok());
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("E38: end of duration is less than previous ended at"));
}

#[test]
fn locked_seed_e39_user_cannot_unlock_seed(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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

    println!("----->> Farmer1 lock 1st ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.1").into(), 100u32),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 lock 1st ft token");

    println!("----->> Farmer1 unlock ft token");
    let out_come = call!(
        farmer1,
        farming.unlock_ft_balance(token1.account_id(), to_yocto("0.1").into()),
        deposit = 1
    );
    assert!(!out_come.is_ok());
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("E39: user cannot unlock seed"));
}

#[test]
fn locked_seed_e40_user_does_not_have_locked_seed(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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

    println!("----->> Farmer1 unlock ft token");
    let out_come = call!(
        farmer1,
        farming.unlock_ft_balance(token1.account_id(), to_yocto("0.5").into()),
        deposit = 1
    );
    assert!(!out_come.is_ok());
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("E40: user does not have locked seed"));
}

#[test]
fn locked_seed_normal_withdraw(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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

    println!("----->> Farmer1 lock ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.5").into(), 10u32),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 lock ft token at ts{}", root.borrow_runtime().current_block().block_timestamp);

    println!("----->> move to 60 secs later.");
    assert!(root.borrow_runtime_mut().produce_blocks(15).is_ok());
    println!("<<----- Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    println!("----->> Farmer1 unlock ft token");
    let out_come = call!(
        farmer1,
        farming.unlock_ft_balance(token1.account_id(), to_yocto("0.5").into()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 unlock ft token");

    println!("----->> Withdraw seed");
    let out_come = call!(
        farmer1,
        farming.withdraw_seed(token1.account_id(), to_yocto("1").into()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Withdraw seed");
}

#[test]
fn locked_seed_withdraw_locked_seed_e32_not_enough_amount_of_seed(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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

    println!("----->> Farmer1 lock ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.5").into(), 10u32),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 lock ft token at ts{}", root.borrow_runtime().current_block().block_timestamp);

    println!("----->> Withdraw seed");
    let out_come = call!(
        farmer1,
        farming.withdraw_seed(token1.account_id(), to_yocto("1").into()),
        deposit = 1
    );
    assert!(!out_come.is_ok());
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("E32: not enough amount of seed"));
}

#[test]
fn locked_seed_withdraw_retention_locked_seed(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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

    println!("----->> Farmer1 lock ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.5").into(), 1u32),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 lock ft token at ts{}", root.borrow_runtime().current_block().block_timestamp);

    println!("----->> move to 25 hours later.");
    assert!(root.borrow_runtime_mut().produce_blocks(60 * 60 * 25).is_ok());
    println!("<<----- Chain goes {} blocks, now #{}, ts:{}.",
             60 * 60 * 25,
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    // user can withdraw without unlock the seed if the seed ended_at is reached the limit retention tolerance
    println!("----->> Withdraw seed");
    let out_come = call!(
        farmer1,
        farming.withdraw_seed(token1.account_id(), to_yocto("1").into()),
        deposit = 1
    );
    out_come.assert_success();

    let locked_seeds = show_userlockedseeds(&farming, farmer1.account_id(), true);
    assert!(locked_seeds.get(&token1.account_id()).is_none());
}

#[test]
fn locked_seed_unlock_x_amount_balance(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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

    println!("----->> Farmer1 lock 1st ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.1").into(), 10u32),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 lock 1st ft token at ts{}", root.borrow_runtime().current_block().block_timestamp);

        println!("----->> move to 60 secs later.");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("<<----- Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    println!("----->> Farmer1 unlock 0.01 balance");
        let out_come = call!(
        farmer1,
        farming.unlock_ft_balance(token1.account_id(), to_yocto("0.01").into()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 unlock 0.01 balance");

    let locked_seeds = show_userlockedseeds(&farming, farmer1.account_id(), true);
    let locked_seed = locked_seeds.get(&token1.account_id()).unwrap();
    assert!(locked_seed.balance == to_yocto("0.09").into());
}

#[test]
fn locked_seed_without_unlock_balance(){
    let root = init_simulator(None);

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, farming_id(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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

    println!("----->> Farmer1 lock ft token");
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.5").into(), 10u32),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 lock ft token at ts{}", root.borrow_runtime().current_block().block_timestamp);

    println!("----->> move to 60 secs later.");
    assert!(root.borrow_runtime_mut().produce_blocks(15).is_ok());
    println!("<<----- Chain goes 60 blocks, now #{}, ts:{}.",
             root.borrow_runtime().current_block().block_height,
             root.borrow_runtime().current_block().block_timestamp);

    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("1"));

    println!("----->> 1st withdraw seed");
    let out_come = call!(
        farmer1,
        farming.withdraw_seed(token1.account_id(), to_yocto("0.1").into()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- 1st withdraw seed");

    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("0.9"));

    println!("----->> Farmer1 unlock ft token");
    let out_come = call!(
        farmer1,
        farming.unlock_ft_balance(token1.account_id(), to_yocto("0.5").into()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 unlock ft token");

    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("0.9"));

    println!("----->> 2nd withdraw seed");
    let out_come = call!(
        farmer1,
        farming.withdraw_seed(token1.account_id(), to_yocto("0.8").into()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- 2nd withdraw seed");

    let user_seeds = show_userseeds(&farming, farmer1.account_id(), false);
    assert_eq!(user_seeds.get(&String::from("dai")).unwrap().0, to_yocto("0.1"));
}

pub fn init_runtime_near(
    genesis_config: Option<GenesisConfig>,
) -> (RuntimeStandalone, InMemorySigner, String) {
    let mut genesis = genesis_config.unwrap_or_default();
    genesis.runtime_config.wasm_config.limit_config.max_total_prepaid_gas = genesis.gas_limit;
    let root_account_id = "near".to_string();
    let signer = genesis.init_root_signer(&root_account_id);
    let runtime = RuntimeStandalone::new_with_store(genesis);
    (runtime, signer, root_account_id)
}

// init custom simulator with "near" as root account
pub fn init_simulator_near(genesis_config: Option<GenesisConfig>) -> UserAccount {
    let (runtime, signer, root_account_id) = init_runtime_near(genesis_config);
    UserAccount::new(&Rc::new(RefCell::new(runtime)), root_account_id, signer)
}

#[test]
fn locked_seed_validate_duration(){
    let root = init_simulator_near(None);
    let paras = root.create_user("paras.near".to_string(), to_yocto("1000"));

    println!("----->> Prepare accounts.");
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));
    println!("<<----- owner and 1 farmer prepared.");

    let (_, token1, _) = prepair_pool_and_liquidity(
         &root, &owner, "staking.paras.near".to_string(), vec![&farmer1]);

    // deploy farming contract and register user
    println!("----->> Deploy farming and register farmer.");
    let farming = deploy_farming(&paras, "staking.paras.near".to_string(), owner.account_id());
    call!(farmer1, farming.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    println!("<<----- farming deployed, farmer registered.");

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
        token1.storage_deposit(Some(to_va("staking.paras.near".to_string())), None),
        deposit = to_yocto("1")
    );

    mint_token(&token1, &root, to_yocto("10"));

    println!("----->> Deposit reward to turn farm Running.");
    call!(
        root,
        token1.ft_transfer_call(to_va("staking.paras.near".to_string()), U128(to_yocto("10")), None, farm_id.clone()),
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
        token1.ft_transfer_call(to_va("staking.paras.near".to_string()), U128(to_yocto("1")), None, "".to_string()),
        deposit = 1
    );
    out_come.assert_success();

    println!("----->> Farmer1 lock ft token 30 days");
    let _30_days_in_second: u32 = 60 * 60 * 24 * 30;
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.5").into(), _30_days_in_second),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 lock ft token 30 days at ts{}", root.borrow_runtime().current_block().block_timestamp);

    println!("----->> Farmer1 lock ft token 90 days");
    let _90_days_in_second: u32 = 60 * 60 * 24 * 90;
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.5").into(), _90_days_in_second),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 lock ft token 90 days at ts{}", root.borrow_runtime().current_block().block_timestamp);

    println!("----->> Farmer1 lock ft token 91 days should be error");
    let _90_days_in_second: u32 = 60 * 60 * 24 * 91;
    let out_come = call!(
        farmer1,
        farming.lock_ft_balance(token1.account_id(), to_yocto("0.5").into(), _90_days_in_second),
        deposit = 1
    );
    assert!(!out_come.is_ok());

    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("E401: lock ft balance duration is not valid"));
}
