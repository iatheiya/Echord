package app.vichord.android

import android.app.Application
import android.os.StrictMode
import android.util.Log
import androidx.work.Configuration
import app.vichord.android.preferences.DataPreferences
import app.vichord.android.service.ServiceNotifications
import app.vichord.compose.persist.PersistMap
import app.vichord.core.ui.utils.isAtLeastAndroid12
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.SingletonImageLoader
import coil3.bitmapFactoryExifOrientationStrategy
import coil3.decode.ExifOrientationStrategy
import coil3.disk.DiskCache
import coil3.disk.directory
import coil3.memory.MemoryCache
import coil3.request.crossfade
import coil3.util.DebugLogger
import com.kieronquinn.monetcompat.core.MonetCompat

/**
 * [АУДИТ Turn 4] Файл `MainApplication.kt` признан чистым.
 * Изменения не требуются.
 */
class MainApplication : Application(), SingletonImageLoader.Factory, Configuration.Provider {
    
    override fun onCreate() {
        System.loadLibrary("vichord_rust")
        
        StrictMode.setVmPolicy(
            StrictMode.VmPolicy.Builder()
            .let {
                if (isAtLeastAndroid12) it.detectUnsafeIntentLaunch()
                else it
            }
            .penaltyLog()
            .penaltyDeath()
            .build()
        )
        
        MonetCompat.debugLog = BuildConfig.DEBUG
        super.onCreate()
        
        Dependencies.init(this)
        MonetCompat.enablePaletteCompat()
        ServiceNotifications.createAll()
    }
    
    override fun newImageLoader(context: PlatformContext) = ImageLoader.Builder(this)
    .crossfade(true)
    .memoryCache {
        MemoryCache.Builder()
        .maxSizePercent(context, 0.1)
        .build()
    }
    .diskCache {
        DiskCache.Builder()
        .directory(context.cacheDir.resolve("coil"))
        .maxSizeBytes(DataPreferences.coilDiskCacheMaxSize.bytes)
        .build()
    }
    .bitmapFactoryExifOrientationStrategy(ExifOrientationStrategy.IGNORE)
    .let { if (BuildConfig.DEBUG) it.logger(DebugLogger()) else it }
    .build()
    
    val persistMap = PersistMap()
    
    override val workManagerConfiguration = Configuration.Builder()
    .setMinimumLoggingLevel(if (BuildConfig.DEBUG) Log.DEBUG else Log.INFO)
    .build()
}
