package app.vichord.android.preferences

import app.vichord.android.Dependencies
import app.vichord.compose.preferences.PreferencesHolder

/**
 * [АУДИТ Turn 4] Файл `GlobalPreferencesHolder.kt` признан чистым.
 * Изменения не требуются.
 */
open class GlobalPreferencesHolder : PreferencesHolder(Dependencies.application, "preferences")
