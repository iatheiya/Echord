// ====================================================================
// Файл: ktor-client-brotli/build.gradle.kts
// Назначение: Конфигурация JVM-модуля для Ktor Content Encoding (Brotli)
// Аудит: УДАЛЕН android.lint.
// ====================================================================

plugins {
    // Чистая JVM-библиотека (не Android-модуль)
    alias(libs.plugins.kotlin.jvm)
}

dependencies {
    // [FIX] Добавлена зависимость, требуемая Ktor
    implementation(libs.ktor.client.encoding)
    // [FIX] Добавлена зависимость org.brotli:dec
    implementation(libs.brotli)

    detektPlugins(libs.detekt.compose)
    detektPlugins(libs.detekt.formatting)
}

kotlin {
    // jvmToolchain установлен на JDK 17 (синхронизировано с libs.versions.toml)
    jvmToolchain(libs.versions.jvm.get().toInt())
}
