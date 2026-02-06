// ====================================================================
// Файл: compose/routing/build.gradle.kts
// Назначение: Конфигурация Compose-модуля для маршрутизации (routing)
// Аудит: ДОБАВЛЕНЫ явные compileSdk и targetSdk.
// ====================================================================

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.parcelize)
}

android {
    namespace = "app.vitune.compose.routing"
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
    implementation(libs.compose.activity)
    implementation(libs.compose.foundation)
    implementation(libs.compose.animation)

    detektPlugins(libs.detekt.compose)
    detektPlugins(libs.detekt.formatting)
}
