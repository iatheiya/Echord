// [Player.kt]
// [AUDITED] Без изменений.

package app.vichord.providers.innertube.requests

import app.vichord.providers.innertube.Innertube
import app.vichord.providers.innertube.models.Context
import app.vichord.providers.utils.runCatchingCancellable
import io.ktor.util.generateNonce
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.isActive
import android.util.Log
import app.vichord.BuildConfig
import app.vichord.providers.innertube.models.bodies.QueueBody

// [MODIFIED] Импорты FFI из app.vitune.rust
import app.vitune.rust.InnertubeClient
import app.vitune.rust.PlayerBody as FfiPlayerBody
import app.vitune.rust.PlayerResponse as FfiPlayerResponse
import app.vitune.rust.SongItem as FfiSongItem
import app.vitune.rust.InnertubeError

// [FIX] Импорт унифицированного FFI-маппера
import app.vichord.providers.innertube.toFfi

private val innertubeClient = InnertubeClient.shared()

// =================================================================
// ЛОГИКА PLAYER
// =================================================================

private suspend fun Innertube.tryContexts(
    body: app.vichord.providers.innertube.models.bodies.PlayerBody,
    checkIsValid: Boolean,
    vararg contexts: Context
): FfiPlayerResponse? { // [MODIFIED] Тип FfiPlayerResponse
    val signatureTimestamp = client.getSignatureTimestamp().getOrNull()
    
    contexts.forEach { context ->
        if (!currentCoroutineContext().isActive) return null
        
        logger.info("Trying ${context.client.clientName} context for player request.")
        
        val ffiBody = FfiPlayerBody( // [MODIFIED] Тип FfiPlayerBody
            videoId = body.videoId,
            playlistId = body.playlistId,
            cpn = body.cpn ?: generateNonce(),
            signatureTimestamp = signatureTimestamp
        )
        
        val playerResponse = try {
            // [AUDITED] Вызов FFI-метода (IO + Парсинг)
            innertubeClient.fetchPlayer(
                body = ffiBody,
                context = context.toFfi(), // Использует унифицированный .toFfi()
                userAgent = context.client.userAgent
            ).getOrThrow()
        } catch (e: Exception) {
            if (BuildConfig.DEBUG) Log.e("InnertubePlayer", "FFI fetchPlayer failed for ${context.client.clientName}: ${e.message}", e)
            return@forEach
        }
        
        val isValid = playerResponse.playabilityStatusStatus == "OK" && playerResponse.streamingDataPresent
        
        if (isValid) {
            return playerResponse
        }
        
        if (BuildConfig.DEBUG) {
            Log.w("InnertubePlayer", "Response for ${context.client.clientName} is invalid (Status: ${playerResponse.playabilityStatusStatus}, StreamingData: ${playerResponse.streamingDataPresent}). Trying next context.")
        }
    }
    
    return null
}

suspend fun Innertube.player(
    body: app.vichord.providers.innertube.models.bodies.PlayerBody,
    checkIsValid: Boolean = true
): Result<FfiPlayerResponse?>? = runCatchingCancellable { // [MODIFIED] Тип FfiPlayerResponse
    tryContexts(
        body = body,
        checkIsValid = checkIsValid,
        Context.DefaultIOS,
        Context.DefaultWeb,
        Context.DefaultAndroidMusic,
        Context.DefaultTV
    )
}

// =================================================================
// ЛОГИКА QUEUE и SONG
// =================================================================

suspend fun Innertube.queue(body: QueueBody): Result<List<FfiSongItem>>? = runCatchingCancellable { // [MODIFIED] Тип FfiSongItem
    
    val kotlinContext = Context.DefaultAndroidMusic
    
    // [AUDITED] Вызов FFI-метода (IO + Парсинг)
    innertubeClient.fetchQueue(
        videoId = body.videoId,
        playlistId = body.playlistId,
        context = kotlinContext.toFfi(),
        userAgent = kotlinContext.client.userAgent
    ).getOrThrow()
}

suspend fun Innertube.song(videoId: String): Result<FfiSongItem?>? = runCatchingCancellable { // [MODIFIED] Тип FfiSongItem
    val body = QueueBody(videoId = videoId)
    queue(body)?.getOrThrow()?.firstOrNull()
}
