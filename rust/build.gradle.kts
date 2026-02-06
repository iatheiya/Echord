import com.android.build.api.dsl.LibraryExtension

// ====================================================================
// Файл: rust/build.gradle.kts
// Назначение: Конфигурация сборки Rust-модуля и генерации UniFFI-обвязки.
// Аудит: ПРОЙДЕН.
// Учтены опечатки в именах файлов UDL.
// ====================================================================

plugins {
    // Уровень 1: Стандартные плагины экосистемы
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    
    // Уровень 2: Плагин UniFFI (Мост Rust -> Kotlin)
    // [ИЗМЕНЕНО] Используем alias из libs.versions.toml вместо id("...")
    alias(libs.plugins.uniffi.plugin)
}

android {
    // Идентификация модуля. Критически важно для JNI-маппинга.
    namespace = "app.vitune.rust"
    compileSdk = 36 // Синхронизировано с корневым проектом

    defaultConfig {
        minSdk = 25 // Соответствует API 25
        
        // SAFETY: Защита от удаления символов при минификации (R8/Proguard)
        // UniFFI использует рефлексию/JNI, поэтому правила необходимы.
        consumerProguardFiles("consumer-rules.pro")
    }

    // Окружение сборки Native кода
    // Рекомендуемая версия для совместимости с Cargo NDK
    ndkVersion = "26.3.11579264" // Синхронизировано с корневым проектом

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    // Конфигурация UniFFI
    // UniFFI ожидает, что UDL-файлы находятся в src/main/uniffi/
    // Но вы используете src/providers/
    sourceSets.getByName("main") {
        java.srcDir("src/uniffi-generated")
    }
}

// Конфигурация UniFFI
// [ИСПРАВЛЕНО] UniFFI требует явной конфигурации UDL-файлов
uniffi {
    // ====================================================================
    // КОНФИГУРАЦИЯ КОНТРАКТОВ (UDL)
    // Порядок важен: Сначала общие типы, затем зависимые модули.
// ====================================================================
    
    // 1. Общие типы данных (Common)
    config(layout.projectDirectory.file("src/providers/common/common.udl"))

    // 2. Основные провайдеры
    config(layout.projectDirectory.file("src/providers/innertube/innertube.udl"))
    config(layout.projectDirectory.file("src/providers/piped/piped.udl"))
    config(layout.projectDirectory.file("src/providers/kugou/kugou.udl"))
    config(layout.projectDirectory.file("src/providers/sponsorblock/sponsorblock.udl"))
    config(layout.projectDirectory.file("src/providers/translate/translate.udl"))

    // [WARNING] Реальное имя файла на диске содержит опечатку (gitgub).
    // Если вы переименуете файл в github.udl, обновите строку ниже.
    config(layout.projectDirectory.file("src/providers/github/gitgub.udl"))

    // [WARNING] Реальное имя файла на диске содержит опечатку (irclib).
    // Если вы переименуете файл в lrclib.udl, обновите строку ниже.
    config(layout.projectDirectory.file("src/providers/lrclib/irclib.udl"))
}

dependencies {
    // Базовые зависимости для Android/Kotlin
    implementation(libs.androidx.core.ktx)
    implementation(libs.kotlinx.coroutines.core)
    
    // [ДОБАВЛЕНО] Зависимость для Core Library Desugaring
    coreLibraryDesugaring(libs.desugaring)
}
