// Файл: core/mod.rs
// Регламент 2.0.2: Корневой файл 'core' для общих утилит.
// Объединяет унифицированные модули ошибок, HTTP, JSON и FFI-реализации.

pub mod error;
pub mod http;
pub mod json;
pub mod request_impl; // Реализация универсальных FFI-запросов
