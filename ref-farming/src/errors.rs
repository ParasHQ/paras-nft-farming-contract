// Storage errors //
pub const ERR10_ACC_NOT_REGISTERED: &str = "E10: account not registered";
pub const ERR11_INSUFFICIENT_STORAGE: &str = "E11: insufficient $NEAR storage deposit";
pub const ERR12_STORAGE_UNREGISTER_REWARDS_NOT_EMPTY: &str = "E12: still has rewards when unregister";
pub const ERR13_STORAGE_UNREGISTER_SEED_NOT_EMPTY: &str = "E13: still has staked seed when unregister";
pub const ERR14_ACC_ALREADY_REGISTERED: &str = "E14: account already registered";

// Reward errors //
pub const ERR21_TOKEN_NOT_REG: &str = "E21: token not registered";
pub const ERR22_NOT_ENOUGH_TOKENS: &str = "E22: not enough tokens in deposit";

pub const ERR25_CALLBACK_POST_WITHDRAW_INVALID: &str = "E25: expected 1 promise result from withdraw";

// Seed errors //
pub const ERR31_SEED_NOT_EXIST: &str = "E31: seed not exist";
pub const ERR32_NOT_ENOUGH_SEED: &str = "E32: not enough amount of seed";
pub const ERR33_INVALID_SEED_ID: &str = "E33: invalid seed id";
pub const ERR34_BELOW_MIN_SEED_DEPOSITED: &str = "E34: below min_deposit of this seed";
pub const ERR35_ILLEGAL_TOKEN_ID: &str = "E35: illegal token_id in mft_transfer_call";
pub const ERR36_SEED_TYPE_IS_NOT_FT: &str = "E36: seed type is not FT";
pub const ERR37_BALANCE_IS_NOT_ENOUGH: &str = "E37: balance is not enough";
pub const ERR38_END_OF_DURATION_IS_LESS_THAN_ENDED_AT: &str = "E38: end of duration is less than previous ended at";
pub const ERR39_USER_CANNOT_UNLOCK_SEED: &str = "E39: user cannot unlock seed";
pub const ERR40_USER_DOES_NOT_HAVE_LOCKED_SEED: &str = "E40: user does not have locked seed";

// farm errors //
pub const ERR41_FARM_NOT_EXIST: &str = "E41: farm not exist";
pub const ERR42_INVALID_FARM_ID: &str = "E42: invalid farm id";
pub const ERR43_INVALID_FARM_STATUS: &str = "E43: invalid farm status";
pub const ERR44_INVALID_FARM_REWARD: &str = "E44: invalid reward token for this farm";

// nft errors //
pub const ERR51_SUB_NFT_IS_NOT_EXIST: &str = "E51: sub nft is not exist";

pub const ERR500: &str = "E500: Internal ERROR!";
