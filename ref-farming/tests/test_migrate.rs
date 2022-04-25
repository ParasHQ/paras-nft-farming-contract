use std::convert::TryFrom;

use near_sdk::json_types::ValidAccountId;
use near_sdk_sim::{deploy, init_simulator, to_yocto};
use near_sdk::serde_json::json;


use ref_farming::ContractContract as Farming;

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
        init_method: new(ValidAccountId::try_from(root.account_id.clone()).unwrap(), None, None, None)
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

    farming.user_account.deploy(&FARMING_WASM_BYTES, farming.user_account.account_id.clone(), 100);

    let result = farming.user_account.call(
        farming.user_account.account_id.clone(),
        "migrate",
        format!("").as_bytes(),
        near_sdk_sim::DEFAULT_GAS,
        0,
    )
        .status();
}
