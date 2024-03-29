//! Farmer records a farmer's 
//! * all claimed reward tokens, 
//! * all seeds he staked,
//! * user_rps per farm,
//! and the deposited near amount prepaid as storage fee


use std::collections::HashMap;
use near_sdk::collections::LookupMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::{env, AccountId, Balance};
use crate::{SeedId, FarmId, RPS};
use crate::simple_farm::ContractNFTTokenId;
use crate::errors::*;
use crate::utils::{MAX_ACCOUNT_LENGTH, TimestampSec, to_sec};
use crate::StorageKeys;

use near_sdk::collections::UnorderedSet;

/// each entry cost MAX_ACCOUNT_LENGTH bytes,
/// amount: Balance cost 16 bytes
/// each empty hashmap cost 4 bytes
pub const MIN_FARMER_LENGTH: u128 = MAX_ACCOUNT_LENGTH + 16 + 4 * 3;

/// retention is used to invalidate the locked_seed when the user forgot to unlock the balance 
pub const LOCKED_SEED_RETENTION: TimestampSec = 60 * 60 * 24;

/// Account deposits information and storage cost (LEGACY).
#[derive(BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct FarmerV101 {
    pub farmer_id: AccountId,
    /// Native NEAR amount sent to this contract.
    /// Used for storage.
    pub amount: Balance,
    /// Amounts of various reward tokens the farmer claimed.
    pub rewards: HashMap<AccountId, Balance>,
    /// Amounts of various seed tokens the farmer staked.
    pub seeds: HashMap<SeedId, Balance>,
    /// record user_last_rps of farms
    pub user_rps: LookupMap<FarmId, RPS>,
    pub rps_count: u32,
    pub nft_seeds: HashMap<SeedId, UnorderedSet<ContractNFTTokenId>>,
}

impl From<FarmerV101> for Farmer{
    fn from (f: FarmerV101) -> Self{
        let FarmerV101 { farmer_id, amount, rewards, seeds, user_rps, rps_count, nft_seeds } = f;

        Self{
            farmer_id,
            amount,
            rewards,
            seeds,
            user_rps,
            rps_count,
            nft_seeds,

            // added new locked seeds 
            locked_seeds: HashMap::new()
        }

    }
}

#[derive(Serialize, BorshSerialize, BorshDeserialize, Default)]
#[cfg_attr(feature = "test", derive(Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct LockedSeed {
    pub balance: Balance,
    pub started_at: TimestampSec,
    pub ended_at: TimestampSec 
}

/// Account deposits information and storage cost.
#[derive(BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Farmer {
    pub farmer_id: AccountId,
    /// Native NEAR amount sent to this contract.
    /// Used for storage.
    pub amount: Balance,
    /// Amounts of various reward tokens the farmer claimed.
    pub rewards: HashMap<AccountId, Balance>,
    /// Amounts of various seed tokens the farmer staked.
    pub seeds: HashMap<SeedId, Balance>,
    /// record user_last_rps of farms
    pub user_rps: LookupMap<FarmId, RPS>,
    pub rps_count: u32,
    pub nft_seeds: HashMap<SeedId, UnorderedSet<ContractNFTTokenId>>,
    pub locked_seeds: HashMap<SeedId, LockedSeed>,
}

impl Farmer {

    /// Adds amount to the balance of given token
    pub(crate) fn add_reward(&mut self, token: &AccountId, amount: Balance) {
        if let Some(x) = self.rewards.get_mut(token) {
            *x = *x + amount;
        } else {
            self.rewards.insert(token.clone(), amount);
        }
    }

    /// Subtract from `reward` balance.
    /// if amount == 0, subtract all reward balance.
    /// Panics if `amount` is bigger than the current balance.
    /// return actual subtract amount
    pub(crate) fn sub_reward(&mut self, token: &AccountId, amount: Balance) -> Balance {
        let value = *self.rewards.get(token).expect(ERR21_TOKEN_NOT_REG);
        assert!(value >= amount, "{}", ERR22_NOT_ENOUGH_TOKENS);
        if amount == 0 {
            self.rewards.remove(&token.clone());
            value
        } else {
            self.rewards.insert(token.clone(), value - amount);
            amount
        }
    }

    pub fn add_seed(&mut self, seed_id: &SeedId, amount: Balance) {
        if amount > 0 {
            self.seeds.insert(
                seed_id.clone(), 
                amount + self.seeds.get(seed_id).unwrap_or(&0_u128)
            );
        }
        
    }

    /// return seed remained.
    pub fn sub_seed(&mut self, seed_id: &SeedId, amount: Balance) -> Balance {
        self.seeds.get(seed_id).expect(&format!("{}", ERR31_SEED_NOT_EXIST));

        let available_balance = self.get_available_balance(seed_id);
        assert!(available_balance >= amount, "{}", ERR32_NOT_ENOUGH_SEED);

        let user_balance = self.get_balance(seed_id);
        let cur_balance = user_balance - amount;
        if cur_balance > 0 {
            self.seeds.insert(seed_id.clone(), cur_balance);
        } else {
            self.seeds.remove(seed_id);
        }
        cur_balance
    }

    /// return locked seed remained.
    pub fn sub_locked_seed_balance(&mut self, seed_id: &SeedId, amount: Balance) -> Balance {
        let locked_seed = self.get_locked_seed_with_retention_wrapped(seed_id).expect(&format!("{}", ERR40_USER_DOES_NOT_HAVE_LOCKED_SEED));

        assert!(locked_seed.balance >= amount, "{}", ERR321_NOT_ENOUGH_LOCKED_SEED);

        let cur_locked_balance = locked_seed.balance - amount;
        if cur_locked_balance > 0 {
            let curr_locked_seed = LockedSeed{
                started_at: locked_seed.started_at,
                ended_at: locked_seed.ended_at,
                balance: cur_locked_balance
            };
            self.locked_seeds.insert(seed_id.clone(), curr_locked_seed);
        } else {
            self.locked_seeds.remove(seed_id);
        }
        
        cur_locked_balance
    }

    pub fn get_rps(&self, farm_id: &FarmId) -> RPS {
        self.user_rps.get(farm_id).unwrap_or(RPS::default()).clone()
    }

    pub fn set_rps(&mut self, farm_id: &FarmId, rps: RPS) {
        if !self.user_rps.contains_key(farm_id) {
            self.rps_count += 1;
        } 
        self.user_rps.insert(farm_id, &rps);
    }

    pub fn remove_rps(&mut self, farm_id: &FarmId) {
        if self.user_rps.contains_key(farm_id) {
            self.user_rps.remove(farm_id);
            self.rps_count -= 1;
        }
    }

    /// Returns amount of yocto near necessary to cover storage used by this data structure.
    pub fn storage_usage(&self) -> Balance {
        (
            MIN_FARMER_LENGTH 
            + self.rewards.len() as u128 * (4 + MAX_ACCOUNT_LENGTH + 16)
            + self.seeds.len() as u128 * (4 + MAX_ACCOUNT_LENGTH + 16)
            + self.rps_count as u128 * (4 + 1 + 2 * MAX_ACCOUNT_LENGTH + 32)
        )
        * env::storage_byte_cost()
    }

    pub fn add_nft(&mut self, seed_id: &SeedId, contract_nft_token_id: ContractNFTTokenId) {
        if let Some(nft_contract_seed) = self.nft_seeds.get_mut(seed_id) {
            nft_contract_seed.insert(&contract_nft_token_id);
        } else {
            let mut new_nft_contract_seeds = UnorderedSet::new(StorageKeys::AccountSeedId {
                account_seed_id: format!("{}:{}", self.farmer_id, seed_id)
            });
            new_nft_contract_seeds.insert(&contract_nft_token_id);
            self.nft_seeds.insert(seed_id.clone(), new_nft_contract_seeds);
        }
    }

    pub fn sub_nft(&mut self, seed_id: &SeedId, contract_nft_token_id: ContractNFTTokenId ) -> ContractNFTTokenId {
        let mut nft_token_id_exist: bool = false;
        if let Some(nft_contract_seed) = self.nft_seeds.get_mut(seed_id) {
            nft_token_id_exist = nft_contract_seed.remove(&contract_nft_token_id);
        }

        if !nft_token_id_exist {
            env::panic(format!("{}", ERR51_SUB_NFT_IS_NOT_EXIST).as_bytes());
        }

        contract_nft_token_id
    }

    /// Return current balance - locked balanced 
    pub fn get_available_balance(&self, seed_id: &SeedId) -> Balance {
        let balance = self.seeds.get(seed_id).unwrap_or(&0).clone();
        if let Some(locked_seed) = self.get_locked_seed_with_retention_wrapped(seed_id){
            return balance - locked_seed.balance;
        }
        balance
    }

    pub fn get_balance(&self, seed_id: &SeedId) -> Balance {
        self.seeds.get(seed_id).unwrap_or(&0).clone()
    }

    pub fn get_locked_balance(&self, seed_id: &SeedId) -> Balance {
        let default_locked_seed = LockedSeed::default();
        let locked_seed = self.locked_seeds.get(seed_id).unwrap_or(&default_locked_seed);
        locked_seed.balance
    }

    pub fn add_or_create_locked_seed(&mut self, seed_id: &SeedId, balance: Balance, started_at: TimestampSec, ended_at: TimestampSec){
        let mut locked_seed = LockedSeed{
            balance,
            started_at,
            ended_at
        };

        if let Some(current_locked_seed) = self.get_locked_seed_with_retention_wrapped(seed_id){
            locked_seed.balance += current_locked_seed.balance;
        } 

        self.locked_seeds.insert(seed_id.clone(), locked_seed);
    }

    /// get locked seed with retention tolerance
    pub fn get_locked_seed_with_retention_wrapped(&self, seed_id: &SeedId) -> Option<&LockedSeed>{
        let current_block_time = to_sec(env::block_timestamp());
        if let Some(locked_seed) = self.locked_seeds.get(seed_id){
            let ended_at_tolerance = locked_seed.ended_at + LOCKED_SEED_RETENTION;
            if current_block_time > ended_at_tolerance{
                return None;
            }
            return Some(locked_seed);
        }
        None
    }

    pub fn delete_expired_locked_seed(&mut self, seed_id: &SeedId){
        let current_block_time = to_sec(env::block_timestamp());
        if let Some(locked_seed) = self.locked_seeds.get(seed_id){
            let ended_at_tolerance = locked_seed.ended_at + LOCKED_SEED_RETENTION;
            if current_block_time > ended_at_tolerance{
                self.locked_seeds.remove(seed_id);
            }
        }
    }
}


/// Versioned Farmer, used for lazy upgrade.
/// Which means this structure would upgrade automatically when used.
/// To achieve that, each time the new version comes in, 
/// each function of this enum should be carefully re-code!
#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedFarmer {
    V101(FarmerV101),
    V102(Farmer),
}

impl VersionedFarmer {

    pub fn new(farmer_id: AccountId, amount: Balance) -> Self {
        VersionedFarmer::V102(Farmer {
            farmer_id: farmer_id.clone(),
            amount,
            rewards: HashMap::new(),
            seeds: HashMap::new(),
            user_rps: LookupMap::new(StorageKeys::UserRps {
                account_id: farmer_id.clone(),
            }),
            rps_count: 0,
            nft_seeds: HashMap::new(),
            locked_seeds: HashMap::new()
        })
    }

    /// Upgrades from other versions to the currently used version.
    pub fn upgrade(self) -> Self {
        match self {
            VersionedFarmer::V101(farmer_v101) => {
                VersionedFarmer::V102(Farmer::from(farmer_v101))
            },
            VersionedFarmer::V102(farmer) => VersionedFarmer::V102(farmer),
        }
    }

    #[inline]
    #[allow(unreachable_patterns)]
    pub fn need_upgrade(&self) -> bool {
        match self {
            VersionedFarmer::V102(_) => false,
            _ => true,
        }
    }

    #[inline]
    #[allow(unreachable_patterns)]
    pub fn get_ref(&self) -> &Farmer {
        match self {
            VersionedFarmer::V102(farmer) => farmer,
            _ => unimplemented!(),
        }
    }

    #[inline]
    #[allow(unreachable_patterns)]
    pub fn get(self) -> Farmer {
        match self {
            VersionedFarmer::V102(farmer) => farmer,
            _ => unimplemented!(),
        }
    }

    #[inline]
    #[allow(unreachable_patterns)]
    pub fn get_ref_mut(&mut self) -> &mut Farmer {
        match self {
            VersionedFarmer::V102(farmer) => farmer,
            _ => unimplemented!(),
        }
    }
}
