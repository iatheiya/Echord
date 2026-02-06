// [Lyrics.kt]
// [AUDITED] Без изменений.

package app.vichord.providers.innertube.requests

import android.util.Log
import app.vichord.BuildConfig
import app.vichord.providers.innertube.Innertube
import app.vichord.providers.utils.runCatchingCancellable

// Импорты FFI из app.vitune.rust
import app.vitune.rust.InnertubeClient
import app.vitune.rust.InnertubeError
import app.vichord.providers.innertube.models.Context // Импорт для Context
import app.vichord.providers.innertube.toFfi

private val innertubeClient = InnertubeClient.shared()

/**
 * Получает текст песни для заданного видео ID.
 *
 * Логика полностью делегирована Rust FFI (один асинхронный FFI-вызов,
 * который управляет сетевой оркестровкой: /next -> /browse).
 *
 * @param videoId Идентификатор видео.
 * @return Result со строкой текста или null.
 */
suspend fun Innertube.lyrics(videoId: String): Result<String?>? = runCatchingCancellable {
    if (videoId.isBlank()) {
        if (BuildConfig.DEBUG) {
            Log.w("InnertubeLyrics", "Video ID is blank, cannot fetch lyrics.")
        }
        return@runCatchingCancellable null
    }
    
    // [UNIFIED] Используем DefaultAndroidMusic
    val kotlinContext = Context.DefaultAndroidMusic
    
    // Используем унифицированный хелпер 'toFfi()' из InnertubeFfiExt.kt
    val ffiContext = kotlinContext.toFfi()
    val userAgent = kotlinContext.client.userAgent
    
    // [FINALIZED] Единый FFI-вызов, инкапсулирующий всю IO и парсинг.
    val lyricsText = try {
        // [PRE-CONDITION]: videoId не пуст, ffiContext и userAgent корректны.
        // [POST-CONDITION]: Возвращается Optional<String> или InnertubeError (Network/Logic/JSON).
        innertubeClient.fetchLyrics(
            videoId = videoId,
            context = ffiContext,
            userAgent = userAgent
        )
    } catch (e: InnertubeError) {
        if (BuildConfig.DEBUG) {
            Log.e("InnertubeLyrics", "FFI fetchLyrics failed: ${e.message}", e)
        }
        // FFI-ошибки (NetworkError, LogicError) пробрасываются вверх
        throw e
    }
    
    if (BuildConfig.DEBUG && lyricsText == null) {
        Log.w("InnertubeLyrics", "Lyrics text not found (null) for videoId: $videoId")
    }
    
    return@runCatchingCancellable lyricsText
}
