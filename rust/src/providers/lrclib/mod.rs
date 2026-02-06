// Файл: mod.rs
// Корневой файл модуля lrclib

pub mod models;
pub mod requests;

// Включаем код, сгенерированный UniFFI
// АУДИТ 4.1: Предполагается, что 'uniffi' настроен
// для компиляции 'lrclib.udl' И 'core.udl' вместе.
uniffi::include_scaffolding!("lrclib");
