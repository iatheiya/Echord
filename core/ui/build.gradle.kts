// ====================================================================
// Файл: core/ui/build.gradle.kts
// Назначение: Конфигурация Android-модуля для основных Compose UI компонентов
// Аудит: УДАЛЕН устаревший sourceSets. ДОБАВЛЕНЫ явные SDK и coreLibraryDesugaring.
// ====================================================================

plugins {
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.parcelize)
}

android {
    namespace = "app.vitune.core.ui"
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
    implementation(projects.core.data)

    // [ДОБАВЛЕНО] Зависимость для Core Library Desugaring
    coreLibraryDesugaring(libs.desugaring)

    implementation(libs.core.ktx)

    implementation(platform(libs.compose.bom))
    implementation(libs.compose.animation)
    implementation(libs.compose.foundation)
    implementation(libs.compose.shimmer)
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.util)
    implementation(libs.compose.ui.fonts)
    implementation(libs.compose.material3)
    implementation(libs.palette)

    detektPlugins(libs.detekt.compose)
    detektPlugins(libs.detekt.formatting)
}

kotlin {
    jvmToolchain(libs.versions.jvm.get().toInt())
}
