use schemars::JsonSchema;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    env, AccountId};
use crate::models::traits::{
    Storable, StorageError, StorageMetrics,
    Ownable, OwnershipError};

use crate::models::config::{reward::*, storage::*};

pub type RewardId = String;

// === Core State and Action Enums ===
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq, JsonSchema, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
pub enum RewardState {
    Active,
    Completed
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Copy)]
#[serde(crate = "near_sdk::serde")]
pub enum RewardAction {
    Complete,
    Update,
    Delete,
    View,
}

// === Error Hierarchy ===
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum RewardError {
    Validation(RewardValidationError),
    Storage(StorageError),
    Access(OwnershipError),
    State(RewardStateError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum RewardValidationError {
    Title {
        reason: RewardTitleError,
        current_length: usize,
    },
    Description {
        reason: RewardDescriptionError,
        current_length: usize,
    },
    Cost {
        reason: RewardCostError,
        provided_cost: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum RewardTitleError {
    Empty,
    TooLong,
    InvalidCharacters,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum RewardDescriptionError {
    TooLong,
    InvalidCharacters,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum RewardCostError {
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum RewardStateError {
    InvalidTransition { from: RewardState, to: RewardState },
    InvalidActionForState { state: RewardState, action: RewardAction }
}

// === Core Data Structures ===
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct Reward {
    pub id: RewardId,
    pub title: String,
    pub description: String,
    pub cost: u32,
    pub state: RewardState,
    #[schemars(with = "String")]
    owner_id: AccountId,
}

// === Trait Definitions ===
pub trait RewardValidation {
    fn validate_title(&mut self) -> Result<(), RewardValidationError>;
    fn validate_description(&mut self) -> Result<(), RewardValidationError>; 
    fn validate_cost(&mut self) -> Result<(), RewardValidationError>;
    fn validate_state_for_action(&self, action: RewardAction) -> Result<(), RewardStateError>;
}

// === Error Conversions ===
impl From<StorageError> for RewardError {
    fn from(err: StorageError) -> Self {
        RewardError::Storage(err)
    }
}

impl From<OwnershipError> for RewardError {
    fn from(err: OwnershipError) -> Self {
        RewardError::Access(err)
    }
}

impl From<RewardStateError> for RewardError {
    fn from(err: RewardStateError) -> Self {
        RewardError::State(err)
    }
}

impl std::fmt::Display for RewardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(v) => write!(f, "Validation error: {:?}", v),
            Self::Storage(s) => write!(f, "Storage error: {:?}", s),
            Self::Access(a) => write!(f, "Access error: {:?}", a),
            Self::State(s) => write!(f, "State error: {:?}", s),
        }
    }
}

impl std::fmt::Display for RewardValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Title { reason, current_length } => {
                write!(f, "Title error: {:?} (length: {})", reason, current_length)
            },
            Self::Description { reason, current_length } => {
                write!(f, "Description error: {:?} (length: {})", reason, current_length)
            },
            Self::Cost { reason, provided_cost } => {
                write!(f, "Cost error: {:?} (cost: {})", reason, provided_cost)
            }
        }
    }
}

// === Core Implementations ===
impl Reward {
    pub fn new(
        title: String,
        description: String,
        cost: u32,
        owner_id: AccountId,
    ) -> Result<Self, RewardError> {
        let mut reward = Self {
            id: format!("reward-{}-{}", owner_id, env::block_timestamp()),
            title,
            description,
            cost,
            owner_id,
            state: RewardState::Active,
        };

        reward.validate()?;
        Ok(reward)
    }

    pub fn validate(&mut self) -> Result<(), RewardError> {
        self.validate_title()
            .map_err(RewardError::Validation)?;
        self.validate_description()
            .map_err(RewardError::Validation)?;
        self.validate_cost()
            .map_err(RewardError::Validation)?;
        self.validate_storage()
            .map_err(|e| RewardError::Storage(e))?;
        Ok(())
    }

    pub fn transition_to(&mut self, new_state: RewardState) -> Result<(), RewardError> {
        match (&self.state, &new_state) {
            (RewardState::Active, RewardState::Completed) => {
                self.state = new_state;
                Ok(())
            },
            _ => Err(RewardError::State(RewardStateError::InvalidTransition {
                from: self.state.clone(),
                to: new_state,
            })),
        }
    }

    pub fn is_affordable(&self, available_points: u32) -> bool {
        match available_points.checked_sub(self.cost) {
            Some(_) => true,
            None => false
        }
    }
    
}

impl Ownable for Reward {
    fn get_owner_id(&self) -> &AccountId {
        &self.owner_id
    }
}

impl Storable for Reward {
    const BASE_STORAGE: u64 = REWARD_BASE_STORAGE;
    const MAX_STORAGE: u64 = REWARD_MAX_STORAGE;

    fn calculate_storage_metrics(&self) -> StorageMetrics {
        
        let dynamic_size = 
            self.id.len() as u64 +
            self.title.len() as u64 +
            self.description.len() as u64 +
            self.owner_id.to_string().len() as u64;
            
        let total_bytes = Self::BASE_STORAGE + dynamic_size;
        let cost_per_byte = env::storage_byte_cost().as_yoctonear();
        let metrics = StorageMetrics {
            base_size: Self::BASE_STORAGE,
            dynamic_size,
            total_bytes,
            cost_per_byte,
            total_cost: cost_per_byte * total_bytes as u128,
        };
        metrics
    }
}

impl RewardValidation for Reward {
    fn validate_title(&mut self) -> Result<(), RewardValidationError> {
        if self.title.is_empty() {
            return Err(RewardValidationError::Title {
                reason: RewardTitleError::Empty,
                current_length: 0,
            });
        }
        if self.title.len() > MAX_TITLE_LENGTH {
            return Err(RewardValidationError::Title {
                reason: RewardTitleError::TooLong,
                current_length: self.title.len(),
            });
        }
        if self.title.chars().any(|c| {
            let code = c as u32;
            // Allow tab (0x09), line feed (0x0A), and carriage return (0x0D)
            // Prohibit other control characters
            (code <= 0x08) || (code >= 0x0B && code <= 0x0C) || 
            (code >= 0x0E && code <= 0x1F) || (code == 0x7F)
        }) {
            return Err(RewardValidationError::Title {
                reason: RewardTitleError::InvalidCharacters,
                current_length: self.title.len(),
            });
        }
        self.title = self.title.trim().to_string();
        Ok(())
    }

    fn validate_description(&mut self) -> Result<(), RewardValidationError> {
        if self.description.len() > MAX_DESCRIPTION_LENGTH {
            return Err(RewardValidationError::Description {
                reason: RewardDescriptionError::TooLong,
                current_length: self.description.len(),
            });
        }
        
        if self.description.chars().any(|c| {
            let code = c as u32;
            // Allow tab (0x09), line feed (0x0A), and carriage return (0x0D)
            // Prohibit other control characters
            (code <= 0x08) || (code >= 0x0B && code <= 0x0C) || 
            (code >= 0x0E && code <= 0x1F) || (code == 0x7F)
        }) {
            return Err(RewardValidationError::Description {
                reason: RewardDescriptionError::InvalidCharacters,
                current_length: self.description.len(),
            });
        }
        self.description = self.description.trim().to_string();
        Ok(())
    }

    fn validate_cost(&mut self) -> Result<(), RewardValidationError> {
        match self.cost {
            cost if cost == u32::MAX => {
                Err(RewardValidationError::Cost {
                    reason: RewardCostError::Invalid,
                    provided_cost: cost,
                })
            },
            _ => Ok(()),
        }
    }

    fn validate_state_for_action(&self, action: RewardAction) -> Result<(), RewardStateError> {
        match (&self.state, &action) {
            (RewardState::Completed, _) => {
                Err(RewardStateError::InvalidActionForState {
                    state: self.state.clone(),
                    action,
                })
            }
            _ => Ok(()),
        }
    }
}
