# ====================================================================
# Kotlin Coroutines
# ====================================================================
-keep,allowobfuscation,allowshrinking class kotlin.coroutines.Continuation

# ====================================================================
# Kotlin Serialization
# Сохраняем сериализаторы от удаления R8
# ====================================================================
#noinspection ShrinkerUnresolvedReference
-if @kotlinx.serialization.Serializable class **
-keepclassmembers class <1> {
    static <1>$Companion Companion;
}

-if @kotlinx.serialization.Serializable class ** {
    static **$* *;
}
-keepclassmembers class <2>$<3> {
    #noinspection ShrinkerUnresolvedReference
    kotlinx.serialization.KSerializer serializer(...);
}

-if @kotlinx.serialization.Serializable class ** {
    public static ** INSTANCE;
}
-keepclassmembers class <1> {
    public static <1> INSTANCE;
    #noinspection ShrinkerUnresolvedReference
    kotlinx.serialization.KSerializer serializer(...);
}

# ====================================================================
# UniFFI / Rust / JNA
# Критически важно для работы Native-моста
# ====================================================================

# 1. Пользовательские обертки и пространство имен
# Исправлено: { *; } вместо { ; }
-keep class app.vichord.** { *; }

# 2. Внутренние классы UniFFI
-keep class uniffi.** { *; }

# 3. JNA (Java Native Access)
# UniFFI использует JNA для вызова Rust кода. R8 не видит рефлексию JNA
# и удаляет классы, что приводит к крашу "Structure definition not found".
-dontwarn java.awt.*
-dontwarn com.sun.jna.**
-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { *; }

# ====================================================================
# Android Credentials & Play Services
# ====================================================================
-if class androidx.credentials.CredentialManager
-keep class androidx.credentials.playservices.** {
  *;
}

# ====================================================================
# Security & Crypto (BouncyCastle, Conscrypt)
# ====================================================================
-keepattributes RuntimeVisibleAnnotations,AnnotationDefault

-dontwarn org.bouncycastle.jsse.BCSSLParameters
-dontwarn org.bouncycastle.jsse.BCSSLSocket
-dontwarn org.bouncycastle.jsse.provider.BouncyCastleJsseProvider
-dontwarn org.conscrypt.Conscrypt$Version
-dontwarn org.conscrypt.Conscrypt
-dontwarn org.conscrypt.ConscryptHostnameVerifier
-dontwarn org.openjsse.javax.net.ssl.SSLParameters
-dontwarn org.openjsse.javax.net.ssl.SSLSocket
-dontwarn org.openjsse.net.ssl.OpenJSSE
-dontwarn org.slf4j.impl.StaticLoggerBinder

# ====================================================================
# Rhino (JavaScript Engine)
# ====================================================================
-keep class org.mozilla.javascript.** { *; }
-keep class org.mozilla.classfile.ClassFileWriter
-dontwarn jdk.dynalink.**
-dontwarn org.mozilla.javascript.JavaToJSONConverters
-dontwarn org.mozilla.javascript.tools.**
