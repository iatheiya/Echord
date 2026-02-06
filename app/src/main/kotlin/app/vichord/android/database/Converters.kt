package app.vichord.android

import android.os.Parcel
import androidx.annotation.OptIn
import androidx.media3.common.MediaItem
import androidx.media3.common.util.UnstableApi
import androidx.room.TypeConverter
import io.ktor.http.Url

/**
 * Type converters for Room.
 * Warning: Media3 UnstableApi still required 2025 (no stabilization per GitHub issues #176, #503).
 * Parcel RAII via recycle(); safe for ByteArray.
 */
@Suppress("unused")
object Converters {
    @TypeConverter
    @OptIn(UnstableApi::class)
    fun mediaItemFromByteArray(value: ByteArray?): MediaItem? = value?.let { byteArray ->
        runCatching {
            val parcel = Parcel.obtain()
            parcel.unmarshall(byteArray, 0, byteArray.size)
            parcel.setDataPosition(0)
            val bundle = parcel.readBundle(MediaItem::class.java.classLoader)
            parcel.recycle()
            bundle?.let(MediaItem::fromBundle)
        }.getOrNull()
    }

    @TypeConverter
    @OptIn(UnstableApi::class)
    fun mediaItemToByteArray(mediaItem: MediaItem?): ByteArray? = mediaItem?.toBundle()?.let { bundle ->
        val parcel = Parcel.obtain()
        parcel.writeBundle(bundle)
        val bytes = parcel.marshall()
        parcel.recycle()
        bytes
    }

    @TypeConverter
    fun urlToString(url: Url) = url.toString()

    @TypeConverter
    fun stringToUrl(string: String) = Url(string)
}