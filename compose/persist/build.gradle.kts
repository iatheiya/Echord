// ====================================================================
// Файл: compose/persist/build.gradle.kts
// Назначение: Конфигурация Compose-модуля для сохранения состояния (persist)
// Аудит: УДАЛЕН устаревший sourceSets. ДОБАВЛЕНЫ явные compileSdk и targetSdk.
// ====================================================================

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "app.vitune.compose.persist"
    compileSdk = 36

    defaultConfig {
        compileSdk = 36 // Явно указано
        targetSdk = 36 // Явно указано
        // Соответствует цели Android 7.1 (API 25)
        minSdk = 25
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

    implementation(libs.kotlin.immutable)

    detektPlugins(libs.detekt.compose)
    detektPlugins(libs.detekt.formatting)
}
