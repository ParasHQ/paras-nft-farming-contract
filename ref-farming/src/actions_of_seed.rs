
use std::convert::TryInto;
use near_sdk::json_types::U128;
use near_sdk::{AccountId, Balance, PromiseResult};

use crate::event::{NearEvent, UnlockFTBalanceData, LockFTBalanceData};
use crate::utils::{assert_one_yocto, ext_multi_fungible_token, ext_fungible_token, ext_non_fungible_token, ext_self, wrap_mft_token_id, parse_seed_id, GAS_FOR_FT_TRANSFER, GAS_FOR_RESOLVE_TRANSFER, GAS_FOR_NFT_TRANSFER, FT_INDEX_TAG, get_nft_balance_equivalent, to_sec};
use crate::errors::*;
use crate::farm_seed::SeedType;
use crate::*;
use crate::simple_farm::{NFTTokenId, ContractNFTTokenId};
use crate::utils::NFT_DELIMETER;

#[near_bindgen]
impl Contract {

    pub fn force_upgrade_seed(&mut self, seed_id: SeedId) {
        self.assert_owner();
        let seed = self.get_seed_and_upgrade(&seed_id);
        self.data_mut().seeds.insert(&seed_id, &seed);
    }

    #[payable]
    pub fn withdraw_nft(&mut self, seed_id: SeedId, nft_contract_id: String, nft_token_id: NFTTokenId) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        self.internal_nft_withdraw(&seed_id, &sender_id, &nft_contract_id, &nft_token_id);

        // transfer nft back to the owner
        ext_non_fungible_token::nft_transfer(
            sender_id.clone(),
            nft_token_id.clone(),
            None,
            None,
            &nft_contract_id,
            1,
            GAS_FOR_NFT_TRANSFER
        )
        .then(ext_self::callback_post_withdraw_nft(
            seed_id,
            sender_id,
            nft_contract_id,
            nft_token_id,
            &env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_TRANSFER
        ));
    }

    #[payable]
    pub fn withdraw_seed(&mut self, seed_id: SeedId, amount: U128) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        let seed_contract_id: AccountId = seed_id.split(FT_INDEX_TAG).next().unwrap().to_string();
        let amount: Balance = amount.into();

        // update inner state
        let seed_type = self.internal_seed_withdraw(&seed_id, &sender_id, amount);

        match seed_type {
            SeedType::FT => {
                ext_fungible_token::ft_transfer(
                    sender_id.clone().try_into().unwrap(),
                    amount.into(),
                    None,
                    &seed_contract_id,
                    1,  // one yocto near
                    GAS_FOR_FT_TRANSFER,
                )
                    .then(ext_self::callback_post_withdraw_ft_seed(
                        seed_id,
                        sender_id,
                        amount.into(),
                        &env::current_account_id(),
                        0,
                        GAS_FOR_RESOLVE_TRANSFER,
                    ));
            },
            SeedType::NFT => {
                panic!("Use withdraw_nft for this");
            },
            SeedType::MFT => {
                let (receiver_id, token_id) = parse_seed_id(&seed_id);
                ext_multi_fungible_token::mft_transfer(
                    wrap_mft_token_id(&token_id),
                    sender_id.clone().try_into().unwrap(),
                    amount.into(),
                    None,
                    &receiver_id,
                    1,  // one yocto near
                    GAS_FOR_FT_TRANSFER,
                )
                    .then(ext_self::callback_post_withdraw_mft_seed(
                        seed_id,
                        sender_id,
                        amount.into(),
                        &env::current_account_id(),
                        0,
                        GAS_FOR_RESOLVE_TRANSFER,
                    ));
            }
        }
    }

    #[payable]
    pub fn lock_ft_balance(&mut self, seed_id: SeedId, amount: U128, duration: u32){
        assert_one_yocto();
        let sender_id = &env::predecessor_account_id();
        self.internal_lock_ft_balance(&seed_id, sender_id, &amount.into(), &duration);

        let farmer = self.get_farmer(&sender_id);
        let locked_seed = farmer.get_ref().get_locked_seed_with_retention_wrapped(&seed_id).unwrap();
        NearEvent::log_lock_ft_balance(LockFTBalanceData{
            account_id: sender_id.to_string(),
            seed_id: seed_id.to_string(),
            amount: amount.0.to_string(),
            duration,
            started_at: locked_seed.started_at,
            ended_at: locked_seed.ended_at,
        });
    }

    #[payable]
    pub fn unlock_ft_balance(&mut self, seed_id: SeedId, amount: U128){
        assert_one_yocto();
        let sender_id = &env::predecessor_account_id();
        self.internal_unlock_ft_balance(sender_id, &seed_id, &amount.into());

        NearEvent::log_unlock_ft_balance(UnlockFTBalanceData{
            account_id: sender_id.to_string(),
            seed_id: seed_id.to_string(),
            amount: amount.0.to_string()
        });
    }

    #[private]
    pub fn callback_post_withdraw_nft(
        &mut self,
        seed_id: SeedId,
        sender_id: AccountId,
        nft_contract_id: String,
        nft_token_id: String
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR25_CALLBACK_POST_WITHDRAW_INVALID
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                env::log(
                    format!(
                        "{} withdraw {} nft from {}, Callback failed.",
                        sender_id, nft_token_id, nft_contract_id
                    ).as_bytes()
                );

                // revert withdraw

                let mut farmer = self.get_farmer(&sender_id);
                let mut farm_seed = self.get_seed(&seed_id);

                let contract_nft_token_id : ContractNFTTokenId = format!("{}{}{}", nft_contract_id, NFT_DELIMETER, nft_token_id);
                let nft_balance = self.data().nft_balance_seeds.get(&seed_id).unwrap();
                if let Some(nft_balance_equivalent) = get_nft_balance_equivalent(nft_balance, contract_nft_token_id.clone()) {
                    self.internal_claim_user_reward_by_seed_id(&sender_id, &seed_id);

                    farmer.get_ref_mut().add_nft(&seed_id, contract_nft_token_id);

                    farmer.get_ref_mut().add_seed(&seed_id, nft_balance_equivalent);
                    self.data_mut().farmers.insert(&sender_id, &farmer);

                    // **** update seed (new version)
                    farm_seed.get_ref_mut().add_amount(nft_balance_equivalent);
                    self.data_mut().seeds.insert(&seed_id, &farm_seed);
                }
            },
            PromiseResult::Successful(_) => {
                env::log(
                    format!(
                        "{} withdraw {} nft from {}, Succeed.",
                        sender_id, nft_token_id, nft_contract_id
                    ).as_bytes()
                );
            }
        }
    }
    #[private]
    pub fn callback_post_withdraw_ft_seed(
        &mut self,
        seed_id: SeedId,
        sender_id: AccountId,
        amount: U128,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR25_CALLBACK_POST_WITHDRAW_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                env::log(
                    format!(
                        "{} withdraw {} ft seed with amount {}, Callback Failed.",
                        sender_id, seed_id, amount,
                    )
                    .as_bytes(),
                );
                // revert withdraw, equal to deposit, claim reward to update user reward_per_seed
                self.internal_claim_user_reward_by_seed_id(&sender_id, &seed_id);
                // **** update seed (new version)
                let mut farm_seed = self.get_seed(&seed_id);
                farm_seed.get_ref_mut().add_amount(amount);
                self.data_mut().seeds.insert(&seed_id, &farm_seed);

                let mut farmer = self.get_farmer(&sender_id);
                farmer.get_ref_mut().add_seed(&seed_id, amount);
                self.data_mut().farmers.insert(&sender_id, &farmer);
            },
            PromiseResult::Successful(_) => {
                env::log(
                    format!(
                        "{} withdraw {} ft seed with amount {}, Succeed.",
                        sender_id, seed_id, amount,
                    )
                    .as_bytes(),
                );
            }
        };
    }

    #[private]
    pub fn callback_post_withdraw_mft_seed(
        &mut self,
        seed_id: SeedId,
        sender_id: AccountId,
        amount: U128,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR25_CALLBACK_POST_WITHDRAW_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                env::log(
                    format!(
                        "{} withdraw {} mft seed with amount {}, Callback Failed.",
                        sender_id, seed_id, amount,
                    )
                    .as_bytes(),
                );
                // revert withdraw, equal to deposit, claim reward to update user reward_per_seed

                self.internal_claim_user_reward_by_seed_id(&sender_id, &seed_id);
                // **** update seed (new version)
                let mut farm_seed = self.get_seed(&seed_id);
                farm_seed.get_ref_mut().add_amount(amount);
                self.data_mut().seeds.insert(&seed_id, &farm_seed);

                let mut farmer = self.get_farmer(&sender_id);
                farmer.get_ref_mut().add_seed(&seed_id, amount);
                self.data_mut().farmers.insert(&sender_id, &farmer);
            },
            PromiseResult::Successful(_) => {
                env::log(
                    format!(
                        "{} withdraw {} mft seed with amount {}, Succeed.",
                        sender_id, seed_id, amount,
                    )
                    .as_bytes(),
                );
            }
        };
    }
}


/// Internal methods implementation.
impl Contract {

    #[inline]
    pub(crate) fn get_seed_and_upgrade(&mut self, seed_id: &String) -> VersionedFarmSeed {
        let orig = self.data().seeds.get(seed_id).expect(&format!("{}", ERR31_SEED_NOT_EXIST));
        if orig.need_upgrade() {
            orig.upgrade(self)
        } else {
            orig
        }
    }

    #[inline]
    pub(crate) fn get_seed(&self, seed_id: &String) -> VersionedFarmSeed {
        let orig = self.data().seeds.get(seed_id).expect(&format!("{}", ERR31_SEED_NOT_EXIST));
        if orig.need_upgrade() {
            panic!("Need upgrade");
        } else {
            orig
        } 
    }

    #[inline]
    pub(crate) fn get_seed_wrapped(&self, seed_id: &String) -> Option<VersionedFarmSeed> {
        if let Some(farm_seed) = self.data().seeds.get(seed_id) {
            if farm_seed.need_upgrade() {
                panic!("Need upgrade");
            } else {
                Some(farm_seed)
            }
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn is_seed_type(&self, seed_id: &String, seed_type: SeedType) -> bool {
        if let Some(farm_seed) = self.data().seeds.get(seed_id) {
            if farm_seed.get_ref().seed_type == seed_type{
                return true
            }
        }
        return false
    }

    pub(crate) fn internal_seed_deposit(
        &mut self, 
        seed_id: &String, 
        sender_id: &AccountId, 
        amount: Balance, 
        _seed_type: SeedType) {

        // first claim all reward of the user for this seed farms
        // to update user reward_per_seed in each farm
        self.internal_claim_user_reward_by_seed_id(sender_id, seed_id);

        let mut farm_seed = self.get_seed(seed_id);

        let mut farmer = self.get_farmer(sender_id);

        // **** update seed (new version)
        farm_seed.get_ref_mut().add_amount(amount);
        self.data_mut().seeds.insert(&seed_id, &farm_seed);

        farmer.get_ref_mut().add_seed(&seed_id, amount);
        self.data_mut().farmers.insert(sender_id, &farmer);

        let mut reward_tokens: Vec<AccountId> = vec![];
        for farm_id in farm_seed.get_ref().farms.iter() {
            let reward_token = self.data().farms.get(farm_id).unwrap().get_reward_token();
            if !reward_tokens.contains(&reward_token) {
                if farmer.get_ref().rewards.get(&reward_token).is_some() {
                    self.private_withdraw_reward(reward_token.clone(), sender_id.to_string(), None);
                }
                reward_tokens.push(reward_token);
            }
        };
    }

    pub(crate) fn internal_seed_redeposit(
        &mut self,
        seed_id: &String,
        sender_id: &AccountId,
        _is_deposit_seed_reward: bool,
    ) {
        self.internal_claim_user_reward_by_seed_id(&sender_id, seed_id);

        let mut farm_seed = self.get_seed(seed_id);
        let mut farmer = self.get_farmer(sender_id);

        let amount = if farmer.get_ref().rewards.get(seed_id).is_some() {
            farmer.get_ref_mut().sub_reward(&seed_id, 0)
        } else {
            0
        };

        // **** update seed (new version)
        farm_seed.get_ref_mut().add_amount(amount);
        self.data_mut().seeds.insert(&seed_id, &farm_seed);

        farmer.get_ref_mut().add_seed(&seed_id, amount);
        self.data_mut().farmers.insert(sender_id, &farmer);

        let mut reward_tokens: Vec<AccountId> = vec![];

        for farm_id in farm_seed.get_ref().farms.iter() {
            let reward_token = self.data().farms.get(farm_id).unwrap().get_reward_token();
            if !reward_tokens.contains(&reward_token) {
                if farmer.get_ref().rewards.get(&reward_token).is_some() {
                    self.private_withdraw_reward(reward_token.clone(), sender_id.to_string(), None);
                }
                reward_tokens.push(reward_token);
            }
        };
    }

    fn internal_seed_withdraw(
        &mut self, 
        seed_id: &SeedId, 
        sender_id: &AccountId, 
        amount: Balance) -> SeedType {

        // first claim all reward of the user for this seed farms
        // to update user reward_per_seed in each farm
        self.internal_claim_user_reward_by_seed_id(sender_id, seed_id);

        let mut farm_seed = self.get_seed(seed_id);
        let mut farmer = self.get_farmer(sender_id);

        // Then update user seed and total seed of this LPT
        let farmer_seed_remain = farmer.get_ref_mut().sub_seed(seed_id, amount);
        farm_seed.get_ref_mut().sub_amount(amount);

        farmer.get_ref_mut().delete_expired_locked_seed(seed_id);

        if farmer_seed_remain == 0 {
            // remove farmer rps of relative farm
            for farm_id in farm_seed.get_ref().farms.iter() {
                farmer.get_ref_mut().remove_rps(farm_id);
            }
        }
        self.data_mut().farmers.insert(sender_id, &farmer);
        self.data_mut().seeds.insert(seed_id, &farm_seed);

        let mut reward_tokens: Vec<AccountId> = vec![];
        for farm_id in farm_seed.get_ref().farms.iter() {
            let reward_token = self.data().farms.get(farm_id).unwrap().get_reward_token();
            if !reward_tokens.contains(&reward_token) {
                if farmer.get_ref().rewards.get(&reward_token).is_some() {
                    self.private_withdraw_reward(reward_token.clone(), sender_id.to_string(), None);
                }
                reward_tokens.push(reward_token);
            }
        };

        farm_seed.get_ref().seed_type.clone()
    }

    pub(crate) fn internal_nft_deposit(
        &mut self,
        seed_id: &String,
        sender_id: &AccountId,
        nft_contract_id: &String,
        nft_token_id: &String,
    ) -> bool {
        let mut farm_seed = self.get_seed(seed_id);

        assert_eq!(farm_seed.get_ref().seed_type, SeedType::NFT, "Cannot deposit NFT to this farm");

        // update farmer seed
        let contract_nft_token_id = format!("{}{}{}", nft_contract_id, NFT_DELIMETER, nft_token_id);
        let nft_balance = self.data().nft_balance_seeds.get(&seed_id).unwrap();
        return if let Some(nft_balance_equivalent) = get_nft_balance_equivalent(nft_balance, contract_nft_token_id.clone()) {
            // first claim all reward of the user for this seed farms
            // to update user reward_per_seed in each farm
            self.internal_claim_user_reward_by_seed_id(sender_id, seed_id);
            let mut farmer = self.get_farmer(sender_id);
            farmer.get_ref_mut().add_nft(seed_id, contract_nft_token_id);

            farmer.get_ref_mut().add_seed(seed_id, nft_balance_equivalent);
            self.data_mut().farmers.insert(sender_id, &farmer);

            // **** update seed (new version)
            farm_seed.get_ref_mut().add_amount(nft_balance_equivalent);
            self.data_mut().seeds.insert(&seed_id, &farm_seed);

            let mut reward_tokens: Vec<AccountId> = vec![];
            for farm_id in farm_seed.get_ref().farms.iter() {
                let reward_token = self.data().farms.get(farm_id).unwrap().get_reward_token();
                if !reward_tokens.contains(&reward_token) {
                    if farmer.get_ref().rewards.get(&reward_token).is_some() {
                        self.private_withdraw_reward(reward_token.clone(), sender_id.to_string(), None);
                    }
                    reward_tokens.push(reward_token);
                }
            };

            true
        } else {
            false
        }
    }

    fn internal_nft_withdraw(
        &mut self,
        seed_id: &String,
        sender_id: &AccountId,
        nft_contract_id: &String,
        nft_token_id: &String
    ) -> ContractNFTTokenId {
        self.internal_claim_user_reward_by_seed_id(sender_id, seed_id);

        let mut farm_seed = self.get_seed(seed_id);
        let mut farmer = self.get_farmer(sender_id);

        // sub nft
        let contract_nft_token_id : ContractNFTTokenId = format!("{}{}{}", nft_contract_id, NFT_DELIMETER, nft_token_id);
        farmer.get_ref_mut().sub_nft(seed_id, contract_nft_token_id.clone());
        let nft_balance = self.data().nft_balance_seeds.get(&seed_id).unwrap();
        let nft_balance_equivalent: Balance = get_nft_balance_equivalent(nft_balance, contract_nft_token_id.clone()).unwrap();

        let farmer_seed_remain = farmer.get_ref_mut().sub_seed(seed_id, nft_balance_equivalent);

        // calculate farm_seed after multiplier get removed
        farm_seed.get_ref_mut().sub_amount(nft_balance_equivalent);

        if farmer_seed_remain == 0 {
            // remove farmer rps of relative farm
            for farm_id in farm_seed.get_ref().farms.iter() {
                farmer.get_ref_mut().remove_rps(farm_id);
            }
        }

        self.data_mut().farmers.insert(sender_id, &farmer);
        self.data_mut().seeds.insert(seed_id, &farm_seed);

        let mut reward_tokens: Vec<AccountId> = vec![];
        for farm_id in farm_seed.get_ref().farms.iter() {
            let reward_token = self.data().farms.get(farm_id).unwrap().get_reward_token();
            if !reward_tokens.contains(&reward_token) {
                if farmer.get_ref().rewards.get(&reward_token).is_some() {
                    self.private_withdraw_reward(reward_token.clone(), sender_id.to_string(), None);
                }
                reward_tokens.push(reward_token);
            }
        };

        contract_nft_token_id
    }


    pub fn internal_lock_ft_balance(&mut self, seed_id: &SeedId, sender_id: &AccountId, amount: &Balance, duration: &u32){
        let current_block_time = to_sec(env::block_timestamp());
        let ended_at = current_block_time + duration;

        assert!(self.is_seed_type(&seed_id, SeedType::FT), "{}", ERR36_SEED_TYPE_IS_NOT_FT);

        let mut farmer = self.get_farmer(&sender_id);
        
        let user_balance = &farmer.get_ref().get_available_balance(&seed_id);
        assert!(user_balance >= &amount, "{}", ERR37_BALANCE_IS_NOT_ENOUGH);

        if let Some(previous_locked_seed) = farmer.get_ref().get_locked_seed_with_retention_wrapped(seed_id){
            assert!(previous_locked_seed.ended_at <= ended_at, "{}", ERR38_END_OF_DURATION_IS_LESS_THAN_ENDED_AT);
        } 

        farmer.get_ref_mut().add_or_create_locked_seed(&seed_id, *amount, current_block_time, ended_at);
        self.data_mut().farmers.insert(&sender_id, &farmer);
    }


    pub fn internal_unlock_ft_balance(&mut self, sender_id: &AccountId, seed_id: &SeedId, amount: &Balance){
        assert_one_yocto();

        let current_block_time = to_sec(env::block_timestamp());

        assert!(self.is_seed_type(&seed_id, SeedType::FT), "{}", ERR36_SEED_TYPE_IS_NOT_FT);

        let mut farmer = self.get_farmer(&sender_id);
        if let Some(locked_seed) = farmer.get_ref().get_locked_seed_with_retention_wrapped(seed_id){
            assert!(locked_seed.ended_at <= current_block_time, "{}", ERR39_USER_CANNOT_UNLOCK_SEED);

            farmer.get_ref_mut().sub_locked_seed_balance(seed_id, *amount);
            self.data_mut().farmers.insert(&sender_id, &farmer);
        } else {
            farmer.get_ref_mut().delete_expired_locked_seed(seed_id);
            self.data_mut().farmers.insert(&sender_id, &farmer);

            env::panic(format!("{}", ERR40_USER_DOES_NOT_HAVE_LOCKED_SEED).as_bytes());
        }
    }
}
