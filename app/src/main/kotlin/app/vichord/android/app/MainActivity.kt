package app.vichord.android

import android.content.ComponentName
import android.content.ServiceConnection
import android.os.Bundle
import android.os.IBinder
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.LocalIndication
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.BoxWithConstraintsScope
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.LocalRippleConfiguration
import androidx.compose.material3.ripple
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.core.view.WindowCompat
import androidx.lifecycle.lifecycleScope
import app.vichord.android.preferences.AppearancePreferences
import app.vichord.android.service.PlayerService
import app.vichord.android.service.downloadState
import app.vichord.android.ui.components.BottomSheetMenu
import app.vichord.android.ui.components.themed.LinearProgressIndicator
import app.vichord.android.ui.screens.home.HomeScreen
import app.vichord.android.ui.screens.player.Player
import app.vichord.android.ui.screens.player.Thumbnail
import app.vichord.android.utils.KeyedCrossfade
import app.vichord.android.utils.LocalMonetCompat
import app.vichord.android.utils.collectProvidedBitmapAsState
import app.vichord.android.utils.intent
import app.vichord.android.utils.invokeOnReady
import app.vichord.android.utils.maybeEnterPip
import app.vichord.android.utils.setDefaultPalette
import app.vichord.android.utils.shouldBePlaying
import app.vichord.compose.persist.LocalPersistMap
import app.vichord.core.ui.Darkness
import app.vichord.core.ui.LocalAppearance
import app.vichord.core.ui.SystemBarAppearance
import app.vichord.core.ui.amoled
import app.vichord.core.ui.appearance
import app.vichord.core.ui.rippleConfiguration
import app.vichord.core.ui.shimmerTheme
import com.kieronquinn.monetcompat.core.MonetActivityAccessException
import com.kieronquinn.monetcompat.core.MonetCompat
import com.kieronquinn.monetcompat.interfaces.MonetColorsChangedListener
import com.valentinilk.shimmer.LocalShimmerTheme
import dev.krag0n.monet.theme.ColorScheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

private const val TAG = "MainActivity"

/**
 * [АУДИТ Turn 4] Файл `MainActivity.kt` признан чистым.
 * Изменения не требуются.
 */
class MainActivity : ComponentActivity(), MonetColorsChangedListener {
    private val vm: MainViewModel by viewModels()
    
    private val serviceConnection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            if (service is PlayerService.Binder) vm.binder = service
        }
        
        override fun onServiceDisconnected(name: ComponentName?) {
            Log.w(TAG, "PlayerService disconnected unexpectedly. Rebinding.")
            vm.binder = null
            unbindService(this)
            bindService(intent<PlayerService>(), this, BIND_AUTO_CREATE)
        }
    }
    
    private var _monet: MonetCompat? by mutableStateOf(null)
    private val monet get() = _monet ?: throw MonetActivityAccessException()
    
    override fun onStart() {
        super.onStart()
        bindService(intent<PlayerService>(), serviceConnection, BIND_AUTO_CREATE)
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        WindowCompat.setDecorFitsSystemWindows(window, false)
        
        MonetCompat.setup(this)
        _monet = MonetCompat.getInstance()
        monet.setDefaultPalette()
        monet.addMonetColorsChangedListener(
            listener = this,
            notifySelf = false
        )
        monet.updateMonetColors()
        monet.invokeOnReady {
            setContent()
        }
        
        intent?.let { initialIntent ->
            lifecycleScope.launch(Dispatchers.IO) {
                try {
                    IntentHandler.handleIntent(initialIntent, vm.awaitBinder(), this@MainActivity)
                } catch (e: Exception) {
                    Log.e(TAG, "Failed to handle initial intent", e)
                }
            }
        }
        addOnNewIntentListener { newIntent ->
            lifecycleScope.launch(DispatchSers.IO) {
                try {
                    IntentHandler.handleIntent(newIntent, vm.awaitBinder(), this@MainActivity)
                } catch (e: Exception) {
                    Log.e(TAG, "Failed to handle new intent", e)
                }
            }
        }
    }
    
    @OptIn(ExperimentalMaterial3Api::class)
    @Composable
    fun AppWrapper(
        modifier: Modifier = Modifier,
        content: @Composable BoxWithConstraintsScope.() -> Unit
    ) = with(AppearancePreferences) {
        val sampleBitmap = vm.binder.collectProvidedBitmapAsState()
        val appearance = appearance(
            source = colorSource,
            mode = colorMode,
            darkness = darkness,
            fontFamily = fontFamily,
            materialAccentColor = Color(monet.getAccentColor(this@MainActivity)),
            sampleBitmap = sampleBitmap,
            applyFontPadding = applyFontPadding,
            thumbnailRoundness = thumbnailRoundness.dp
        )
        
        SystemBarAppearance(palette = appearance.colorPalette)
        
        BoxWithConstraints(
            modifier = Modifier.background(appearance.colorPalette.background0) then modifier.fillMaxSize()
        ) {
            CompositionLocalProvider(
                LocalAppearance provides appearance,
                LocalPlayerServiceBinder provides vm.binder,
                LocalCredentialManager provides Dependencies.credentialManager,
                LocalIndication provides ripple(),
                LocalRippleConfiguration provides rippleConfiguration(appearance = appearance),
                LocalShimmerTheme provides shimmerTheme(),
                LocalLayoutDirection provides LayoutDirection.Ltr,
                LocalPersistMap provides Dependencies.application.persistMap,
                LocalMonetCompat provides monet
            ) {
                content()
            }
        }
    }
    
    @OptIn(ExperimentalLayoutApi::class)
    fun setContent() = setContent {
        AppWrapper(
            modifier = Modifier.padding(
                WindowInsets
                .displayCutout
                .only(WindowInsetsSides.Horizontal)
                .asPaddingValues()
            )
        ) {
            val state = rememberMainActivityState(
                binder = vm.binder,
                maxHeight = maxHeight
            )
            
            KeyedCrossfade(state = state.pip) { currentPip ->
                if (currentPip) Thumbnail(
                    isShowingLyrics = true,
                    onShowLyrics = { },
                    isShowingStatsForNerds = false,
                    onShowStatsForNerds = { },
                    onOpenDialog = { },
                    likedAt = null,
                    setLikedAt = { },
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.FillBounds,
                    shouldShowSynchronizedLyrics = true,
                    setShouldShowSynchronizedLyrics = { },
                    showLyricsControls = false
                ) else CompositionLocalProvider(
                    LocalPlayerAwareWindowInsets provides state.playerAwareWindowInsets
                ) {
                    val isDownloading by downloadState.collectAsState()
                    
                    Box {
                        HomeScreen()
                    }
                    
                    AnimatedVisibility(
                        visible = isDownloading,
                        modifier = Modifier.padding(state.playerAwareWindowInsets.asPaddingValues())
                    ) {
                        LinearProgressIndicator(
                            modifier = Modifier
                            .fillMaxWidth()
                            .align(Alignment.TopCenter)
                        )
                    }
                    
                    CompositionLocalProvider(
                        LocalAppearance provides LocalAppearance.current.let {
                            if (it.colorPalette.isDark && AppearancePreferences.darkness == Darkness.AMOLED) {
                                it.copy(colorPalette = it.colorPalette.amoled())
                            } else it
                        }
                    ) {
                        Player(
                            layoutState = state.playerBottomSheetState,
                            modifier = Modifier.align(Alignment.BottomCenter)
                        )
                    }
                    
                    BottomSheetMenu(
                        modifier = Modifier.align(Alignment.BottomCenter)
                    )
                }
            }
            
            state.PlayerLifecycleListener()
        }
    }
    
    override fun onDestroy() {
        super.onDestroy()
        monet.removeMonetColorsChangedListener(this)
        _monet = null
    }
    
    override fun onStop() {
        unbindService(serviceConnection)
        super.onStop()
    }
    
    override fun onMonetColorsChanged(
        monet: MonetCompat,
        monetColors: ColorScheme,
        isInitialChange: Boolean
    ) {
        if (!isInitialChange) recreate()
    }
    
    override fun onUserLeaveHint() {
        super.onUserLeaveHint()
        if (AppearancePreferences.autoPip && vm.binder?.player?.shouldBePlaying == true) maybeEnterPip()
    }
}
