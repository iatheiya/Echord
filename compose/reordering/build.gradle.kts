// ====================================================================
// Файл: compose/reordering/build.gradle.kts
// Назначение: Конфигурация Compose-модуля
// Аудит: ДОБАВЛЕНЫ явные compileSdk и targetSdk в defaultConfig. minSdk = 25.
// ====================================================================

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "app.vitune.compose.reordering"
    compileSdk = 36

    defaultConfig {
        compileSdk = 36 // Явно указано
        targetSdk = 36 // Явно указано
        
        // Соответствует цели Android 7.1 (API 25)
        minSdk = 25
    }

    kotlinOptions {
        freeCompilerArgs = freeCompilerArgs + listOf(
            "-Xcontext-receivers",
            "-Xsuppress-warning=CONTEXT_RECEIVERS_DEPRECATED"
        )
    }
}

kotlin {
 
   jvmToolchain(libs.versions.jvm.get().toInt())
}

dependencies {
    // [ДОБАВЛЕНО] Зависимость для Core Library Desugaring
    coreLibraryDesugaring(libs.desugaring)

    implementation(platform(libs.compose.bom))
    implementation(libs.compose.foundation)

    detektPlugins(libs.detekt.compose)
    detektPlugins(libs.detekt.formatting)
}
