use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use near_sdk::{
    near, env, PanicOnDefault, AccountId,
    collections::{LookupMap, UnorderedMap, UnorderedSet},
};

pub mod models;
use crate::models::{
    Task, TaskId, Priority, TaskState, TaskTimeSlot,
    TaskError, TaskValidationError, TaskStateError,

    Habit, HabitId,

    Reward, RewardId, RewardState, 
    RewardError, RewardValidationError, RewardStateError,

    TimeSlot, TimeSlotId, SlotType, RecurrencePattern,
    Frequency, DayOfWeek, TimeSlotError, TimeSlotValidationError,

    StorageError, OwnershipError, Ownable,
};

// === Core Enums ===
#[derive(Debug)]
pub enum IndexType {
    Task,
    Habit,
    Reward,
    TimeSlot,
}

// === Return Types ===
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Response<T, E> {
    Success(T),
    Error(E)
}

impl<T, E> Response<T, E> {
    pub fn map_err<F, E2>(self, f: F) -> Response<T, E2>
    where
        F: FnOnce(E) -> E2,
    {
        match self {
            Response::Success(t) => Response::Success(t),
            Response::Error(e) => Response::Error(f(e)),
        }
    }

    pub fn from_result<E2>(result: Result<T, E2>, error_mapper: impl FnOnce(E2) -> E) -> Self {
        match result {
            Ok(t) => Response::Success(t),
            Err(e) => Response::Error(error_mapper(e)),
        }
    }
}

// === Core Error Types ===
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractError {
    ValidationError(String, String, Option<String>), // entity, message, details
    StorageError(StorageError),
    AccessError(OwnershipError), 
    StateError(String, String, String, String), // entity, current_state, attempted_action, message
    NotFound(String, String), // entity, id
    Operation(String) // error message
}

// === Type aliases for response types ===
pub type TaskResponse = Response<Task, ContractError>;
pub type TaskListResponse = Response<Vec<Task>, ContractError>;
pub type TaskActionResponse = Response<TaskId, ContractError>;

pub type HabitListResponse = Response<Vec<Habit>, ContractError>;

pub type RewardResponse = Response<Reward, ContractError>;
pub type RewardActionResponse = Response<RewardId, ContractError>;
pub type RewardListResponse = Response<Vec<Reward>, ContractError>;

pub type TimeSlotResponse = Response<TimeSlot, ContractError>;
pub type TimeSlotListResponse = Response<Vec<TimeSlot>, ContractError>;
pub type TimeSlotActionResponse = Response<TimeSlotId, ContractError>;

pub type PointsResponse = Response<u32, ContractError>;

// === Error Conversion Implementations ===
impl From<StorageError> for ContractError {
    fn from(err: StorageError) -> Self {
        ContractError::StorageError(err)
    }
}

impl From<OwnershipError> for ContractError {
    fn from(err: OwnershipError) -> Self {
        ContractError::AccessError(err)
    }
}

// Task error conversions
impl From<TaskError> for ContractError {
    fn from(err: TaskError) -> Self {
        match err {
            TaskError::Validation(err) => ContractError::ValidationError(
                "Task".to_string(),
                err.to_string(),
                None
            ),
            TaskError::Storage(err) => ContractError::StorageError(err),
            TaskError::Access(err) => ContractError::AccessError(err),
            TaskError::State(err) => err.into()
        }
    }
}

impl From<TaskValidationError> for ContractError {
    fn from(err: TaskValidationError) -> Self {
        ContractError::ValidationError(
            "Task".to_string(),
            err.to_string(),
            None
        )
    }
}

impl From<TaskStateError> for ContractError {
    fn from(err: TaskStateError) -> Self {
        match err {
            TaskStateError::InvalidTransition { from, to } => ContractError::StateError(
                "Task".to_string(),
                format!("{:?}", from),
                format!("transition to {:?}", to),
                "Invalid state transition".to_string()
            ),
            TaskStateError::InvalidActionForState { state, action } => ContractError::StateError(
                "Task".to_string(),
                format!("{:?}", state),
                format!("{:?}", action),
                "Invalid action for current state".to_string()
            )
        }
    }
}

// Reward error conversions
impl From<RewardError> for ContractError {
    fn from(err: RewardError) -> Self {
        match err {
            RewardError::Validation(err) => ContractError::ValidationError(
                "Reward".to_string(),
                err.to_string(),
                None
            ),
            RewardError::Storage(err) => ContractError::StorageError(err),
            RewardError::Access(err) => ContractError::AccessError(err),
            RewardError::State(err) => err.into()
        }
    }
}

impl From<RewardValidationError> for ContractError {
    fn from(err: RewardValidationError) -> Self {
        ContractError::ValidationError(
            "Reward".to_string(),
            err.to_string(),
            None
        )
    }
}

impl From<RewardStateError> for ContractError {
    fn from(err: RewardStateError) -> Self {
        match err {
            RewardStateError::InvalidTransition { from, to } => ContractError::StateError(
                "Reward".to_string(),
                format!("{:?}", from),
                format!("transition to {:?}", to),
                "Invalid state transition".to_string()
            ),
            RewardStateError::InvalidActionForState { state, action } => ContractError::StateError(
                "Reward".to_string(),
                format!("{:?}", state),
                format!("{:?}", action),
                "Invalid action for current state".to_string()
            )
        }
    }
}

// TimeSlot error conversions
impl From<TimeSlotError> for ContractError {
    fn from(err: TimeSlotError) -> Self {
        match err {
            TimeSlotError::Validation(err) => ContractError::ValidationError(
                "TimeSlot".to_string(),
                err.to_string(),
                None
            ),
            TimeSlotError::Storage(err) => ContractError::StorageError(err),
            TimeSlotError::Access(err) => ContractError::AccessError(err),
        }
    }
}

impl From<TimeSlotValidationError> for ContractError {
    fn from(err: TimeSlotValidationError) -> Self {
        ContractError::ValidationError(
            "TimeSlot".to_string(),
            err.to_string(),
            None
        )
    }
}

// === Error Display Implementations ===
impl std::fmt::Display for ContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValidationError(entity, message, details) => {
                if let Some(detail) = details {
                    write!(f, "{} validation error: {} ({})", entity, message, detail)
                } else {
                    write!(f, "{} validation error: {}", entity, message)
                }
            },
            Self::StorageError(err) => write!(f, "Storage error: {}", err),
            Self::AccessError(err) => write!(f, "Access error: {}", err),
            Self::StateError(entity, current_state, attempted_action, message) => {
                write!(f, "{} state error: {} (current state: {}, attempted: {})", 
                    entity, message, current_state, attempted_action)
            },
            Self::NotFound(entity, id) => write!(f, "{} not found: {}", entity, id),
            Self::Operation(err) => write!(f, "Operation error: {}", err)
        }
    }
}

// === Core Data Structures ===
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    tasks: UnorderedMap<TaskId, Task>,
    tasks_per_owner: LookupMap<AccountId, UnorderedSet<TaskId>>,
    habits: UnorderedMap<HabitId, Habit>,
    habits_per_owner: LookupMap<AccountId, UnorderedSet<HabitId>>,
    task_completions: LookupMap<TaskId, Vec<u64>>,
    reward_points: LookupMap<AccountId, u32>,
    rewards: UnorderedMap<RewardId, Reward>,
    rewards_per_owner: LookupMap<AccountId, UnorderedSet<RewardId>>,
    time_slots: UnorderedMap<TimeSlotId, TimeSlot>,
    time_slots_per_owner: LookupMap<AccountId, UnorderedSet<TimeSlotId>>,
}  


#[near]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            tasks: UnorderedMap::new(b"t".to_vec()),
            tasks_per_owner: LookupMap::new(b"to".to_vec()),
            habits: UnorderedMap::new(b"h".to_vec()),
            habits_per_owner: LookupMap::new(b"ho".to_vec()),
            task_completions: LookupMap::new(b"tc".to_vec()),
            reward_points: LookupMap::new(b"rp".to_vec()),
            rewards: UnorderedMap::new(b"r".to_vec()),
            rewards_per_owner: LookupMap::new(b"ro".to_vec()),
            time_slots: UnorderedMap::new(b"ts".to_vec()),
            time_slots_per_owner: LookupMap::new(b"tso".to_vec()),
        }
    }

    fn add_to_owner_index(&mut self, owner_id: &AccountId, id: &str, index_type: IndexType) {
        match index_type {
            IndexType::Task => {
                let mut task_set = self.tasks_per_owner
                    .get(owner_id)
                    .unwrap_or_else(|| UnorderedSet::new(format!("to{}", owner_id).as_bytes()));
                task_set.insert(&id.to_string());
                self.tasks_per_owner.insert(owner_id, &task_set);
            },
            IndexType::Habit => {
                let mut habit_set = self.habits_per_owner
                    .get(owner_id)
                    .unwrap_or_else(|| UnorderedSet::new(format!("ho{}", owner_id).as_bytes()));
                habit_set.insert(&id.to_string());
                self.habits_per_owner.insert(owner_id, &habit_set);
            },
            IndexType::Reward => {
                let mut reward_set = self.rewards_per_owner
                    .get(owner_id)
                    .unwrap_or_else(|| UnorderedSet::new(format!("ro{}", owner_id).as_bytes()));
                reward_set.insert(&id.to_string());
                self.rewards_per_owner.insert(owner_id, &reward_set);
            },
            IndexType::TimeSlot => {
                let mut slot_set = self.time_slots_per_owner
                    .get(owner_id)
                    .unwrap_or_else(|| UnorderedSet::new(format!("tso{}", owner_id).as_bytes()));
                slot_set.insert(&id.to_string());
                self.time_slots_per_owner.insert(owner_id, &slot_set);
            },
        }
    }

    fn remove_from_owner_index(&mut self, owner_id: &AccountId, id: &str, index_type: IndexType) {
        match index_type {
            IndexType::Task => {
                if let Some(mut task_set) = self.tasks_per_owner.get(owner_id) {
                    task_set.remove(&id.to_string());
                    self.tasks_per_owner.insert(owner_id, &task_set);
                }
            },
            IndexType::Habit => {
                if let Some(mut habit_set) = self.habits_per_owner.get(owner_id) {
                    habit_set.remove(&id.to_string());
                    self.habits_per_owner.insert(owner_id, &habit_set);
                }
            },
            IndexType::Reward => {
                if let Some(mut reward_set) = self.rewards_per_owner.get(owner_id) {
                    reward_set.remove(&id.to_string());
                    self.rewards_per_owner.insert(owner_id, &reward_set);
                }
            },
            IndexType::TimeSlot => {
                if let Some(mut slot_set) = self.time_slots_per_owner.get(owner_id) {
                    slot_set.remove(&id.to_string());
                    self.time_slots_per_owner.insert(owner_id, &slot_set);
                }
            },
        }
    }

    // === Reward points management ===
    pub fn get_reward_points(&self, account_id: &AccountId) -> PointsResponse {
        let owner_id = env::predecessor_account_id();
        if parent_task.get_owner_id() != &owner_id {
            return Response::Error(ContractError::AccessError(OwnershipError::NotOwner));
        }

        if account_id.to_string().is_empty() {
            return Response::Error(ContractError::ValidationError(
                "Account".to_string(),
                "Account ID cannot be empty".to_string(),
                None
            ));
        }

        match self.reward_points.get(account_id) {
            Some(points) => Response::Success(points),
            None => Response::Success(0)
        }
    }

    fn add_reward_points(&mut self, account_id: AccountId, points: u32) -> PointsResponse {
        if account_id.to_string().is_empty() {
            return Response::Error(ContractError::ValidationError(
                "Account".to_string(),
                "Account ID cannot be empty".to_string(),
                None
            ));
        }

        let current_points = match self.get_reward_points(&account_id) {
            Response::Success(points) => points,
            Response::Error(err) => return Response::Error(err)
        };

        if points > 0 {
            match current_points.checked_add(points) {
                Some(new_points) => {
                    self.reward_points.insert(&account_id, &new_points);
                    Response::Success(new_points)
                },
                None => Response::Error(ContractError::Operation("Points addition would overflow".to_string()))
            }
        } else {
            let points_to_subtract = points.wrapping_neg();
            if current_points < points_to_subtract {
                return Response::Error(ContractError::Operation(
                    format!("Insufficient points: has {}, needs {}", current_points, points_to_subtract)
                ));
            }
            let new_points = current_points - points_to_subtract;
            self.reward_points.insert(&account_id, &new_points);
            Response::Success(new_points)
        }
    }

    // === Task Management === 
    pub fn get_tasks_by_owner(&self, owner_id: AccountId) -> TaskListResponse {
        let task_set = match self.tasks_per_owner.get(&owner_id) {
            Some(set) => set,
            None => return Response::Error(ContractError::NotFound(
                "Task".to_string(),
                format!("No tasks found for {}", owner_id)
            ))
        };

        let tasks: Vec<Task> = task_set
            .iter()
            .filter_map(|task_id| self.tasks.get(&task_id))
            .collect();

        if tasks.is_empty() {
            return Response::Error(ContractError::NotFound(
                "Task".to_string(),
                format!("No tasks found for {}", owner_id)
            ));
        }

        Response::Success(tasks)
    }

    pub fn get_incomplete_tasks(&self, owner_id: AccountId) -> TaskListResponse {
        let all_tasks = match self.get_tasks_by_owner(owner_id.clone()) {
            Response::Success(tasks) => tasks,
            Response::Error(err) => return Response::Error(err),
        };

        let incomplete_tasks: Vec<Task> = all_tasks
            .into_iter()
            .filter(|task| task.state != TaskState::Completed)
            .collect();

        if incomplete_tasks.is_empty() {
            return Response::Error(ContractError::NotFound(
                "Task".to_string(),
                format!("No incomplete tasks found for {}", owner_id)
            ));
        }

        Response::Success(incomplete_tasks)
    }

    pub fn get_completed_tasks(&self, owner_id: AccountId) -> TaskListResponse {
        let all_tasks = match self.get_tasks_by_owner(owner_id.clone()) {
            Response::Success(tasks) => tasks,
            Response::Error(err) => return Response::Error(err),
        };

        let completed_tasks: Vec<Task> = all_tasks
            .into_iter()
            .filter(|task| task.state == TaskState::Completed)
            .collect();

        if completed_tasks.is_empty() {
            return Response::Error(ContractError::NotFound(
                "Task".to_string(),
                format!("No completed tasks found for {}", owner_id)
            ));
        }

        Response::Success(completed_tasks)
    }

    pub fn add_task(
        &mut self,
        title: String,
        description: String,
        priority: Priority,
        deadline: u64,
        estimated_time: u32,
        time_slots: Option<Vec<TaskTimeSlot>>,
        parent_task_id: Option<TaskId>,
        recurrence_pattern: Option<RecurrencePattern>,
    ) -> TaskActionResponse {
        let owner_id = env::predecessor_account_id();
        
        if let Some(ref parent_id) = parent_task_id {
            let parent_task = match self.tasks.get(parent_id) {
                Some(task) => task,
                None => return Response::Error(ContractError::NotFound(
                    "Parent Task".to_string(),
                    format!("Parent task {} not found", parent_id)
                ))
            };
    
            if parent_task.get_owner_id() != &owner_id {
                return Response::Error(ContractError::AccessError(OwnershipError::NotOwner));
            }
        }
    
        let mut task = match Task::new(
            title,
            description,
            priority,
            deadline,
            estimated_time,
            time_slots.unwrap_or_default(),
            owner_id.clone()
        ) {
            Ok(task) => task,
            Err(e) => return Response::Error(e.into())
        };
    
        if let Some(ref parent_id) = parent_task_id {
            task.parent_task_id = Some(parent_id.clone());
        }
    
        let task_id = task.id.clone();
        self.tasks.insert(&task_id, &task);
        self.add_to_owner_index(&owner_id, &task_id, IndexType::Task);
        
        if let Some(recurrence) = recurrence_pattern {
            match Habit::new(task_id.clone(), recurrence, owner_id.clone()) {
                Ok(habit) => {
                    let habit_id = habit.id.clone();
                    self.habits.insert(&habit_id, &habit);
                    self.add_to_owner_index(&owner_id, &habit_id, IndexType::Habit);
                },
                Err(e) => return Response::Error(e.into())
            }
        }
        
        if let Some(parent_id) = parent_task_id {
            let mut parent_task = match self.tasks.get(&parent_id) {
                Some(task) => task,
                None => return Response::Error(ContractError::NotFound(
                    "Parent Task".to_string(),
                    format!("Parent task {} not found", parent_id)
                ))
            };
    
            match parent_task.add_subtask(task_id.clone()) {
                Ok(_) => {
                    self.tasks.insert(&parent_id, &parent_task);
                    Response::Success(task_id)
                },
                Err(e) => Response::Error(e.into())
            }
        } else {
            Response::Success(task_id)
        }
    }

    pub fn update_task(
        &mut self,
        task_id: TaskId,
        title: String,
        description: String,
        priority: Priority,
        deadline: u64,
        estimated_time: u32,
        time_slots: Option<Vec<TaskTimeSlot>>,
    ) -> TaskActionResponse {
        let mut task = match self.tasks.get(&task_id) {
            Some(t) => t,
            None => return Response::Error(ContractError::NotFound(
                "Task".to_string(),
                format!("Task {} not found", task_id)
            ))
        };
    
        if let Err(e) = task.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
    
        task.title = title;
        task.description = description;
        task.priority = priority;
        task.deadline = deadline;
        task.estimated_time = estimated_time;
        if let Some(slots) = time_slots {
            task.time_slots = slots;
        }
    
        task.reward_points = Task::calculate_reward_points(estimated_time, priority);
    
        if let Err(e) = task.validate() {
            return Response::Error(e.into());
        }
    
        self.tasks.insert(&task_id, &task);
        Response::Success(task_id)
    }

    pub fn complete_task(&mut self, task_id: TaskId) -> TaskActionResponse {
        let mut task = match self.tasks.get(&task_id) {
            Some(t) => t,
            None => return Response::Error(ContractError::NotFound(
                "Task".to_string(),
                format!("Task {} not found", task_id)
            ))
        };
    
        if let Err(e) = task.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
    
        for subtask_id in &task.subtask_ids {
            let mut subtask = match self.tasks.get(subtask_id) {
                Some(t) => t,
                None => return Response::Error(ContractError::NotFound(
                    "Subtask".to_string(),
                    format!("Subtask {} not found", subtask_id)
                ))
            };
    
            if let Err(e) = subtask.transition_to(TaskState::Completed) {
                return Response::Error(e.into());
            }
    
            match self.add_reward_points(subtask.get_owner_id().clone(), subtask.reward_points) {
                Response::Success(_) => (),
                Response::Error(e) => return Response::Error(e)
            }
    
            self.tasks.insert(subtask_id, &subtask);
        }
    
        if let Err(e) = task.transition_to(TaskState::Completed) {
            return Response::Error(e.into());
        }
        
        task.time_slots.clear();
        
        let current_time = env::block_timestamp();
        let mut completions = self.task_completions.get(&task_id).unwrap_or_default();
        completions.push(current_time);
        self.task_completions.insert(&task_id, &completions);
    
        let habit_id_option = self.habits.iter()
            .find(|(_, habit)| habit.task_id == task_id)
            .map(|(id, _)| id.clone());
            
        if let Some(habit_id) = habit_id_option {
            let mut habit = self.habits.get(&habit_id).unwrap();
            
            if habit.verify_streak_continuity() {
                habit.increment_streak();
            } else {
                habit.reset_streak();
            }
            
            let new_deadline = match &habit.recurrence.frequency {
                Frequency::Daily => {
                    let interval = habit.recurrence.interval.unwrap_or(1);
                    current_time + (interval as u64) * 24 * 60 * 60 * 1_000_000_000
                },
                Frequency::Custom => {
                    if let Some(ref days) = habit.recurrence.specific_days {
                        let seconds_per_day = 24 * 60 * 60;
                        let current_days = (current_time / 1_000_000_000) / seconds_per_day;
                        let current_day_of_week = ((current_days + 3) % 7) as usize;
                        
                        let day_mapping = [
                            DayOfWeek::Monday, DayOfWeek::Tuesday, DayOfWeek::Wednesday,
                            DayOfWeek::Thursday, DayOfWeek::Friday, DayOfWeek::Saturday, DayOfWeek::Sunday
                        ];
                        
                        let mut days_until_next = 7;
                        for day_offset in 1..=7 {
                            let next_day_idx = (current_day_of_week + day_offset) % 7;
                            let next_day = day_mapping[next_day_idx].clone();
                            if days.contains(&next_day) {
                                days_until_next = day_offset;
                                break;
                            }
                        }
                        
                        current_time + (days_until_next as u64) * 24 * 60 * 60 * 1_000_000_000
                    } else {
                        current_time + 7 * 24 * 60 * 60 * 1_000_000_000
                    }
                }
            };
            
            task.state = TaskState::Created;
            task.deadline = new_deadline;
            task.time_slots.clear();
            
            habit.task_id = task.id.clone();
            self.habits.insert(&habit_id, &habit);
        }
    
        match self.add_reward_points(task.get_owner_id().clone(), task.reward_points) {
            Response::Success(_) => (),
            Response::Error(e) => return Response::Error(e)
        }
    
        self.tasks.insert(&task_id, &task);
        Response::Success(task_id)
    }

    pub fn mark_task_overdue(&mut self, task_id: TaskId) -> TaskActionResponse {
        let mut task = match self.tasks.get(&task_id) {
            Some(t) => t,
            None => return Response::Error(ContractError::NotFound(
                "Task".to_string(),
                format!("Task {} not found", task_id)
            ))
        };
    
        if let Err(e) = task.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
        
        let current_time = env::block_timestamp();
        if current_time <= task.deadline {
            return Response::Error(ContractError::ValidationError(
                "Task".to_string(),
                "Cannot mark as overdue before deadline".to_string(),
                None
            ));
        }
        
        if let Err(e) = task.transition_to(TaskState::Overdue) {
            return Response::Error(e.into());
        }
        
        task.time_slots.clear();
        
        self.tasks.insert(&task_id, &task);
        Response::Success(task_id)
    }

    pub fn delete_task(&mut self, task_id: TaskId) -> TaskActionResponse {
        let task = match self.tasks.get(&task_id) {
            Some(t) => t,
            None => return Response::Error(ContractError::NotFound(
                "Task".to_string(),
                format!("Task {} not found", task_id)
            ))
        };
    
        if let Err(e) = task.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
    
        for subtask_id in &task.subtask_ids {
            if let Some(subtask) = self.tasks.get(subtask_id) {
                self.tasks.remove(subtask_id);
                self.remove_from_owner_index(
                    subtask.get_owner_id(),
                    subtask_id,
                    IndexType::Task
                );
            }
        }
    
        self.tasks.remove(&task_id);
        self.remove_from_owner_index(
            task.get_owner_id(),
            &task_id,
            IndexType::Task
        );
    
        Response::Success(task_id)
    }
    
    pub fn start_task(&mut self, task_id: TaskId, scheduled_start_time: u64) -> TaskActionResponse {
        let mut task = match self.tasks.get(&task_id) {
            Some(t) => t,
            None => return Response::Error(ContractError::NotFound(
                "Task".to_string(),
                format!("Task {} not found", task_id)
            ))
        };
    
        if let Err(e) = task.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
        
        let estimated_time_ns = task.estimated_time as u64 * 60 * 1_000_000_000;
        let scheduled_end_time = scheduled_start_time + estimated_time_ns;
        
        task.time_slots.push(TaskTimeSlot {
            start_time: scheduled_start_time,
            end_time: scheduled_end_time,
        });
        
        if let Err(e) = task.validate() {
            return Response::Error(e.into());
        }
        
        if let Err(e) = task.transition_to(TaskState::InProgress) {
            return Response::Error(e.into());
        }
    
        self.tasks.insert(&task_id, &task);
        Response::Success(task_id)
    }

    pub fn split_task(&mut self, task_id: TaskId, split_times: Vec<u64>) -> TaskActionResponse {
        if split_times.is_empty() || split_times.len() == 1 {
            return Response::Error(ContractError::ValidationError(
                "Task".to_string(),
                "Need at least two split points to split a task".to_string(),
                None
            ));
        }
    
        let mut task = match self.tasks.get(&task_id) {
            Some(t) => t,
            None => return Response::Error(ContractError::NotFound(
                "Task".to_string(),
                format!("Task {} not found", task_id)
            ))
        };
    
        if let Err(e) = task.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
        
        // Clear existing time slots
        task.time_slots.clear();
        
        // Ensure the split times are sorted
        let mut ordered_times = split_times.clone();
        ordered_times.sort();
        
        // Create time slots from split points
        for i in 0..ordered_times.len() - 1 {
            task.time_slots.push(TaskTimeSlot {
                start_time: ordered_times[i],
                end_time: ordered_times[i + 1],
            });
        }
        
        if let Err(e) = task.validate() {
            return Response::Error(e.into());
        }
        
        if task.state == TaskState::Created {
            if let Err(e) = task.transition_to(TaskState::InProgress) {
                return Response::Error(e.into());
            }
        }
        
        self.tasks.insert(&task_id, &task);
        Response::Success(task_id)
    }

    // === Habit Management ===
    pub fn get_habits_by_owner(&self, owner_id: AccountId) -> HabitListResponse {
        let habit_set = match self.habits_per_owner.get(&owner_id) {
            Some(set) => set,
            None => return Response::Error(ContractError::NotFound(
                "Habit".to_string(),
                format!("No habits found for {}", owner_id)
            ))
        };
        
        let habits: Vec<Habit> = habit_set
            .iter()
            .filter_map(|habit_id| self.habits.get(&habit_id))
            .collect();
        
        if habits.is_empty() {
            return Response::Error(ContractError::NotFound(
                "Habit".to_string(),
                format!("No habits found for {}", owner_id)
            ));
        }
        
        Response::Success(habits)
    }
    
    pub fn get_habit_streak(&self, habit_id: HabitId) -> Response<u32, ContractError> {
        let habit = match self.habits.get(&habit_id) {
            Some(h) => h,
            None => return Response::Error(ContractError::NotFound(
                "Habit".to_string(),
                format!("Habit {} not found", habit_id)
            ))
        };
        
        if let Err(e) = habit.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
        
        Response::Success(habit.streak)
    }
    
    pub fn get_task_completion_history(&self, task_id: TaskId) -> Response<Vec<u64>, ContractError> {
        let task = match self.tasks.get(&task_id) {
            Some(t) => t,
            None => return Response::Error(ContractError::NotFound(
                "Task".to_string(),
                format!("Task {} not found", task_id)
            ))
        };
        
        if let Err(e) = task.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
        
        let completions = self.task_completions.get(&task_id).unwrap_or_default();
        Response::Success(completions)
    }

    // === Reward Management ===
    pub fn get_rewards_by_owner(&self, owner_id: AccountId) -> RewardListResponse {
        let reward_set = match self.rewards_per_owner.get(&owner_id) {
            Some(set) => set,
            None => return Response::Error(ContractError::NotFound(
                "Reward".to_string(),
                format!("No rewards found for {}", owner_id)
            ))
        };
    
        let rewards: Vec<Reward> = reward_set
            .iter()
            .filter_map(|reward_id| self.rewards.get(&reward_id))
            .filter(|reward| reward.state == RewardState::Active)
            .collect();
    
        if rewards.is_empty() {
            return Response::Error(ContractError::NotFound(
                "Reward".to_string(),
                format!("No active rewards found for {}", owner_id)
            ));
        }
    
        Response::Success(rewards)
    }

    pub fn get_retrieved_rewards(&self, owner_id: AccountId) -> RewardListResponse {
        let reward_set = match self.rewards_per_owner.get(&owner_id) {
            Some(set) => set,
            None => return Response::Error(ContractError::NotFound(
                "Reward".to_string(),
                format!("No rewards found for {}", owner_id)
            ))
        };
    
        let rewards: Vec<Reward> = reward_set
            .iter()
            .filter_map(|reward_id| self.rewards.get(&reward_id))
            .filter(|reward| reward.state == RewardState::Completed)
            .collect();
    
        if rewards.is_empty() {
            return Response::Error(ContractError::NotFound(
                "Reward".to_string(),
                format!("No completed rewards found for {}", owner_id)
            ));
        }
    
        Response::Success(rewards)
    }

    pub fn add_reward(&mut self, title: String, description: String, cost: u32) -> RewardActionResponse {
        let owner_id = env::predecessor_account_id();
    
        let reward = match Reward::new(title, description, cost, owner_id.clone()) {
            Ok(r) => r,
            Err(e) => return Response::Error(e.into())
        };
    
        let reward_id = reward.id.clone();
        self.rewards.insert(&reward_id, &reward);
        self.add_to_owner_index(&owner_id, &reward_id, IndexType::Reward);
    
        Response::Success(reward_id)
    }
    
    pub fn update_reward(&mut self, reward_id: RewardId, title: String, description: String, cost: u32) -> RewardActionResponse {
        let mut reward = match self.rewards.get(&reward_id) {
            Some(r) => r,
            None => return Response::Error(ContractError::NotFound(
                "Reward".to_string(),
                format!("Reward {} not found", reward_id)
            ))
        };
    
        if let Err(e) = reward.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
    
        reward.title = title;
        reward.description = description;
        reward.cost = cost;
    
        if let Err(e) = reward.validate() {
            return Response::Error(e.into());
        }
    
        self.rewards.insert(&reward_id, &reward);
        Response::Success(reward_id)
    }
    
    pub fn delete_reward(&mut self, reward_id: RewardId) -> RewardActionResponse {
        let reward = match self.rewards.get(&reward_id) {
            Some(r) => r,
            None => return Response::Error(ContractError::NotFound(
                "Reward".to_string(),
                format!("Reward {} not found", reward_id)
            ))
        };
    
        if let Err(e) = reward.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
    
        self.rewards.remove(&reward_id);
        self.remove_from_owner_index(reward.get_owner_id(), &reward_id, IndexType::Reward);
    
        Response::Success(reward_id)
    }
    
    pub fn redeem_reward(&mut self, reward_id: RewardId) -> RewardActionResponse {
        let reward = match self.rewards.get(&reward_id) {
            Some(r) => r,
            None => return Response::Error(ContractError::NotFound(
                "Reward".to_string(),
                format!("Reward {} not found", reward_id)
            ))
        };
    
        if let Err(e) = reward.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
    
        let available_points = match self.get_reward_points(reward.get_owner_id()) {
            Response::Success(points) => points,
            Response::Error(e) => return Response::Error(e)
        };
    
        if available_points < reward.cost {
            return Response::Error(ContractError::StateError(
                "Reward".to_string(),
                format!("available: {}", available_points),
                format!("required: {}", reward.cost),
                "Insufficient points for redemption".to_string()
            ));
        }
    
        let new_points = available_points - reward.cost;
        self.reward_points.insert(reward.get_owner_id(), &new_points);
    
        let mut updated_reward = reward.clone();
        if let Err(e) = updated_reward.transition_to(RewardState::Completed) {
            return Response::Error(e.into());
        }
    
        self.rewards.insert(&reward_id, &updated_reward);
        Response::Success(reward_id)
    }
    
    // === Time Slot Management ===
    pub fn get_time_slots_by_owner(&self, owner_id: AccountId) -> TimeSlotListResponse {
        let slot_set = match self.time_slots_per_owner.get(&owner_id) {
            Some(set) => set,
            None => return Response::Error(ContractError::NotFound(
                "TimeSlot".to_string(),
                format!("No time slots found for {}", owner_id)
            ))
        };
    
        let slots: Vec<TimeSlot> = slot_set
            .iter()
            .filter_map(|slot_id| self.time_slots.get(&slot_id))
            .collect();
    
        if slots.is_empty() {
            return Response::Error(ContractError::NotFound(
                "TimeSlot".to_string(),
                format!("No time slots found for {}", owner_id)
            ));
        }
    
        Response::Success(slots)
    }
    
    pub fn get_time_slots_by_timeframe(
        &self, 
        owner_id: AccountId,
        start_minutes: u32,
        end_minutes: u32,
        slot_type: Option<SlotType>
    ) -> TimeSlotListResponse {
        let slot_set = match self.time_slots_per_owner.get(&owner_id) {
            Some(s) => s,
            None => return Response::Error(ContractError::NotFound(
                "TimeSlot".to_string(),
                format!("No time slots found for {}", owner_id)
            ))
        };
    
        let slots: Vec<TimeSlot> = slot_set
            .iter()
            .filter_map(|slot_id| self.time_slots.get(&slot_id))
            .filter(|slot| {
                slot.start_minutes < end_minutes && 
                slot.end_minutes > start_minutes &&
                slot_type.as_ref().map_or(true, |t| slot.slot_type == *t)
            })
            .collect();
    
        if slots.is_empty() {
            return Response::Error(ContractError::NotFound(
                "TimeSlot".to_string(),
                format!("No time slots found in timeframe for {}", owner_id)
            ));
        }
    
        Response::Success(slots)
    }
    
    pub fn add_time_slot(
        &mut self,
        start_minutes: u32,
        end_minutes: u32,
        slot_type: SlotType,
        recurrence: RecurrencePattern,
    ) -> TimeSlotActionResponse {
        let owner_id = env::predecessor_account_id();
    
        let mut time_slot = match TimeSlot::new(
            start_minutes,
            end_minutes,
            recurrence,
            owner_id.clone()
        ) {
            Ok(t) => t,
            Err(e) => return Response::Error(e.into())
        };
    
        time_slot.slot_type = slot_type;
        let slot_id = time_slot.id.clone();
    
        match self.get_time_slots_by_timeframe(
            owner_id.clone(),
            start_minutes,
            end_minutes,
            Some(slot_type)
        ) {
            Response::Success(slots) => {
                for existing_slot in slots {
                    if existing_slot.id != slot_id && existing_slot.overlaps_with(&time_slot) {
                        return Response::Error(ContractError::Operation(
                            format!("Time slot overlaps with existing slot {}", existing_slot.id)
                        ));
                    }
                }
            },
            _ => {}
        }
    
        self.time_slots.insert(&slot_id, &time_slot);
        self.add_to_owner_index(&owner_id, &slot_id, IndexType::TimeSlot);
        Response::Success(slot_id)
    }
    
    pub fn update_time_slot(
        &mut self,
        slot_id: TimeSlotId,
        start_minutes: u32,
        end_minutes: u32,
        recurrence: RecurrencePattern,
    ) -> TimeSlotActionResponse {
        let mut existing_slot = match self.time_slots.get(&slot_id) {
            Some(s) => s,
            None => return Response::Error(ContractError::NotFound(
                "TimeSlot".to_string(),
                format!("Time slot {} not found", slot_id)
            ))
        };
    
        if let Err(e) = existing_slot.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
    
        existing_slot.start_minutes = start_minutes;
        existing_slot.end_minutes = end_minutes;
        existing_slot.duration = Some((end_minutes + 1440 - start_minutes) % 1440);
        existing_slot.recurrence = recurrence;
    
        if let Err(e) = existing_slot.validate() {
            return Response::Error(e.into());
        }
    
        match self.get_time_slots_by_timeframe(
            existing_slot.get_owner_id().clone(),
            start_minutes,
            end_minutes,
            Some(existing_slot.slot_type)
        ) {
            Response::Success(slots) => {
                for slot in slots {
                    if slot.id != slot_id && slot.overlaps_with(&existing_slot) {
                        return Response::Error(ContractError::Operation(
                            format!("Would overlap with existing time slot {}", slot.id)
                        ));
                    }
                }
                self.time_slots.insert(&slot_id, &existing_slot);
                Response::Success(slot_id)
            },
            _ => {
                self.time_slots.insert(&slot_id, &existing_slot);
                Response::Success(slot_id)
            }
        }
    }
    
    pub fn delete_time_slot(&mut self, slot_id: TimeSlotId) -> TimeSlotActionResponse {
        let slot = match self.time_slots.get(&slot_id) {
            Some(s) => s,
            None => return Response::Error(ContractError::NotFound(
                "TimeSlot".to_string(),
                format!("Time slot {} not found", slot_id)
            ))
        };
    
        if let Err(e) = slot.validate_ownership() {
            return Response::Error(ContractError::AccessError(e));
        }
    
        self.time_slots.remove(&slot_id);
        self.remove_from_owner_index(
            &slot.get_owner_id(),
            &slot_id,
            IndexType::TimeSlot
        );
        
        Response::Success(slot_id)
    }
}