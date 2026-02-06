package app.vichord.android.utils

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.staticCompositionLocalOf
import app.vichord.android.Dependencies
import app.vichord.android.service.PlayerService

/**
 * [АУДИТ Turn 4] Файл `CompositionLocals.kt` признан чистым.
 * Изменения не требуются.
 */

val LocalPlayerServiceBinder = staticCompositionLocalOf<PlayerService.Binder?> { null }
val LocalPlayerAwareWindowInsets =
    compositionLocalOf<WindowInsets> { error("No player insets provided") }
val LocalCredentialManager = staticCompositionLocalOf { Dependencies.credentialManager }
