/*
 * ФАЙЛ: mod.rs
 * (АУДИТ ПРОЙДЕН)
 */

// (IMPROVEMENT: Cleaned up module structure)
pub mod models;

// (NEW: Added core module for shared HTTP logic per Mandate 2.0.1)
pub mod core;

// (IMPROVEMENT: `pub mod requests;` removed as `requests.rs` was empty)
// (MOVED_TO_MODULE: `requests.rs` was deleted)
