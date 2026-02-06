package app.vichord.android

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import app.vichord.android.service.PlayerService
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first

/**
 * [АУДИТ Turn 4] Файл `MainViewModel.kt` признан чистым.
 * Изменения не требуются.
 */
class MainViewModel : ViewModel() {
    var binder: PlayerService.Binder? by mutableStateOf(null)

    suspend fun awaitBinder(): PlayerService.Binder =
        binder ?: snapshotFlow { binder }.filterNotNull().first()
}
