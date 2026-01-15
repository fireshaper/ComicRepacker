# ComicRepacker

Easily scan directories containing CBR files to find any that have been packed with RAR5

ComicRepacker is a cross-platform desktop application built with Tauri, React, and TypeScript designed to help comic, manga, and graphic novel enthusiasts manage their digital collections.

## Features

-   **Recursive Scanning**: Scans selected directories for comic archives (`.cbr`, `.cbz`, `.rar`, `.zip`).
-   **Intelligent Analysis**: Detects archive formats, including distinguishing between RAR4 and RAR5.
-   **Komga Compatibility**: Identifies files that might be problematic for Komga (e.g., solid compressed or RAR5 archives).
-   **One-Click Conversion**: Easily converts unsupported formats to widely compatible `.cbz` (ZIP) files.
-   **Modern UI**: Clean interface with filtering and batch processing capabilities.

## Prerequisites

Before building the application, ensure you have the following installed:

-   [Node.js](https://nodejs.org/) (v16 or newer recommended)
-   [Rust](https://www.rust-lang.org/tools/install) (latest stable)

### Platform-Specific Dependencies

#### Linux
You will need to install libraries for WebKitGTK and other system dependencies. On Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```
For other distributions, see the [Tauri Linux Setup Guide](https://tauri.app/v1/guides/getting-started/prerequisites#linux).

#### macOS
No extra dependencies are usually required if you have Xcode Command Line Tools installed:
```bash
xcode-select --install
```

#### Windows
-   **Microsoft Visual Studio C++ Build Tools** (Select "Desktop development with C++")
-   **WebView2** (Usually pre-installed on Windows 10/11)

## Build Instructions

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/fireshaper/ComicRepacker.git
    cd ComicRepacker
    ```


2.  **Setup 7-Zip:**
    Download the 7-Zip bundle for your OS.
    -   **Linux/macOS:** 
        1. Download the `7zip tar.gz` bundle.
        2. Extract the bundle to a location of your choice.

    -   **Windows:**
        1.  Download and install the **latest 64-bit 7-Zip installer** (`.exe`).
        2.  Go to the install location (usually `C:\Program Files\7-Zip\`).
        3.  Copy `7z.exe` AND `7z.dll`.

    Copy the files(s) to the `src-tauri/bin/` directory and **rename** it to match the target triple:
    -   **Linux:** `7zz-x86_64-pc-linux-gnu`
    -   **macOS:** `7zz-x86_64-apple-darwin`
    -   **Windows:** `7zz-x86_64-pc-windows-msvc.exe` (Rename `7z.exe` to this)

    > **CRITICAL for Windows:** You **MUST** copy `7z.dll` into the `src-tauri/bin/` folder alongside the renamed executable. `7z.exe` depends on this DLL to support RAR files; without it, it will silently fail to recognize them.

3.  **Install project dependencies:**

    ```bash
    npm install
    ```

4.  **Run in Development Mode:**
    This command will start the frontend dev server and the Tauri backend window.
    ```bash
    npm run tauri dev
    ```

5.  **Build for Production:**
    To create an optimized executable/installer for your current OS:
    ```bash
    npm run tauri build
    ```
    The output binaries will be located in:
    -   **Linux:** `src-tauri/target/release/bundle/deb/` (or AppImage)
    -   **macOS:** `src-tauri/target/release/bundle/dmg/` (or .app)
    -   **Windows:** `src-tauri/target/release/bundle/msi/`

## Technology Stack

-   **Frontend**: React, TypeScript, Vite, Tailwind CSS (optional), LucideReact Icons
-   **Backend**: Rust (Tauri)
-   **Utilities**: `7-Zip` (bundled) for archive operations

## License

[MIT](LICENSE)
