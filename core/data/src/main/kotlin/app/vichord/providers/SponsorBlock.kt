// ------------------------------------
// Файл: SponsorBlockProvider.kt (Очистка по 1.3)
// (Этот файл ЗАМЕНЯЕТ все 5 исходных .kt файлов)
// ------------------------------------
package app.vitune.providers.sponsorblock

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import sponsorblock.Action
import sponsorblock.Category
import sponsorblock.Segment
import sponsorblock.SponsorBlockApi

/**
 * Kotlin-обертка (адаптер) для FFI-реализации SponsorBlockApi.
 *
 * [MOVED_TO_MODULE] Этот файл заменяет оригинальные:
 * 1. `SponsorBlock.kt`
 * 2. `Segments.kt`
 * 3. `Segment.kt`
 * 4. `Category.kt`
 * 5. `Action.kt`
 *
 * Вся логика и модели теперь принадлежат Rust и генерируются UniFFI.
 */
object SponsorBlockProvider {

    // Инициализируем Rust FFI объект
    private val api by lazy { SponsorBlockApi.new() }

    /**
     * Получает сегменты для видео из Rust-модуля.
     * Сигнатура этой функции семантически идентична
     * оригинальной `SponsorBlock.segments(...)` из `Segments.kt`.
     *
     * @param videoId ID видео YouTube.
     * @param categories Список категорий для запроса (по умолчанию: Sponsor, OfftopicMusic, PoiHighlight).
     * @param actions Список действий для запроса (по умолчанию: Skip, POI).
     * @param segments [AUDIT_FIX] Заменяет `SerializableUUID` на `String`. Список UUID сегментов для запроса.
     * @return Result<List<Segment>> Сегменты или ошибка FFI.
     */
    suspend fun segments(
        videoId: String,
        categories: List<Category>? = listOf(
            Category.SPONSOR,
            Category.OFFTOPIC_MUSIC,
            Category.POI_HIGHLIGHT
        ),
        actions: List<Action>? = listOf(Action.SKIP, Action.POI),
        segments: List<String>? = null // [AUDIT_FIX] Замена SerializableUUID -> String
    ): Result<List<Segment>> = withContext(Dispatchers.IO) {
        // Оборачиваем FFI-вызов в Result, чтобы перехватить
        // исключения FFI (например, SponsorBlockError)
        runCatching {
            api.getSegments(
                videoId = videoId,
                categories = categories,
                actions = actions,
                requiredSegments = segments
            )
        }
    }
}

// ПРИМЕЧАНИЕ:
// Модели `Segment`, `Category` и `Action`, используемые здесь,
// теперь являются классами, сгенерированными UniFFI (из sponsorblock.udl).
