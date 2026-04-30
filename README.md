<div align="center">
  <img src="assets/LoonixTunes.png" width="400" alt="Loonix-Tunes Logo">
  <h1>Loonix-Tunes</h1>
  <p><b>A high-performance, native Windows audio player crafted for uncompromising sound quality.</b></p>
  <p>Built with Rust & Qt/QML</p>
</div>

---

## 📸 Screenshot Showroom
| | |
|:---:|:---:|
| ![Main UI](SS/1.png) | ![Playlist View](SS/2.png) |
| ![DSP Controls](SS/3.png) | ![Compact Mode](SS/4.png) |


## 🎵 About The Project
Loonix-Tunes is an elegant, fast, and feature-rich audio player designed for Windows users who care about audio processing and system efficiency. By combining a blazingly fast Rust audio engine with a beautiful, hardware-accelerated Qt interface, Loonix-Tunes delivers a premium listening experience without bloating your system.

## ✨ Key Features
* **Native & Efficient:** Compiled to machine code. Snappy UI, minimal RAM footprint, and instant startup times.
* **Advanced DSP Suite:** Built-in studio-grade effects including Crystalizer, Bass Booster, Compressor, Reverb, and a robust Equalizer.
* **High-Res Audio Engine:** Custom pipeline handling pristine audio decoding, resampling, and gapless playback.
* **Time & Pitch Shifting:** Powered by the industry-standard Rubberband library for high-quality audio stretching.
* **Windows Integration:** Native Windows support with media controls and system integration.

## 🚀 Installation
Head over to the [Releases](https://github.com/citzeye/loonix-tunes-windows/releases) page to download the latest version for Windows.

We provide two simple ways to run Loonix-Tunes on Windows:
1. **Installer:** Download the `.exe` installer and follow the installation wizard.
2. **Portable:** Download the `.zip` file, extract it, and run `LoonixTunesWin64v2.exe` directly. No installation required.


## 🛠️ Building from Source

If you prefer to compile Loonix-Tunes yourself, ensure you have the following installed on your Windows machine:
- Rust toolchain (rustup)
- Qt 6.8+ (MSVC 2022)
- FFmpeg development libraries
- Inno Setup 6 (for building the installer)

```bash
git clone https://github.com/citzeye/loonix-tunes-windows.git
cd loonix-tunes-windows
cd packaging/windows
.\deploy.ps1
```

The build script will:
1. Compile the release binary
2. Deploy Qt runtime and QML files
3. Copy FFmpeg DLLs
4. Create a portable ZIP (`LoonixTunesWin64v2.zip`)
5. Build the installer (`LoonixTunesWin64v2.exe`)

🤝 Issues and feature requests are welcome! Feel free to check the [issues page](https://github.com/citzeye/loonix-tunes-windows/issues).

Created with ❤️ by citzeye