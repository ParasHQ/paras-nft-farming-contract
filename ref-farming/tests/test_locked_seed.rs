use near_sdk_sim::{call, init_simulator, to_yocto, DEFAULT_GAS};
use near_sdk::json_types::U128;
use near_sdk::serde_json::Value;
use near_sdk::serde_json::json;

use ref_farming::HRSimpleFarmTerms;

use crate::common::utils::*;
use crate::common::views::*;
use crate::common::actions::*;
use crate::common::init::deploy_farming;

mod common;

#[test]
fn locked_seed(){
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
    println!("<<----- Farmer1 lock ft token");
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
        farming.unlock_ft_balance(token1.account_id()),
        deposit = 1
    );
    out_come.assert_success();
    println!("<<----- Farmer1 unlock ft token");

    // TODO unclock staking
    // TODO withdraw with lock staking

    // TODO farmer 1 staking 
    // TODO lock staking 
    // TODO withdraw
}
