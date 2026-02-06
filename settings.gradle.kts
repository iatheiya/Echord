@file:Suppress("UnstableApiUsage")

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)

    repositories {
        google()
        mavenCentral()
        maven("https://jitpack.io")
    }
}

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "0.10.0"
}

rootProject.name = "ViChord"

include(
    ":core:data",
    ":core:ui",
    ":compose:routing",
    ":compose:reordering",
    ":compose:preferences",
    ":compose:persist",
    ":app",
    ":rust",
    ":ktor-client-brotli"
)

project(":rust").projectDir = file("rust")