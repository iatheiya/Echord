// [NEW FILE] (app/vichord/providers/innertube/requests/InnertubeFfiExt.kt)
// [AUDITED] Без изменений.

package app.vichord.providers.innertube

import app.vichord.providers.innertube.models.Context
// [MODIFIED] Импорт FFI-типа из app.vitune.rust
import app.vitune.rust.Context as FfiContext

/**
 * [UNIFIED] Преобразует Kotlin Context (Kotlinx-модель) в FFI Context (UniFFI Enum).
 */
internal fun Context.toFfi(): FfiContext {
    return when (this) {
        Context.DefaultAndroidMusic -> FfiContext.DEFAULT_ANDROID_MUSIC
        Context.DefaultIOS -> FfiContext.DEFAULT_IOS
        Context.DefaultWeb -> FfiContext.DEFAULT_WEB
        Context.DefaultTV -> FfiContext.DEFAULT_TV
        // Другие контексты, если добавлены, должны быть обработаны здесь
        // [AUDIT] Kotlin `else` гарантирует, что неизвестные контексты
        // (например, кастомные) будут обработаны.
        else -> FfiContext.DEFAULT_WEB
    }
}
