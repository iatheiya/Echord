// ====================================================================
// Файл: core/data/build.gradle.kts
// Назначение: Конфигурация Android-модуля для работы с данными
// Аудит: УДАЛЕНЫ устаревшие sourceSets и избыточные compilerOptions. ДОБАВЛЕНЫ явные SDK.
// ====================================================================

plugins {
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.android.library)
}

android {
    namespace = "app.vitune.core.data"
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

    compilerOptions {
        // Оставлены только актуальные флаги
        freeCompilerArgs.addAll("-Xcontext-receivers", "-Xsuppress-warning=CONTEXT_RECEIVERS_DEPRECATED")
    }
}

dependencies {
    implementation(libs.core.ktx)
    
    // [ДОБАВЛЕНО] Зависимость для Core Library Desugaring
    coreLibraryDesugaring(libs.desugaring)
    
    implementation(libs.jna)

    api(libs.kotlin.datetime)

    detektPlugins(libs.detekt.compose)
    detektPlugins(libs.detekt.formatting)
}
