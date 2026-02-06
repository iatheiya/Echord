<div align="center">        
    <img src="./app/src/main/ic_launcher-playstore.png" width="128" height="128" style="display: block; margin: 0 auto"/>        
    <h1>ViChord</h1>        
    <p>An Android application for seamless music streaming</p>        
</div>        
        
---        
        
<p align="center">        
  <img src="./fastlane/metadata/android/en-US/images/phoneScreenshots/1.png" width="30%" />        
  <img src="./fastlane/metadata/android/en-US/images/phoneScreenshots/2.png" width="30%" />        
  <img src="./fastlane/metadata/android/en-US/images/phoneScreenshots/3.png" width="30%" />        
        
  <img src="./fastlane/metadata/android/en-US/images/phoneScreenshots/4.png" width="30%" />        
  <img src="./fastlane/metadata/android/en-US/images/phoneScreenshots/5.png" width="30%" />        
  <img src="./fastlane/metadata/android/en-US/images/phoneScreenshots/6.png" width="30%" />        
</p>        
        
## Features        
        
- Play (almost) any audio from multiple sources — mainly YouTube        
- Keep listening in the background or even offline — with cached songs        
- Search through songs, albums, artists, videos, and playlists        
- Import your playlists straight from YouTube        
- View lyrics — fetch, edit, and even sync them        
- Cloud-sync your playlists seamlessly        
- Enjoy an optimized listening experience with audio normalization        
- Android Auto ready        
- Open any YouTube or YouTube Music link (videos, playlists, channels, etc.) right in ViChord        
        
## Installation
      
<p align="center">      
  <a href="https://github.com/25huizengek1/ViChord/releases/latest" style="margin: 15px;">      
    <img src="https://github.com/machiav3lli/oandbackupx/blob/034b226cea5c1b30eb4f6a6f313e4dadcbb0ece4/badge_github.png?raw=true" height="80" style="vertical-align: middle;" alt="Get it on GitHub" />      
  </a>      
  <a href="https://repo.vichord.app/" style="margin: 15px;">      
    <img src="https://fdroid.gitlab.io/artwork/badge/get-it-on.png" height="80" style="vertical-align: middle;" alt="Get it on F-Droid" />      
  </a>      
  <a href="https://apps.obtainium.imranr.dev/redirect?r=obtainium://add/https://github.com/25huizengek1/ViChord/" style="margin: 15px;">      
    <img src="https://github.com/user-attachments/assets/713d71c5-3dec-4ec4-a3f2-8d28d025a9c6" height="80" style="vertical-align: middle;" alt="Get it on Obtainium" />      
  </a>      
</p>
    
## Acknowledgments        
        
- [**YouTube-Internal-Clients**](https://github.com/zerodytrash/YouTube-Internal-Clients): A Python        
  script that discovers hidden YouTube API clients. Just a research project.        
- [**ionicons**](https://github.com/ionic-team/ionicons): Premium hand-crafted icons built by Ionic,        
  for Ionic apps and web apps everywhere.        
- [**Flaticon: Ilham Fitrotul Hayat**](https://www.flaticon.com/authors/ilham-fitrotul-hayat): the        
  app's logo uses a music note icon.        
        
## Disclaimer        
        
This project and its contents are not affiliated with, funded, authorized, endorsed by, or in any        
way associated with YouTube, Google LLC or any of its affiliates and subsidiaries.        
        
Any trademark, service mark, trade name, or other intellectual property rights used in this project        
are owned by the respective owners.

## Rust Integration Guide

### Installing NDK
To integrate Rust with your Android project, first install the Android Native Development Kit (NDK). You can download it from the [Android Studio SDK Manager](https://developer.android.com/ndk/downloads) or via the command line using SDK tools.

### Adding Rustup Targets
Install the necessary Rust targets for Android architectures using rustup:

```
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

### Installing cargo-ndk
Install the `cargo-ndk` tool to simplify building Rust code for Android:

```
cargo install cargo-ndk
```

### Build Steps
1. Build your Rust project for Android using `cargo ndk`:

   ```
   cargo ndk build --release
   ```

2. Copy the generated `.so` files from the `target/<arch>/release/` directories to your Android project's `jniLibs` folder, organized by architecture (e.g., `jniLibs/arm64-v8a/`, `jniLibs/armeabi-v7a/`, etc.).