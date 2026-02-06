package app.vichord.android

import androidx.compose.ui.unit.dp
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import org.mockito.Mockito.mock

/**
 * [АУДИТ Turn 4]
 *
 * [УЛУЧШЕНО] (Testability) Добавлен `testPlayerStateResponderLogic`
 * для покрытия новой чистой функции `handleMediaItemTransition`.
 */
class RefactoringAuditTests {

    @Test
    fun testFfiVideoIdValidation() {
        val validIds = listOf("dQw4w9WgXcQ", "a-b_c-D_123", "12345678901")
        for (id in validIds) {
            assertTrue("Expected $id to be valid", Dependencies.VIDEO_ID_REGEX.matches(id))
        }

        val invalidIds = listOf(
            "dQw4w9WgXcQ1",
            "dQw4w9WgXc",
            "dQw4w9WgXc!",
            "<script>",
            ""
        )
        for (id in invalidIds) {
            assertFalse("Expected $id to be invalid", Dependencies.VIDEO_ID_REGEX.matches(id))
        }

        assertThrows(IllegalArgumentException::class.java) {
            val testId = "<script>"
            require(Dependencies.VIDEO_ID_REGEX.matches(testId)) { "Invalid ID" }
        }
    }

    @Test
    fun testPlayerAwareInsetsLogic() {
        // Сценарий 1: Клавиатура открыта (IME > Player)
        val bottom1 = calculatePlayerAwareBottomInset(
            imeVisible = true,
            imeBottomDp = 300.dp,
            playerSheetValue = 80.dp,
            animatedSystemBottomDp = 0.dp,
            playerSheetCollapsedBound = 80.dp
        )
        assertEquals(300.dp, bottom1)

        // Сценарий 2: Клавиатура открыта (Player > IME)
        val bottom2 = calculatePlayerAwareBottomInset(
            imeVisible = true,
            imeBottomDp = 100.dp,
            playerSheetValue = 200.dp,
            animatedSystemBottomDp = 0.dp,
            playerSheetCollapsedBound = 80.dp
        )
        assertEquals(200.dp, bottom2)

        // Сценарий 3: Клавиатура закрыта, плеер свернут
        val bottom3 = calculatePlayerAwareBottomInset(
            imeVisible = false,
            imeBottomDp = 0.dp,
            playerSheetValue = 80.dp,
            animatedSystemBottomDp = 40.dp,
            playerSheetCollapsedBound = 80.dp
        )
        assertEquals(80.dp, bottom3)

        // Сценарий 5: Клавиатура закрыта, плеер в процессе анимации (скрывается)
        val bottom5 = calculatePlayerAwareBottomInset(
            imeVisible = false,
            imeBottomDp = 0.dp,
            playerSheetValue = 20.dp,
            animatedSystemBottomDp = 40.dp,
            playerSheetCollapsedBound = 80.dp
        )
        assertEquals(40.dp, bottom5)
    }

    /**
     * [НОВЫЙ ТЕСТ]
     * Тестирует чистую логику `handleMediaItemTransition`,
     * извлеченную из `PlayerLifecycleListener`.
     */
    @Test
    fun testPlayerStateResponderLogic() {
        // --- Test Scaffolding ---
        val mockMediaItem: MediaItem = mock(MediaItem::class.java)
        var dismissCalled = false
        var expandCalled = false
        var collapseCalled = false

        val resetMocks = {
            dismissCalled = false
            expandCalled = false
            collapseCalled = false
        }

        // --- Сценарий 1: Плеер остановлен (mediaItem == null) ---
        resetMocks()
        handleMediaItemTransition(
            mediaItem = null,
            reason = 0,
            isDismissed = false,
            isFromPersistentQueue = false,
            openPlayerOnTransition = true,
            dismiss = { dismissCalled = true },
            expand = { expandCalled = true },
            collapse = { collapseCalled = true }
        )
        assertTrue(dismissCalled)
        assertFalse(expandCalled)
        assertFalse(collapseCalled)

        // --- Сценарий 2: Смена плейлиста, не из очереди, настройка "openPlayer" = true ---
        resetMocks()
        handleMediaItemTransition(
            mediaItem = mockMediaItem,
            reason = Player.MEDIA_ITEM_TRANSITION_REASON_PLAYLIST_CHANGED,
            isDismissed = false,
            isFromPersistentQueue = false,
            openPlayerOnTransition = true,
            dismiss = { dismissCalled = true },
            expand = { expandCalled = true },
            collapse = { collapseCalled = true }
        )
        assertFalse(dismissCalled)
        assertTrue(expandCalled)
        assertFalse(collapseCalled)

        // --- Сценарий 3: Смена плейлиста, но настройка "openPlayer" = false ---
        resetMocks()
        handleMediaItemTransition(
            mediaItem = mockMediaItem,
            reason = Player.MEDIA_ITEM_TRANSITION_REASON_PLAYLIST_CHANGED,
            isDismissed = false,
            isFromPersistentQueue = false,
            openPlayerOnTransition = false, // <--
            dismiss = { dismissCalled = true },
            expand = { expandCalled = true },
            collapse = { collapseCalled = true }
        )
        assertFalse(dismissCalled)
        assertFalse(expandCalled) // <-- Не должен открываться
        assertFalse(collapseCalled)

        // --- Сценарий 4: Смена плейлиста, но из очереди (persistent queue) ---
        resetMocks()
        handleMediaItemTransition(
            mediaItem = mockMediaItem,
            reason = Player.MEDIA_ITEM_TRANSITION_REASON_PLAYLIST_CHANGED,
            isDismissed = false,
            isFromPersistentQueue = true, // <--
            openPlayerOnTransition = true,
            dismiss = { dismissCalled = true },
            expand = { expandCalled = true },
            collapse = { collapseCalled = true }
        )
        assertFalse(dismissCalled)
        assertFalse(expandCalled) // <-- Не должен открываться
        assertFalse(collapseCalled)

        // --- Сценарий 5: Плеер был закрыт (isDismissed), но начался новый трек (не смена плейлиста) ---
        resetMocks()
        handleMediaItemTransition(
            mediaItem = mockMediaItem,
            reason = Player.MEDIA_ITEM_TRANSITION_REASON_AUTO, // (любая другая причина)
            isDismissed = true, // <--
            isFromPersistentQueue = false,
            openPlayerOnTransition = true,
            dismiss = { dismissCalled = true },
            expand = { expandCalled = true },
            collapse = { collapseCalled = true }
        )
        assertFalse(dismissCalled)
        assertFalse(expandCalled)
        assertTrue(collapseCalled) // <-- Должен свернуться (collapse)

        // --- Сценарий 6: Плеер был закрыт, но смена плейлиста (Сценарий 2 > Сценарий 5) ---
        resetMocks()
        handleMediaItemTransition(
            mediaItem = mockMediaItem,
            reason = Player.MEDIA_ITEM_TRANSITION_REASON_PLAYLIST_CHANGED,
            isDismissed = true, // <--
            isFromPersistentQueue = false,
            openPlayerOnTransition = true,
            dismiss = { dismissCalled = true },
            expand = { expandCalled = true },
            collapse = { collapseCalled = true }
        )
        assertFalse(dismissCalled)
        assertTrue(expandCalled) // <-- Должен открыться (expand)
        assertFalse(collapseCalled)
    }
}
