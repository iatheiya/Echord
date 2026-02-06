// Файл: app/vitune/providers/lrclib/models/Track.kt
// Очищенная версия: Удалена вся бизнес-логика (lrc, bestMatchingFor),
// которая была перенесена в Rust.

package app.vichord.providers.lrclib

import kotlinx.serialization.Serializable
// MOVED_TO_MODULE: LrcParser и kotlin.time.Duration больше не нужны здесь

@Serializable
data class Track(
    val id: Int,
    val trackName: String,
    val artistName: String,
    val duration: Double,
    val plainLyrics: String?,
    val syncedLyrics: String?
)

// MOVED_TO_MODULE: Свойство `lrc` УДАЛЕНО (логика в Rust `models.rs`)
// MOVED_TO_MODULE: Функция `bestMatchingFor` УДАЛЕНА (логика в Rust `requests.rs`)
