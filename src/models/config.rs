// === Storage Constants ===
pub mod storage {
    pub const REWARD_BASE_STORAGE: u64 = 128;
    pub const REWARD_MAX_STORAGE: u64 = 2048;
    pub const TASK_BASE_STORAGE: u64 = 256;
    pub const TASK_MAX_STORAGE: u64 = 4096;
    pub const TIME_SLOT_BASE_STORAGE: u64 = 128;
    pub const TIME_SLOT_MAX_STORAGE: u64 = 2048;
}

// === Time Related Constants ===
pub mod time {
    pub const MAX_MINUTES: u32 = 24 * 60;
    pub const MAX_FUTURE_TIME: u64 = 365 * 24 * 60 * 60 * 1_000_000_000;
    pub const MAX_SLOT_FUTURE_TIME: u64 = 30 * 24 * 60 * 60 * 1_000_000_000;
}

// === Task Related Constants ===
pub mod task {
    pub const MAX_TITLE_LENGTH: usize = 256;
    pub const MAX_DESCRIPTION_LENGTH: usize = 1024;
}

// === Reward Related Constants ===
pub mod reward {
    pub const MAX_TITLE_LENGTH: usize = 256;
    pub const MAX_DESCRIPTION_LENGTH: usize = 1024;
}