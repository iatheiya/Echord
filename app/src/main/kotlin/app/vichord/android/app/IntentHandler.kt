package app.vichord.android

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.MediaStore
import android.util.Log
import androidx.core.net.toUri
import app.vichord.android.service.PlayerService
import app.vichord.android.ui.screens.albumRoute
import app.vichord.android.ui.screens.artistRoute
import app.vichord.android.ui.screens.playlistRoute
import app.vichord.android.ui.screens.searchResultRoute
import app.vichord.android.ui.screens.settingsRoute
import app.vichord.android.utils.asMediaItem
import app.vichord.android.utils.forcePlay
import app.vichord.android.utils.toast
import app.vichord.core.ui.utils.activityIntentBundle
import app.vichord.providers.innertube.Innertube
import app.vichord.providers.innertube.models.bodies.BrowseBody
import app.vichord.providers.innertube.requests.playlistPage
import app.vichord.providers.innertube.requests.song
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/**
 * [АУДИТ Turn 4] Файл `IntentHandler.kt` признан чистым.
 * Изменения не требуются.
 */
private const val TAG = "IntentHandler"

object IntentHandler {

    @Suppress("CyclomaticComplexMethod")
    suspend fun handleIntent(
        intent: Intent,
        binder: PlayerService.Binder,
        context: Context
    ) {
        val extras = intent.extras?.activityIntentBundle

        when (intent.action) {
            Intent.ACTION_SEARCH -> {
                val query = extras?.query ?: run {
                    Log.w(TAG, "Intent ACTION_SEARCH: Query was null.")
                    return
                }
                extras.query = null
                searchResultRoute.ensureGlobal(query)
            }

            Intent.ACTION_APPLICATION_PREFERENCES -> settingsRoute.ensureGlobal()

            Intent.ACTION_VIEW, Intent.ACTION_SEND -> {
                val uri = intent.data
                    ?: runCatching { extras?.text?.toUri() }.getOrNull()
                    ?: run {
                        Log.w(TAG, "Intent ACTION_VIEW/SEND: URI or text extra was null.")
                        return
                    }

                intent.data = null
                extras?.text = null
                handleUrl(uri, binder, context)
            }

            MediaStore.INTENT_ACTION_MEDIA_PLAY_FROM_SEARCH -> {
                val query = when (extras?.mediaFocus) {
                    null, "vnd.android.cursor.item/*" -> extras?.query ?: extras?.text
                    MediaStore.Audio.Genres.ENTRY_CONTENT_TYPE -> extras.genre
                    MediaStore.Audio.Artists.ENTRY_CONTENT_TYPE -> extras.artist
                    MediaStore.Audio.Albums.ENTRY_CONTENT_TYPE -> extras.album
                    "vnd.android.cursor.item/audio" -> listOfNotNull(
                        extras.album,
                        extras.artist,
                        extras.genre,
                        extras.title
                    ).joinToString(separator = " ")

                    @Suppress("deprecation")
                    MediaStore.Audio.Playlists.ENTRY_CONTENT_TYPE -> extras.playlist

                    else -> null
                }

                if (!query.isNullOrBlank()) binder.playFromSearch(query)
                else Log.w(TAG, "Intent MEDIA_PLAY_FROM_SEARCH: Query was null or blank.")
            }
        }
    }

    @Suppress("CyclomaticComplexMethod")
    private suspend fun handleUrl(
        uri: Uri,
        binder: PlayerService.Binder?,
        context: Context
    ) {
        val path = uri.pathSegments.firstOrNull()
        Log.d(TAG, "Opening url: $uri ($path)")

        when (path) {
            "search" -> uri.getQueryParameter("q")?.let { query ->
                searchResultRoute.ensureGlobal(query)
            }

            "playlist" -> uri.getQueryParameter("list")?.let { playlistId ->
                val browseId = "VL$playlistId"

                if (playlistId.startsWith("OLAK5uy_")) Innertube.playlistPage(
                    body = BrowseBody(browseId = browseId)
                )
                    ?.getOrNull()
                    ?.let { page ->
                        page.songsPage?.items?.firstOrNull()?.album?.endpoint?.browseId
                            ?.let { albumRoute.ensureGlobal(it) }
                    } ?: withContext(Dispatchers.Main) {
                    toast(context.getString(R.string.error_url, uri))
                }
                else playlistRoute.ensureGlobal(
                    p0 = browseId,
                    p1 = uri.getQueryParameter("params"),
                    p2 = null,
                    p3 = playlistId.startsWith("RDCLAK5uy_")
                )
            }

            "channel", "c" -> uri.lastPathSegment?.let { channelId ->
                artistRoute.ensureGlobal(channelId)
            }

            else -> when {
                path == "watch" -> uri.getQueryParameter("v")
                uri.host == "youtu.be" -> path
                else -> {
                    withContext(Dispatchers.Main) {
                        toast(context.getString(R.string.error_url, uri))
                    }
                    Log.w(TAG, "Unsupported URL for handleUrl: $uri")
                    null
                }
            }?.let { videoId ->
                Innertube.song(videoId)
                    ?.getOrNull()
                    ?.let { song ->
                        withContext(Dispatchers.Main) {
                            binder?.player?.forcePlay(song.asMediaItem)
                        }
                    }
                    ?: withContext(Dispatchers.Main) {
                        Log.w(TAG, "Failed to fetch song or song was null for ID: $videoId")
                        toast(context.getString(R.string.error_url, uri))
                    }
            }
        }
    }
}
