use schemars::JsonSchema;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    env, AccountId};
use crate::models::traits::{
    Storable, StorageError, StorageMetrics, Ownable};
use crate::models::time_slot::{RecurrencePattern, Frequency, DayOfWeek};
use crate::models::task::TaskId;

pub type HabitId = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct Habit {
    pub id: HabitId,
    pub task_id: TaskId,
    pub recurrence: RecurrencePattern,
    pub streak: u32,
    pub last_completed: u64,
    #[schemars(with = "String")]
    owner_id: AccountId,
}

impl Habit {
    pub fn new(
        task_id: TaskId,
        recurrence: RecurrencePattern,
        owner_id: AccountId,
    ) -> Result<Self, StorageError> {
        let mut habit = Self {
            id: format!("habit-{}-{}", owner_id, env::block_timestamp()),
            task_id,
            recurrence,
            owner_id,
            streak: 0,
            last_completed: 0,
        };
        
        habit.validate_storage()?;
        Ok(habit)
    }

    pub fn increment_streak(&mut self) -> u32 {
        self.streak += 1;
        self.last_completed = env::block_timestamp();
        self.streak
    }
    
    pub fn reset_streak(&mut self) {
        self.streak = 0;
        self.last_completed = env::block_timestamp();
    }

    pub fn verify_streak_continuity(&self) -> bool {
        if self.last_completed == 0 {
            return true;
        }
        
        let current_time = env::block_timestamp();
        let time_diff = current_time - self.last_completed;
        
        match &self.recurrence.frequency {
            Frequency::Daily => {
                let interval = self.recurrence.interval.unwrap_or(1);
                let allowed_time = (interval as u64) * 24 * 60 * 60 * 1_000_000_000;
                time_diff <= allowed_time
            },
            Frequency::Custom => {
                if let Some(ref days) = self.recurrence.specific_days {
                    let seconds_per_day = 24 * 60 * 60;
                    let last_completed_days = (self.last_completed / 1_000_000_000) / seconds_per_day;
                    let current_days = (current_time / 1_000_000_000) / seconds_per_day;
                    
                    let current_day_of_week = ((current_days + 3) % 7) as usize;
                    
                    let day_mapping = [
                        DayOfWeek::Monday, DayOfWeek::Tuesday, DayOfWeek::Wednesday,
                        DayOfWeek::Thursday, DayOfWeek::Friday, DayOfWeek::Saturday, DayOfWeek::Sunday
                    ];
                    
                    days.contains(&day_mapping[current_day_of_week]) && (current_days - last_completed_days) <= 7
                } else {
                    false
                }
            }
        }
    }
}

impl Ownable for Habit {
    fn get_owner_id(&self) -> &AccountId {
        &self.owner_id
    }
}

impl Storable for Habit {
    const BASE_STORAGE: u64 = 128;
    const MAX_STORAGE: u64 = 2048;
    
    fn calculate_storage_metrics(&self) -> StorageMetrics {
        let dynamic_size = 
            self.id.len() as u64 +
            self.task_id.len() as u64 +
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