=== FILE: src/piped/mod.rs ===
// ФАЙЛ БЕЗ ИЗМЕНЕНИЙ (УЧАСТВУЕТ В СБОРКЕ)
pub mod models;
pub mod requests;

// ОБЯЗАТЕЛЬНОЕ ВКЛЮЧЕНИЕ:
pub mod core; // ДОБАВЛЕНИЕ МОДУЛЯ CORE В КОМПИЛЯЦИЮ (если core находится на уровне src/)

uniffi::include_scaffolding!("piped");
