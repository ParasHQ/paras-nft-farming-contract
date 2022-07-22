use crate::errors::*;
use crate::farm_seed::SeedType;
use crate::*;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::PromiseOrValue;

use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

pub type TokenId = String;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FarmArgs {
    pub transfer_type: String, // "seed", reward must use string only for farm_id
    pub seed_id: String,
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    /// transfer reward token with specific msg indicate
    /// which farm to be deposited to.
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let sender: AccountId = sender_id.into();
        let amount: u128 = amount.into();

        if msg.is_empty() {
            // ****** seed Token deposit in ********

            // if seed not exist, it will panic
            let seed_farm = self.get_seed(&env::predecessor_account_id());

            assert_eq!(seed_farm.get_ref().seed_type, SeedType::FT, "Cannot deposit FT to this seed");

            if amount < seed_farm.get_ref().min_deposit {
                env::panic(
                    format!(
                        "{} {}",
                        ERR34_BELOW_MIN_SEED_DEPOSITED,
                        seed_farm.get_ref().min_deposit
                    )
                    .as_bytes(),
                )
            }

            self.internal_seed_deposit(
                &env::predecessor_account_id(),
                &sender,
                amount.into(),
                SeedType::FT,
            );

            self.assert_storage_usage(&sender);

            env::log(
                format!(
                    "{} deposit FT seed {} with amount {}.",
                    sender,
                    env::predecessor_account_id(),
                    amount,
                )
                .as_bytes(),
            );
            PromiseOrValue::Value(U128(0))
        } else {
            // ****** reward Token deposit in ********
            let farm_id = msg
                .parse::<FarmId>()
                .expect(&format!("{}", ERR42_INVALID_FARM_ID));
            let mut farm = self.data().farms.get(&farm_id).expect(ERR41_FARM_NOT_EXIST);

            // update farm
            assert_eq!(
                farm.get_reward_token(),
                env::predecessor_account_id(),
                "{}",
                ERR44_INVALID_FARM_REWARD
            );
            if let Some(cur_remain) = farm.add_reward(&amount) {
                self.data_mut().farms.insert(&farm_id, &farm);
                let old_balance = self
                    .data()
                    .reward_info
                    .get(&env::predecessor_account_id())
                    .unwrap_or(0);
                self.data_mut()
                    .reward_info
                    .insert(&env::predecessor_account_id(), &(old_balance + amount));

                env::log(
                    format!(
                        "{} added {} Reward Token, Now has {} left",
                        sender, amount, cur_remain
                    )
                    .as_bytes(),
                );
                PromiseOrValue::Value(U128(0))
            } else {
                env::panic(format!("{}", ERR43_INVALID_FARM_STATUS).as_bytes())
            }
        }
    }
}

pub trait MFTTokenReceiver {
    fn mft_on_transfer(
        &mut self,
        token_id: String,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

// Receiving NFTs
#[near_bindgen]
impl NonFungibleTokenReceiver for Contract {
    fn nft_on_transfer(
        &mut self,
        _sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        let nft_contract_id = env::predecessor_account_id();
        let signer_id = env::signer_account_id();

        assert_ne!(
            nft_contract_id, signer_id,
            "Paras(farming): nft_on_approve should only be called via cross-contract call"
        );

        assert_eq!(
            previous_owner_id,
            signer_id,
            "Paras(farming): owner_id should be signer_id"
        );

        let deposit_res = self.internal_nft_deposit(&msg, &previous_owner_id.to_string(), &nft_contract_id, &token_id);
        if !deposit_res {
            panic!("Paras(farming): nft token does not exist on seed");
        }
        PromiseOrValue::Value(false)
    }
}
