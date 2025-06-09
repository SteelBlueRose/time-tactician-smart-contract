use schemars::JsonSchema;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    env, AccountId};
use crate::models::traits::{
    Storable, StorageError, StorageMetrics,
    Ownable, OwnershipError};

use crate::models::config::{task::*, time::*, storage::*};

pub type TaskId = String;

// === Core State and Action Enums ===
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, 
    Debug, PartialEq, Clone, Copy, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema,
    Debug, PartialEq, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskState {
    Created,
    InProgress,
    Completed,
    Overdue
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskAction {
    Start,
    Complete,
    Update,
    Delete,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct TaskTimeSlot {
    pub start_time: u64,
    pub end_time: u64,
}

// === Error Hierarchy ===
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskError {
    Validation(TaskValidationError),
    Storage(StorageError),
    Access(OwnershipError),
    State(TaskStateError)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskValidationError {
    Title {
        reason: TitleError,
        current_length: usize,
    },
    Description {
        reason: DescriptionError,
        current_length: usize,
    },
    Deadline {
        reason: DeadlineError,
        provided_time: u64,
    },
    EstimatedTime {
        reason: EstimatedTimeError,
        provided_estimated_time: u32,
    },
    Timing {
        reason: TimingError,
        provided_time: u64,
    },
    Subtasks {
        reason: SubtaskError,
        current_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TitleError {
    Empty,
    TooLong,
    InvalidCharacters,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum DescriptionError {
    TooLong,
    InvalidCharacters,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum DeadlineError {
    PastDeadline,
    TooFarInFuture,
    BeforeEndTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum EstimatedTimeError {
    Zero,
    TooLong,
    MissingEstimatedTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TimingError {
    EndBeforeStart,
    OverlappingSlots
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum SubtaskError {
    DuplicateId,
    CircularDependency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskStateError {
    InvalidTransition { from: TaskState, to: TaskState },
    InvalidActionForState { state: TaskState, action: TaskAction }
}

// === Core Data Structures ===
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub deadline: u64,
    pub estimated_time: u32,
    pub reward_points: u32,
    pub time_slots: Vec<TaskTimeSlot>,
    pub state: TaskState,
    #[schemars(with = "String")]
    owner_id: AccountId,
    pub parent_task_id: Option<TaskId>,
    pub subtask_ids: Vec<TaskId>,
}

// === Trait Definitions ===
pub trait TaskValidation {
    fn validate_title(&mut self) -> Result<(), TaskValidationError>;
    fn validate_description(&mut self) -> Result<(), TaskValidationError>;
    fn validate_deadline(&self) -> Result<(), TaskValidationError>;
    fn validate_estimated_time(&self) -> Result<(), TaskValidationError>;
    fn validate_timing(&self) -> Result<(), TaskValidationError>;
    fn validate_subtasks(&self) -> Result<(), TaskValidationError>;
    fn validate_state_for_action(&self, action: TaskAction) -> Result<(), TaskStateError>;
}

// === Error Conversions ===
impl From<TaskValidationError> for TaskError {
    fn from(err: TaskValidationError) -> Self {
        TaskError::Validation(err)
    }
}

impl From<StorageError> for TaskError {
    fn from(err: StorageError) -> Self {
        TaskError::Storage(err)
    }
}

impl From<OwnershipError> for TaskError {
    fn from(err: OwnershipError) -> Self {
        TaskError::Access(err)
    }
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(v) => write!(f, "Validation error: {:?}", v),
            Self::Storage(s) => write!(f, "Storage error: {:?}", s),
            Self::Access(a) => write!(f, "Access error: {:?}", a),
            Self::State(s) => write!(f, "State error: {:?}", s),
        }
    }
}


impl std::fmt::Display for TaskValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Title { reason, current_length } => {
                write!(f, "Title validation error: {:?} (length: {})", reason, current_length)
            },
            Self::Description { reason, current_length } => {
                write!(f, "Description validation error: {:?} (length: {})", reason, current_length)
            },
            Self::Deadline { reason, provided_time } => {
                write!(f, "Deadline validation error: {:?} (time: {})", reason, provided_time)
            },
            Self::EstimatedTime { reason, provided_estimated_time } => {
                write!(f, "Estimated time validation error: {:?} (time: {})", reason, provided_estimated_time)
            },
            Self::Timing { reason, provided_time } => {
                write!(f, "Timing validation error: {:?} (time: {})", reason, provided_time)
            },
            Self::Subtasks { reason, current_count } => {
                write!(f, "Subtasks validation error: {:?} (count: {})", reason, current_count)
            }
        }
    }
}

// === Core Implementations ===
impl Task {
    pub fn new(
        title: String,
        description: String,
        priority: Priority,
        deadline: u64,
        estimated_time: u32,
        time_slots: Vec<TaskTimeSlot>,
        owner_id: AccountId,
    ) -> Result<Self, TaskError> {
        let mut task = Self {
            id: format!("task-{}-{}", owner_id, env::block_timestamp()),
            title,
            description,
            priority,
            deadline,
            estimated_time,
            reward_points: Self::calculate_reward_points(estimated_time, priority),
            time_slots,
            owner_id,
            state: TaskState::Created,
            parent_task_id: None,
            subtask_ids: Vec::new(),
        };

        task.validate()?;
        Ok(task)
    }

    pub fn calculate_reward_points(estimated_time: u32, priority: Priority) -> u32 {
        let base_points = if estimated_time % 30 >= 15 {
            (estimated_time / 30) + 1
        } else {
            estimated_time / 30
        };
        
        match priority {
            Priority::Low => base_points,
            Priority::Medium => base_points * 2,
            Priority::High => base_points * 3,
            Priority::Critical => base_points * 4,
        }
    }

    pub fn validate(&mut self) -> Result<(), TaskError> {
        self.validate_title()
            .map_err(TaskError::Validation)?;
        self.validate_description()
            .map_err(TaskError::Validation)?;
        self.validate_deadline()
            .map_err(TaskError::Validation)?;
        self.validate_estimated_time()
            .map_err(TaskError::Validation)?;
        self.validate_timing()
            .map_err(TaskError::Validation)?;
        self.validate_subtasks()
            .map_err(TaskError::Validation)?;
        self.validate_storage()
            .map_err(|e| TaskError::Storage(e))?;
        Ok(())
    }

    pub fn transition_to(&mut self, new_state: TaskState) -> Result<(), TaskError> {
        if new_state == TaskState::Completed && self.parent_task_id.is_some() {
            self.state = new_state;
            return Ok(());
        }

        match (&self.state, &new_state) {
            (TaskState::Created, TaskState::InProgress) | 
            (TaskState::InProgress, TaskState::Completed) => {
                self.state = new_state;
                Ok(())
            },
            (TaskState::Created | TaskState::InProgress, TaskState::Overdue) => {
                let current_time = env::block_timestamp();
                if current_time > self.deadline {
                    self.state = new_state;
                    Ok(())
                } else {
                    Err(TaskError::State(TaskStateError::InvalidTransition {
                        from: self.state,
                        to: new_state,
                    }))
                }
            },
            (TaskState::Overdue, TaskState::Completed) => {
                self.state = new_state;
                Ok(())
            },
            _ => Err(TaskError::State(TaskStateError::InvalidTransition {
                from: self.state,
                to: new_state,
            }))
        }
    }

    pub fn add_subtask(&mut self, subtask_id: TaskId) -> Result<(), TaskError> {
        if self.subtask_ids.contains(&subtask_id) {
            return Err(TaskError::Validation(TaskValidationError::Subtasks {
                reason: SubtaskError::DuplicateId,
                current_count: self.subtask_ids.len(),
            }));
        }

        self.subtask_ids.push(subtask_id);
        Ok(())
    }
}

impl TaskValidation for Task {
    fn validate_title(&mut self) -> Result<(), TaskValidationError> {
        if self.title.is_empty() {
            return Err(TaskValidationError::Title {
                reason: TitleError::Empty,
                current_length: 0,
            });
        }
        if self.title.len() > MAX_TITLE_LENGTH {
            return Err(TaskValidationError::Title {
                reason: TitleError::TooLong,
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
            return Err(TaskValidationError::Title {
                reason: TitleError::InvalidCharacters,
                current_length: self.title.len(),
            });
        }
        self.title = self.title.trim().to_string();
        Ok(())
    }

    fn validate_description(&mut self) -> Result<(), TaskValidationError> {
        if self.description.len() > MAX_DESCRIPTION_LENGTH {
            return Err(TaskValidationError::Description {
                reason: DescriptionError::TooLong,
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
            return Err(TaskValidationError::Description {
                reason: DescriptionError::InvalidCharacters,
                current_length: self.description.len(),
            });
        }
        self.description = self.description.trim().to_string();
        Ok(())
    }

    fn validate_deadline(&self) -> Result<(), TaskValidationError> {
        if self.state == TaskState::Overdue {
            return Ok(());
        }
        
        let current_time = env::block_timestamp();
        
        if self.deadline <= current_time {
            return Err(TaskValidationError::Deadline {
                reason: DeadlineError::PastDeadline,
                provided_time: self.deadline,
            });
        }
    
        if self.deadline >= current_time + MAX_FUTURE_TIME {
            return Err(TaskValidationError::Deadline {
                reason: DeadlineError::TooFarInFuture,
                provided_time: self.deadline,
            });
        }
        
        if !self.time_slots.is_empty() {
            let latest_end_time = self.time_slots.iter()
                .map(|slot| slot.end_time)
                .max()
                .unwrap();
                
            if self.deadline <= latest_end_time {
                return Err(TaskValidationError::Deadline {
                    reason: DeadlineError::BeforeEndTime,
                    provided_time: self.deadline,
                });
            }
        }
    
        Ok(())
    }

    fn validate_estimated_time(&self) -> Result<(), TaskValidationError> {
        if self.estimated_time >= MAX_MINUTES {
            return Err(TaskValidationError::EstimatedTime {
                reason: EstimatedTimeError::TooLong,
                provided_estimated_time: self.estimated_time,
            });
        }
        else if self.estimated_time == 0 {
            return Err(TaskValidationError::EstimatedTime {
                reason: EstimatedTimeError::Zero,
                provided_estimated_time: self.estimated_time,
            });
        }
        else {
            Ok(())
        }
    }

    fn validate_timing(&self) -> Result<(), TaskValidationError> {
        // valid (no scheduling)
        if self.time_slots.is_empty() {
            return Ok(());
        }
        
        for slot in &self.time_slots {
            if slot.end_time <= slot.start_time {
                return Err(TaskValidationError::Timing {
                    reason: TimingError::EndBeforeStart,
                    provided_time: slot.end_time,
                });
            }
        }
        
        for i in 0..self.time_slots.len() {
            for j in i+1..self.time_slots.len() {
                if self.time_slots[i].start_time < self.time_slots[j].end_time && 
                   self.time_slots[i].end_time > self.time_slots[j].start_time {
                    return Err(TaskValidationError::Timing {
                        reason: TimingError::OverlappingSlots,
                        provided_time: self.time_slots[j].start_time,
                    });
                }
           
            }
        }
        Ok(())
    }

    fn validate_subtasks(&self) -> Result<(), TaskValidationError> {
        let mut unique_ids = Vec::new();
        for id in &self.subtask_ids {
            if unique_ids.contains(id) {
                return Err(TaskValidationError::Subtasks {
                    reason: SubtaskError::DuplicateId,
                    current_count: self.subtask_ids.len(),
                });
            }
            unique_ids.push(id.clone());
        }

        Ok(())
    }

    fn validate_state_for_action(&self, action: TaskAction) -> Result<(), TaskStateError> {
        match (&self.state, &action) {
            (TaskState::Completed, TaskAction::Update) => {
                Err(TaskStateError::InvalidActionForState {
                    state: self.state,
                    action: action,
                })
            },
            _ => Ok(())
        }
    }
}

impl Ownable for Task {
    fn get_owner_id(&self) -> &AccountId {
        &self.owner_id
    }
}

impl Storable for Task {
    const BASE_STORAGE: u64 = TASK_BASE_STORAGE;
    const MAX_STORAGE: u64 = TASK_MAX_STORAGE;
    
    fn calculate_storage_metrics(&self) -> StorageMetrics {
        
        let dynamic_size = 
            self.id.len() as u64 +
            self.title.len() as u64 +
            self.description.len() as u64 +
            self.owner_id.to_string().len() as u64 +
            self.parent_task_id.as_ref().map_or(0, |id| id.len() as u64) +
            self.subtask_ids.iter().map(|id| id.len() as u64).sum::<u64>();
            
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
