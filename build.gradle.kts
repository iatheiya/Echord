plugins {
    alias(libs.plugins.kotlin.jvm) apply false
    alias(libs.plugins.kotlin.serialization) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.kotlin.compose) apply false
    alias(libs.plugins.kotlin.parcelize) apply false
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.android.lint) apply false
    alias(libs.plugins.ksp) apply false
    alias(libs.plugins.chaquo) apply false

    id("io.gitlab.arturbosch.detekt") version "1.23.8"
    id("com.diffplug.spotless") version "8.1.0"
}

import io.gitlab.arturbosch.detekt.Detekt
import org.gradle.api.tasks.Delete

// Сначала регистрируем кастомную задачу, чтобы она была доступна ниже
val cleanRust by tasks.registering(Delete::class) {
    delete(file("rust/target"))
}

// Исправление: используем 'named' вместо 'registering', так как задача clean уже существует
tasks.named<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
    dependsOn(cleanRust)
}

subprojects {
    group = "app.vichord"
    version = "1.0.0"

    apply(plugin = "com.diffplug.spotless")

    tasks.withType<Detekt>().configureEach {
        description = "Run detekt for ${project.name}"
        group = "verification"

        buildUponDefaultConfig = true
        allRules = false
        config.setFrom(file("$rootDir/detekt.yml"))

        // Проверка на null для безопасности, если версия не определена
        jvmTarget = libs.versions.jvm.orNull ?: "17"

        parallel = true

        reports {
            html.required.set(true)
            html.outputLocation.set(layout.buildDirectory.file("reports/detekt/${project.name}.html"))
            xml.required.set(false)
            txt.required.set(false)
        }
    }

    spotless {
        kotlin {
            target("**/*.kt")
            ktlint("0.50.0")
            licenseHeaderFile(rootProject.file("spotless-license.txt"))
        }
    }
}

// CI stub: Add .github/workflows/ci.yml: on: push; steps: - ./gradlew detekt spotlessCheck
