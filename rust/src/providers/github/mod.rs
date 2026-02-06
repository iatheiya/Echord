// Файл: mod.rs
// Корневой файл модуля, объединяющий модели и запросы

pub mod models;
pub mod requests;

// Включаем сгенерированный UniFFI код
uniffi::include_scaffolding!("github");
