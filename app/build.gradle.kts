import org.jetbrains.kotlin.compose.compiler.gradle.ComposeFeatureFlag
import org.jetbrains.kotlin.gradle.dsl.KotlinVersion

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.ksp)
    alias(libs.plugins.chaquo)
}

android {
    val appId = "${project.group}.android"

    namespace = appId
    compileSdk = 35
    
    ndkVersion = "29.0.14206865"

    defaultConfig {
        applicationId = appId

        minSdk = 25
        targetSdk = 35

        versionCode = System.getenv("ANDROID_VERSION_CODE")?.toIntOrNull() ?: 17
        versionName = project.version.toString()

        multiDexEnabled = true

        ndk {
            abiFilters.clear()
            abiFilters.addAll(listOf("arm64-v8a", "armeabi-v7a", "x86", "x86_64"))
        }
    }

    splits {
        abi {
            reset()
            isUniversalApk = true
        }
    }

    signingConfigs {
        create("ci") {
            storeFile = System.getenv("ANDROID_NIGHTLY_KEYSTORE")?.let { file(it) }
            storePassword = System.getenv("ANDROID_NIGHTLY_KEYSTORE_PASSWORD")
   
            keyAlias = System.getenv("ANDROID_NIGHTLY_KEYSTORE_ALIAS")
            keyPassword = System.getenv("ANDROID_NIGHTLY_KEYSTORE_PASSWORD")
        }
    }

    buildTypes {
        debug {
            applicationIdSuffix = ".debug"
            versionNameSuffix = "-DEBUG"
            manifestPlaceholders["appName"] = "ViChord Debug"
        }

        release {
            versionNameSuffix = "-RELEASE"
            isMinifyEnabled = true
            isShrinkResources = true
            manifestPlaceholders["appName"] = "ViChord"
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }

        create("nightly") {
            initWith(getByName("release"))
            matchingFallbacks += "release"

            applicationIdSuffix = ".nightly"
            versionNameSuffix = "-NIGHTLY"
      
            manifestPlaceholders["appName"] = "ViChord Nightly"
            signingConfig = signingConfigs.findByName("ci")
        }
    }

    buildFeatures {
        buildConfig = true
    }

    compileOptions {
        isCoreLibraryDesugaringEnabled = true
    }

    packaging {
        resources.excludes.add("META-INF/**/*")
    }

    androidResources {
        @Suppress("UnstableApiUsage")
        generateLocaleConfig = true
    }

    lint {
        checkAllWarnings = true
        disable.add("TypographyDashes")
        disable.add("TypographyQuotes")
    }

    testOptions {
        unitTests.all {
            useJUnitPlatform()
        }
    }
}

kotlin {
    jvmToolchain(libs.versions.jvm.get().toInt())

    compilerOptions {
        languageVersion.set(KotlinVersion.KOTLIN_2_2_21)

        freeCompilerArgs.addAll(
            "-Xcontext-receivers",
            "-Xnon-local-break-continue",
            "-Xconsistent-data-class-copy-visibility",
            "-Xsuppress-warning=CONTEXT_RECEIVERS_DEPRECATED"
        )
    }
}

ksp {
    arg("room.schemaLocation", "$projectDir/schemas")
}

composeCompiler {
    featureFlags = setOf(
        ComposeFeatureFlag.OptimizeNonSkippingGroups
    )

    if (project.findProperty("enableComposeCompilerReports") == "true") {
        val dest = layout.buildDirectory.dir("compose_metrics")
        metricsDestination = dest
        reportsDestination = dest
    }
}

chaquopy {
    defaultConfig {
        version = "3.13"
        pip {
            install("yt-dlp")
        }
    }
}

dependencies {
    coreLibraryDesugaring(libs.desugaring)

    implementation(projects.compose.persist)
    implementation(projects.compose.preferences)
    implementation(projects.compose.routing)
    implementation(projects.compose.reordering)

    implementation(fileTree(projectDir.resolve("vendor")))

    implementation(platform(libs.compose.bom))
    implementation(libs.compose.activity)
    implementation(libs.compose.foundation)
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.util)
    implementation(libs.compose.shimmer)
    implementation(libs.compose.lottie)
    implementation(libs.compose.material3)

    implementation(libs.coil.compose)
    implementation(libs.coil.ktor)

    implementation(libs.palette)
    implementation(libs.monet)
    runtimeOnly(projects.core.materialCompat)

    implementation(libs.exoplayer)
    implementation(libs.exoplayer.workmanager)
    implementation(libs.media3.session)
    implementation(libs.media)

    implementation(libs.workmanager)
    implementation(libs.workmanager.ktx)

    implementation(libs.credentials)
    implementation(libs.credentials.play)

    implementation(libs.kotlin.coroutines)
    implementation(libs.kotlin.immutable)
    implementation(libs.kotlin.datetime)

    implementation(libs.room)
    ksp(libs.room.compiler)

    implementation(libs.log4j)
    implementation(libs.slf4j)
    implementation(libs.logback)
    
    implementation(libs.jna)

    implementation(projects.providers.github)
    implementation(projects.providers.innertube)
    implementation(projects.providers.kugou)
    implementation(projects.providers.lrclib)
    implementation(projects.providers.piped)
    implementation(projects.providers.sponsorblock)
    implementation(projects.providers.translate)
    implementation(projects.core.data)
    implementation(projects.core.ui)

    detektPlugins(libs.detekt.compose)
    detektPlugins(libs.detekt.formatting)
}

// Rust tasks
val buildRust = tasks.register<Exec>("buildRust") {
    group = "rust"
    description = "Build Rust for ABIs."
    val rustDir = rootProject.file("rust")
    if (!rustDir.exists()) {
        throw GradleException("Rust dir missing: ${rustDir.absolutePath}")
    }
    workingDir = rustDir
    
    commandLine(
        "cargo", "ndk",
        "-t", "armeabi-v7a",
        "-t", "arm64-v8a",
        "-t", "x86",
        "-t", "x86_64",
        "build", "--release"
    )
}

val copyRustLibs = tasks.register("copyRustLibs") {
    group = "rust"
    description = "Copy .so to jniLibs."
    dependsOn(buildRust)

    val rustTargetDir = rootProject.layout.projectDirectory.dir("rust/target")
    val destJniLibsDir = layout.projectDirectory.dir("src/main/jniLibs")

    val cargoFile = rootProject.file("rust/Cargo.toml")
    val libName = try {
        val cargoToml = cargoFile.readText(Charsets.UTF_8)
        cargoToml.find("""name\s*=\s*["']([^"']+)["']""".toRegex())?.groupValues?.get(1)?.let { "lib$it.so" } ?: "librust.so"
    } catch (e: Exception) {
        println("WARN: Cargo.toml parse fail: ${e.message}; fallback librust.so")
        "librust.so"
    }
    
    val targets = mapOf(
        "armv7-linux-androideabi" to "armeabi-v7a",
        "aarch64-linux-android" to "arm64-v8a",
        "i686-linux-android" to "x86",
        "x86_64-linux-android" to "x86_64"
    )

    doLast {
        targets.forEach { (target, abi) ->
            val source = rustTargetDir.file("$target/release/$libName").asFile
            val dest = destJniLibsDir.dir(abi).file(libName).asFile

            if (source.exists()) {
                println("Copy: $target -> $abi")
                dest.parentFile.mkdirs()
                source.copyTo(dest, overwrite = true)
                
                if (dest.length() < 50_000) {
                    throw GradleException("Suspicious .so size: \( {dest.name} ( \){dest.length()} bytes)")
                }
            } else {
                println("WARN: Missing $target/$libName")
            }
        }
    }
}

tasks.named("preBuild") {
    dependsOn(copyRustLibs)
}

tasks.register<Exec>("validateManifestLint") {
    group = "test"
    description = "Full lint."
    commandLine("./gradlew", "lint")
}

tasks.register("verifyVersions") {
    group = "verification"
    description = "Verify SDK/NDK versions"
    doLast {
        val required = mapOf("compileSdk" to 35, "targetSdk" to 35, "ndkVersion" to "29.0.14206865")
        required.forEach { (k, v) ->
            val actual = android."$k".toString().trim()
            if (actual != v.toString()) {
                throw GradleException("$k mismatch: expected $v, got $actual")
            }
        }
        println("Versions verified OK")
    }
}

preBuild.dependsOn(verifyVersions)

// ProGuard for Media3 (add to proguard-rules.pro): -keep class androidx.media3.** { *; }