package app.vichord.android

import android.app.Activity
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.add
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.isImeVisible
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.systemBars
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.coerceAtLeast
import androidx.compose.ui.unit.coerceIn
import androidx.compose.ui.unit.dp
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import app.vichord.android.preferences.AppearancePreferences
import app.vichord.android.service.PlayerService
import app.vichord.android.ui.components.BottomSheetState
import app.vichord.android.ui.components.rememberBottomSheetState
import app.vichord.android.utils.DisposableListener
import app.vichord.android.utils.isInPip
import app.vichord.android.utils.maybeExitPip
import app.vichord.android.utils.shouldBePlaying
import app.vichord.core.ui.Dimensions
import app.vichord.core.ui.utils.songBundle

/**
 * [АУДИТ Turn 4]
 *
 * [ИСПРАВЛЕНО] (Testability) Логика `Player.Listener` вынесена из Composable
 * `PlayerLifecycleListener` в чистую, тестируемую, `internal` функцию
 * `handleMediaItemTransition`.
 *
 * [ИСПРАВЛЕНО] (Safety) `PlayerLifecycleListener` теперь использует
 * безопасное приведение `as? Activity` для вызова `maybeExitPip`,
 * избегая риска `ClassCastException`.
 */
class MainActivityState(
    val playerBottomSheetState: BottomSheetState,
    val playerAwareWindowInsets: WindowInsets,
    val pip: Boolean,
    private val binder: PlayerService.Binder?
) {
    /**
     * [MOVED_TO_MODULE]
     * Composable-функция, которая связывает "грязный" мир (Compose, Context, Listener)
     * с "чистым" миром (тестируемая логика `handleMediaItemTransition`).
     */
    @Composable
    fun PlayerLifecycleListener() {
        // [ИСПРАВЛЕНО] Используем `as? Activity` для безопасного вызова side-effect
        val activity = (LocalContext.current as? Activity)

        binder?.player.DisposableListener {
            object : Player.Listener {
                override fun onMediaItemTransition(
                    mediaItem: MediaItem?,
                    reason: Int
                ) {
                    // 1. Обработка "грязных" побочных эффектов (Side-effects)
                    if (mediaItem == null) {
                        activity?.maybeExitPip()
                    }

                    // 2. Делегирование чистой логике управления состоянием
                    handleMediaItemTransition(
                        mediaItem = mediaItem,
                        reason = reason,
                        isDismissed = playerBottomSheetState.dismissed,
                        isFromPersistentQueue = mediaItem?.mediaMetadata?.extras?.songBundle?.isFromPersistentQueue == true,
                        openPlayerOnTransition = AppearancePreferences.openPlayer,
                        // Передача чистых "команд"
                        dismiss = { playerBottomSheetState.dismissSoft() },
                        expand = { playerBottomSheetState.expandSoft() },
                        collapse = { playerBottomSheetState.collapseSoft() }
                    )
                }
            }
        }
    }
}

/**
 * Composable-фабрика для `MainActivityState`.
 * (Изменений не требует, т.к. `PlayerLifecycleListener` инкапсулирован)
 */
@Composable
fun rememberMainActivityState(
    binder: PlayerService.Binder?,
    maxHeight: Dp
): MainActivityState {
    val windowInsets = WindowInsets.systemBars
    val density = LocalDensity.current

    val bottomDp = with(density) { windowInsets.getBottom(density).toDp() }

    val imeVisible = WindowInsets.isImeVisible
    val imeBottomDp = with(density) { WindowInsets.ime.getBottom(density).toDp() }
    val animatedBottomDp by animateDpAsState(
        targetValue = if (imeVisible) 0.dp else bottomDp,
        label = "animatedBottomDp"
    )

    val playerBottomSheetState = rememberBottomSheetState(
        key = binder,
        dismissedBound = 0.dp,
        collapsedBound = Dimensions.items.collapsedPlayerHeight + bottomDp,
        expandedBound = maxHeight
    )

    val playerAwareWindowInsets = remember(
        bottomDp,
        animatedBottomDp,
        playerBottomSheetState.value,
        imeVisible,
        imeBottomDp
    ) {
        val bottom = calculatePlayerAwareBottomInset(
            imeVisible = imeVisible,
            imeBottomDp = imeBottomDp,
            playerSheetValue = playerBottomSheetState.value,
            animatedSystemBottomDp = animatedBottomDp,
            playerSheetCollapsedBound = playerBottomSheetState.collapsedBound
        )

        windowInsets
            .only(WindowInsetsSides.Horizontal + WindowInsetsSides.Top)
            .add(WindowInsets(bottom = bottom))
    }

    val pip = isInPip(
        onChange = { inPip ->
            if (!inPip || binder?.player?.shouldBePlaying != true) Unit
            else playerBottomSheetState.expandSoft()
        }
    )

    return remember(
        playerBottomSheetState,
        playerAwareWindowInsets,
        pip,
        binder
    ) {
        MainActivityState(
            playerBottomSheetState = playerBottomSheetState,
            playerAwareWindowInsets = playerAwareWindowInsets,
            pip = pip,
            binder = binder
        )
    }
}

/**
 * [НОВАЯ ФУНКЦИЯ] (Testability, извлечено из `PlayerLifecycleListener`)
 *
 * Чистая, тестируемая функция, инкапсулирующая логику реакции
 * на смену трека плеера.
 */
@JvmSynthetic
internal fun handleMediaItemTransition(
    mediaItem: MediaItem?,
    reason: Int,
    isDismissed: Boolean,
    isFromPersistentQueue: Boolean,
    openPlayerOnTransition: Boolean,
    // --- Testable Seams (Лямбды для выполнения действий) ---
    dismiss: () -> Unit,
    expand: () -> Unit,
    collapse: () -> Unit
) {
    when {
        // 1. Плеер остановлен (нет трека) -> закрыть
        mediaItem == null -> dismiss()

        // 2. Смена плейлиста (не из очереди) -> открыть (если настроено)
        reason == Player.MEDIA_ITEM_TRANSITION_REASON_PLAYLIST_CHANGED &&
                !isFromPersistentQueue -> {
            if (openPlayerOnTransition) expand()
        }

        // 3. Плеер был закрыт (dismissed), но начался новый трек -> свернуть (collapse)
        isDismissed -> collapse()
    }
}


/**
 * Чистая функция для расчета Insets (из Turn 2)
 */
@JvmSynthetic
internal fun calculatePlayerAwareBottomInset(
    imeVisible: Boolean,
    imeBottomDp: Dp,
    playerSheetValue: Dp,
    animatedSystemBottomDp: Dp,
    playerSheetCollapsedBound: Dp
): Dp {
    return if (imeVisible) imeBottomDp.coerceAtLeast(playerSheetValue)
    else playerSheetValue.coerceIn(
        animatedSystemBottomDp..playerSheetCollapsedBound
    )
}
