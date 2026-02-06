mod providers;

uniffi::include_scaffolding!("common"); // Добавлен недостающий UDL
uniffi::include_scaffolding!("translate");
uniffi::include_scaffolding!("sponsorblock");
uniffi::include_scaffolding!("piped");
uniffi::include_scaffolding!("irclib"); // Синхронизировано с опечаткой в build.gradle.kts
uniffi::include_scaffolding!("kugou");
uniffi::include_scaffolding!("innertube");
uniffi::include_scaffolding!("gitgub"); // Синхронизировано с опечаткой в build.gradle.kts

use env_logger;

fn init_logger() {
    env_logger::init();
}

#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub extern "C" fn JNI_OnLoad(vm: jni::JavaVM, _reserved: *mut std::os::raw::c_void) -> jni::sys::jint {
    init_logger();
    jni::sys::JNI_VERSION_1_6
}
