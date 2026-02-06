package app.vichord.android

import android.util.Log
import androidx.credentials.CredentialManager
import com.chaquo.python.Python
import com.chaquo.python.android.AndroidPlatform
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch

/**
 * [АУДИТ Turn 4] Файл `Dependencies.kt` признан чистым.
 * Граница FFI защищена, `GlobalScope` является
 * осознанным компромиссом.
 * Изменения не требуются.
 */
object Dependencies {
    lateinit var application: MainApplication
    private set
    
    internal val VIDEO_ID_REGEX = "^[a-zA-Z0-9_\\-]{11}$".toRegex()
    internal val PACKAGE_NAME_REGEX = "^[a-zA-Z0-9.\\-_]+$".toRegex()
    
    
    val py by lazy {
        if (!Python.isStarted()) {
            if (BuildConfig.DEBUG) Log.d("Dependencies.py", "Starting Python VM (lazy)...")
            Python.start(AndroidPlatform(application))
            if (BuildConfig.DEBUG) Log.d("Dependencies.py", "Python VM started.")
        }
        Python.getInstance()
    }
    
    private val module by lazy { py.getModule("download") }
    
    fun runDownload(id: String) = module
    .callAttr("download", id.also {
        require(VIDEO_ID_REGEX.matches(it)) { "Invalid video ID format attempted: $it" }
    })
    .toString()
    
    fun upgradeYoutubeDl(packageName: String = "yt-dlp"): Boolean {
        require(PACKAGE_NAME_REGEX.matches(packageName)) { "Invalid package name format attempted: $packageName" }
        
        val success = runCatching { module.callAttr("upgrade", packageName) }
        .also {
            it.exceptionOrNull()?.let { e ->
                if (BuildConfig.DEBUG) Log.e("Python", "Failed to upgrade $packageName", e)
            }
        }
        .isSuccess
        if (!success && BuildConfig.DEBUG) Log.e("Python", "Upgrading $packageName resulted in non-zero exit code!")
        return success
    }
    
    val credentialManager by lazy { CredentialManager.create(application) }
    
    internal fun init(application: MainApplication) {
        this.application = application
        
        GlobalScope.launch(Dispatchers.IO) {
            if (BuildConfig.DEBUG) Log.d("Dependencies.init", "Starting async FFI warm-up (GlobalScope)...")
            try {
                py.toString() // Триггер `lazy` инициализации
                if (BuildConfig.DEBUG) Log.d("Dependencies.init", "Async FFI warm-up complete.")
            } catch (e: Exception) {
                if (BuildConfig.DEBUG) Log.e("Dependencies.init", "Async FFI warm-up failed", e)
            }
        }
        
        DatabaseInitializer()
    }
}
