// ====================================================================
// Файл: core/material-compat/build.gradle.kts
// Назначение: Конфигурация Android-модуля для совместимости с Material
// Аудит: ИСПРАВЛЕНО: namespace изменен с com.google.android.material на app.vichord.core.materialcompat.
// УДАЛЕН устаревший sourceSets. ДОБАВЛЕНЫ явные compileSdk и targetSdk.
// ====================================================================

plugins {
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.android.library)
}

android {
    namespace = "app.vichord.core.materialcompat" // ИСПРАВЛЕНО: Установлено уникальное имя
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

dependencies {
    // [ДОБАВЛЕНО] Зависимость для Core Library Desugaring
    coreLibraryDesugaring(libs.desugaring)
    
    implementation(projects.core.ui)

    detektPlugins(libs.detekt.compose)
    detektPlugins(libs.detekt.formatting)
}

kotlin {
    jvmToolchain(libs.versions.jvm.get().toInt())
}
