use near_sdk::{env, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, 
    Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum StorageError {
    InsufficientBalance { required: u128, available: u128 },
    ExceedsMaxSize { size: u64, max_allowed: u64 }
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientBalance { required, available } => {
                write!(f, "Insufficient balance: required {}, available {}", required, available)
            },
            Self::ExceedsMaxSize { size, max_allowed } => {
                write!(f, "Exceeds max size: size {}, max allowed {}", size, max_allowed)
            }
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, 
    Debug, Clone, PartialEq, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageMetrics {
    pub base_size: u64,
    pub dynamic_size: u64,
    pub total_bytes: u64,
    pub cost_per_byte: u128,
    pub total_cost: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, 
    Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum OwnershipError {
    NotOwner
}

impl std::fmt::Display for OwnershipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotOwner => write!(f, "Operation can only be performed by the owner")
        }
    }
}

pub trait Ownable {
    fn get_owner_id(&self) -> &AccountId;

    fn validate_ownership(&self) -> Result<(), OwnershipError> {
        if env::predecessor_account_id() != *self.get_owner_id() {
            return Err(OwnershipError::NotOwner);
        }
        Ok(())
    }
}

pub trait Storable {
    const BASE_STORAGE: u64;
    const MAX_STORAGE: u64;

    fn calculate_storage_metrics(&self) -> StorageMetrics;
    
    fn validate_storage(&mut self) -> Result<(), StorageError> {
        let metrics = self.calculate_storage_metrics();
        
        if metrics.total_bytes > Self::MAX_STORAGE {
            return Err(StorageError::ExceedsMaxSize {
                size: metrics.total_bytes,
                max_allowed: Self::MAX_STORAGE,
            });
        }
        
        let available = env::account_balance().as_yoctonear();
        if available < metrics.total_cost {
            return Err(StorageError::InsufficientBalance {
                required: metrics.total_cost,
                available,
            });
        }
    
        Ok(())
    }
}
