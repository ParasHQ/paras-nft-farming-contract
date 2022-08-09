use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "standard")]
#[serde(rename_all = "snake_case")]
pub enum NearEvent {
    ParasFarming(ParasFarmingEvent),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParasFarmingEvent {
    pub version: String,
    #[serde(flatten)]
    pub event_kind: ParasFarmingEventKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum ParasFarmingEventKind {
    LockFtBalance(LockFTBalanceData),
    UnlockFtBalance(UnlockFTBalanceData)
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct LockFTBalanceData {
    pub account_id: String,
    pub seed_id: String,
    pub amount: String,
    pub duration: u32,
    pub started_at: u32,
    pub ended_at: u32,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct UnlockFTBalanceData {
    pub account_id: String,
    pub seed_id: String,
    pub amount: String,
}

impl Display for NearEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("EVENT_JSON:{}", self.to_json_string()))
    }
}

impl NearEvent {
    pub fn new(version: String, event_kind: ParasFarmingEventKind) -> Self {
        NearEvent::ParasFarming(ParasFarmingEvent { version, event_kind })
    }
    
    pub fn new_v1(event_kind: ParasFarmingEventKind) -> Self {
        NearEvent::new("1.0.0".to_string(), event_kind)
    }

    pub(crate) fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn log(&self) {
        near_sdk::env::log(&self.to_string().as_bytes());
    }

    pub fn lock_ft_balance(data: LockFTBalanceData) -> Self {
        NearEvent::new_v1(ParasFarmingEventKind::LockFtBalance(data))
    }

    pub fn unlock_ft_balance(data: UnlockFTBalanceData) -> Self {
        NearEvent::new_v1(ParasFarmingEventKind::UnlockFtBalance(data))
    }

    pub fn log_lock_ft_balance(data: LockFTBalanceData){
        NearEvent::lock_ft_balance(data).log();
    }

    pub fn log_unlock_ft_balance(data: UnlockFTBalanceData){
        NearEvent::unlock_ft_balance(data).log();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_ft_balance() {
        let lock_ft_balance_log = LockFTBalanceData{
            account_id: "darmaji".to_string(),
            seed_id: "seed_id_1".to_string(),
            amount: "1".to_string(),
            duration: 1,
            started_at: 1,
            ended_at: 2
        };
        let event_log = NearEvent::lock_ft_balance(lock_ft_balance_log);
    
        assert_eq!(
            serde_json::to_string(&event_log).unwrap(),
            r#"{"standard":"paras_farming","version":"1.0.0","event":"lock_ft_balance","data":{"account_id":"darmaji","seed_id":"seed_id_1","amount":"1","duration":1,"started_at":1,"ended_at":2}}"#
        );
    }

    #[test]
    fn unlock_ft_balance() {
        let unlock_ft_balance_log = UnlockFTBalanceData{
            account_id: "darmaji".to_string(),
            seed_id: "seed_id_1".to_string(),
            amount: "1".to_string(),
        };
        let event_log = NearEvent::unlock_ft_balance(unlock_ft_balance_log);
    
        assert_eq!(
            serde_json::to_string(&event_log).unwrap(),
            r#"{"standard":"paras_farming","version":"1.0.0","event":"unlock_ft_balance","data":{"account_id":"darmaji","seed_id":"seed_id_1","amount":"1"}}"#
        );
    }
}
