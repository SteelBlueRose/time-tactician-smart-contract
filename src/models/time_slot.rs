use schemars::JsonSchema;
use std::collections::HashSet;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    env, AccountId};
use crate::models::traits::{
    Storable, StorageError, StorageMetrics,
    Ownable, OwnershipError};

use crate::models::config::storage::*;

pub type TimeSlotId = String;

// === Error Hierarchy ===
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TimeSlotError {
    Validation(TimeSlotValidationError),
    Storage(StorageError),
    Access(OwnershipError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TimeSlotValidationError {
    Timing {
        reason: TimeSlotTimingError,
        start_minutes: u32,
        end_minutes: u32,
    },
    Recurrence(TimeSlotRecurrenceError)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TimeSlotTimingError {
    InvalidTimeOfDay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TimeSlotRecurrenceError {
    EmptyDays,
    InvalidPattern
}

// === Core Data Structures ===
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, 
    Clone, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct TimeSlot {
    pub id: TimeSlotId,
    pub start_minutes: u32, // Minutes from midnight (0-1439)
    pub end_minutes: u32,   // Minutes from midnight (0-1439)
    pub duration: Option<u32>,
    pub recurrence: RecurrencePattern,
    #[schemars(with = "String")]
    owner_id: AccountId,
    pub slot_type: SlotType,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, 
    Clone, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct RecurrencePattern {
    pub frequency: Frequency,
    pub interval: Option<u32>,
    pub specific_days: Option<Vec<DayOfWeek>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, 
    PartialEq, Debug, Clone, Copy, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub enum SlotType {
    Break,
    WorkingHours
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, 
    PartialEq, Debug, Clone, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub enum Frequency {
    Daily,
    Custom,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, 
    Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

// === Trait Definitions ===
pub trait TimeSlotValidation {
    fn validate_recurrence(&self) -> Result<(), TimeSlotValidationError>;
}

// === Error Conversions ===
impl From<StorageError> for TimeSlotError {
    fn from(err: StorageError) -> Self {
        TimeSlotError::Storage(err)
    }
}

impl From<OwnershipError> for TimeSlotError {
    fn from(err: OwnershipError) -> Self {
        TimeSlotError::Access(err)
    }
}

impl From<TimeSlotValidationError> for TimeSlotError {
    fn from(err: TimeSlotValidationError) -> Self {
        TimeSlotError::Validation(err)
    }
}

impl std::fmt::Display for TimeSlotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(v) => write!(f, "Validation error: {:?}", v),
            Self::Storage(s) => write!(f, "Storage error: {:?}", s),
            Self::Access(a) => write!(f, "Access error: {:?}", a),
        }
    }
}

impl std::fmt::Display for TimeSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimeSlot {{ duration: {:?}, ... }}", self.duration)
    }
}

impl std::fmt::Display for TimeSlotValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timing { reason, start_minutes, end_minutes } => {
                write!(f, "Timing error: {:?} (start: {}, end: {})", reason, start_minutes, end_minutes)
            },
            Self::Recurrence(reason) => {
                write!(f, "Recurrence error: {:?}", reason)
            }
        }
    }
}

// === Core Implementations ===
impl TimeSlot {    
    pub fn new(
        start_minutes: u32,
        end_minutes: u32,
        recurrence: RecurrencePattern,
        owner_id: AccountId,
    ) -> Result<Self, TimeSlotError> {
        if start_minutes >= 1440 || end_minutes >= 1440 || 
            start_minutes == end_minutes {
            return Err(TimeSlotError::Validation(TimeSlotValidationError::Timing {
                reason: TimeSlotTimingError::InvalidTimeOfDay,
                start_minutes,
                end_minutes,
            }));
        }

        let duration = if end_minutes >= start_minutes {
            end_minutes - start_minutes
        } else {
            // Wraparound case
            (1440 - start_minutes) + end_minutes
        };

        let mut time_slot = Self {
            id: format!("time_slot-{}-{}", owner_id, env::block_timestamp()),
            start_minutes,
            end_minutes,
            duration: Some(duration),
            recurrence,
            owner_id,
            slot_type: SlotType::WorkingHours,
        };
        
        time_slot.validate()?;
        Ok(time_slot)
    }

    pub fn validate(&mut self) -> Result<(), TimeSlotError> {
        self.validate_recurrence()
            .map_err(TimeSlotError::Validation)?;
        self.validate_storage()
            .map_err(|e| TimeSlotError::Storage(e))?;
        Ok(())
    }

    pub fn overlaps_with(&self, other: &TimeSlot) -> bool {
        // If slots are of different types (one is Break and one is WorkingHours),
        // they are allowed to overlap
        if self.slot_type != other.slot_type {
            return false;
        }
        
        if self.start_minutes < self.end_minutes && other.start_minutes < other.end_minutes {
            // Normal case
            return self.start_minutes < other.end_minutes && self.end_minutes > other.start_minutes;

        } else if self.start_minutes >= self.end_minutes && other.start_minutes < other.end_minutes {
            // Self wraps around midnight, other doesn't
            return self.start_minutes < other.end_minutes || self.end_minutes > other.start_minutes;

        } else if self.start_minutes < self.end_minutes && other.start_minutes >= other.end_minutes {
            // Other wraps around midnight, self doesn't
            return other.start_minutes < self.end_minutes || other.end_minutes > self.start_minutes;

        } else {
            // Both wrap around midnight
            return true;
        }
    }
}

impl Ownable for TimeSlot {
    fn get_owner_id(&self) -> &AccountId {
        &self.owner_id
    }
}

impl Storable for TimeSlot {
    const BASE_STORAGE: u64 = TIME_SLOT_BASE_STORAGE;
    const MAX_STORAGE: u64 = TIME_SLOT_MAX_STORAGE;

    fn calculate_storage_metrics(&self) -> StorageMetrics {
        
        let dynamic_size = 
            self.id.len() as u64 +
            self.owner_id.to_string().len() as u64 +
            match &self.recurrence {
                RecurrencePattern { specific_days: Some(days), .. } => {
                    days.len() as u64 * std::mem::size_of::<DayOfWeek>() as u64
                },
                _ => 0,
            };
            
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

impl TimeSlotValidation for TimeSlot {
    fn validate_recurrence(&self) -> Result<(), TimeSlotValidationError> {
        match &self.recurrence {
            RecurrencePattern { frequency: Frequency::Custom, specific_days: None, .. } => {
                return Err(TimeSlotValidationError::Recurrence(TimeSlotRecurrenceError::EmptyDays));
            },
            _ => {}
        }
        Ok(())
    }
}

impl RecurrencePattern {
    pub fn new_daily() -> Self {
        Self {
            frequency: Frequency::Daily,
            interval: Some(1),
            specific_days: None,
        }
    }

    pub fn new_custom(days: Vec<DayOfWeek>) -> Self {
        if days.is_empty() {
            env::panic_str("Must specify at least one day");
        }

        let mut unique_days: Vec<DayOfWeek> = days.into_iter().collect::<HashSet<_>>().into_iter().collect();
        unique_days.sort();

        RecurrencePattern {
            frequency: Frequency::Custom,
            interval: None,
            specific_days: Some(unique_days),
        }
    }

    pub fn is_valid(&self) -> bool {
        match self.frequency {
            Frequency::Custom => {
                // Custom frequency must have specific days and no interval
                self.specific_days.is_some() && 
                !self.specific_days.as_ref().unwrap().is_empty() && 
                self.interval.is_none()
            }
            Frequency::Daily => {
                // Daily frequency must have an interval > 0 and no specific days
                self.interval.is_some() && 
                self.interval.unwrap() > 0 && 
                self.specific_days.is_none()
            }
        }
    }
}