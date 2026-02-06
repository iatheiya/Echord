// kugou/mod.rs

// Подключаем локальные модули
pub mod models;
pub mod requests;

// Делаем FFI-функцию `lyrics` доступной
pub use requests::lyrics;

// Генерируем связующий код UniFFI
uniffi::include_scaffolding!("kugou");
