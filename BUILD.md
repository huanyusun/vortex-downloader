# Build Instructions

This document provides instructions for building and packaging the YouTube Downloader application.

## Prerequisites

### Required Dependencies

1. **Node.js** (v16 or later)
   ```bash
   node --version
   ```

2. **Rust** (latest stable)
   ```bash
   rustc --version
   cargo --version
   ```

3. **Tauri CLI**
   ```bash
   npm install -g @tauri-apps/cli
   # or
   cargo install tauri-cli
   ```

4. **yt-dlp** (for runtime functionality)
   ```bash
   brew install yt-dlp
   ```

5. **ffmpeg** (required by yt-dlp)
   ```bash
   brew install ffmpeg
   ```

### macOS-Specific Requirements

For code signing and notarization (optional, for distribution):

1. **Apple Developer Account**
2. **Xcode Command Line Tools**
   ```bash
   xcode-select --install
   ```

## Development Build

### Running in Development Mode

```bash
# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

This will:
- Start the Vite development server
- Build the Rust backend
- Launch the application with hot-reload enabled

## Production Build

### Building for macOS

```bash
# Install dependencies
npm install

# Build the application
npm run tauri build
```

This will create:
- `.app` bundle in `src-tauri/target/release/bundle/macos/`
- `.dmg` installer in `src-tauri/target/release/bundle/dmg/`

### Build Targets

For universal macOS binary (Intel + Apple Silicon):

```bash
npm run tauri build -- --target universal-apple-darwin
```

For specific architectures:

```bash
# Intel only
npm run tauri build -- --target x86_64-apple-darwin

# Apple Silicon only
npm run tauri build -- --target aarch64-apple-darwin
```

## Code Signing (Optional)

### Setting Up Code Signing

1. **Get your signing identity:**
   ```bash
   security find-identity -v -p codesigning
   ```

2. **Update `tauri.conf.json`:**
   ```json
   {
     "tauri": {
       "bundle": {
         "macOS": {
           "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)"
         }
       }
     }
   }
   ```

3. **Build with signing:**
   ```bash
   npm run tauri build
   ```

### Notarization

For distribution outside the Mac App Store, you need to notarize your app:

1. **Set environment variables:**
   ```bash
   export APPLE_ID="your-apple-id@example.com"
   export APPLE_PASSWORD="app-specific-password"
   export APPLE_TEAM_ID="YOUR_TEAM_ID"
   ```

2. **Build and notarize:**
   ```bash
   npm run tauri build
   ```

Tauri will automatically notarize if the environment variables are set.

## Build Configuration

### Key Configuration Files

1. **`tauri.conf.json`** - Main Tauri configuration
   - Application metadata
   - Bundle settings
   - Window configuration
   - Security policies

2. **`Cargo.toml`** - Rust dependencies and metadata

3. **`package.json`** - Frontend dependencies and scripts

### Customizing the Build

#### Application Metadata

Edit `tauri.conf.json`:

```json
{
  "package": {
    "productName": "YouTube Downloader",
    "version": "0.1.0"
  },
  "tauri": {
    "bundle": {
      "identifier": "com.youtube-downloader.app",
      "category": "Utility",
      "copyright": "Copyright © 2024. All rights reserved."
    }
  }
}
```

#### DMG Appearance

Customize the DMG installer appearance in `tauri.conf.json`:

```json
{
  "tauri": {
    "bundle": {
      "dmg": {
        "windowSize": {
          "width": 600,
          "height": 400
        },
        "appPosition": {
          "x": 180,
          "y": 170
        },
        "applicationFolderPosition": {
          "x": 420,
          "y": 170
        }
      }
    }
  }
}
```

## Troubleshooting

### Common Issues

1. **"yt-dlp not found" error**
   - Ensure yt-dlp is installed: `brew install yt-dlp`
   - Check PATH: `which yt-dlp`

2. **Build fails with Rust errors**
   - Update Rust: `rustup update`
   - Clean build: `cargo clean` in `src-tauri/`

3. **Frontend build fails**
   - Clear node_modules: `rm -rf node_modules && npm install`
   - Clear cache: `npm cache clean --force`

4. **Code signing fails**
   - Verify signing identity: `security find-identity -v -p codesigning`
   - Check Xcode is installed: `xcode-select -p`

### Build Artifacts

After a successful build, you'll find:

```
src-tauri/target/release/bundle/
├── macos/
│   └── YouTube Downloader.app
└── dmg/
    └── YouTube Downloader_0.1.0_universal.dmg
```

## Distribution

### For Testing

Share the `.dmg` file directly. Users will need to:
1. Open the DMG
2. Drag the app to Applications
3. Right-click and select "Open" (first time only, due to Gatekeeper)

### For Production

1. **Code sign** the application
2. **Notarize** with Apple
3. **Staple** the notarization ticket:
   ```bash
   xcrun stapler staple "YouTube Downloader.app"
   ```
4. Create a new DMG with the stapled app

## Performance Optimization

### Build Optimizations

The release build includes:
- Dead code elimination
- Link-time optimization (LTO)
- Minimal binary size
- Optimized dependencies

### Runtime Performance

The application includes:
- Progress update throttling (500ms)
- Metadata caching (5-minute TTL)
- Efficient queue management
- Async I/O operations

## Version Management

Update version in both:

1. **`package.json`:**
   ```json
   {
     "version": "0.1.0"
   }
   ```

2. **`tauri.conf.json`:**
   ```json
   {
     "package": {
       "version": "0.1.0"
     }
   }
   ```

3. **`src-tauri/Cargo.toml`:**
   ```toml
   [package]
   version = "0.1.0"
   ```

## Additional Resources

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Tauri Bundle Configuration](https://tauri.app/v1/api/config#bundleconfig)
- [macOS Code Signing Guide](https://tauri.app/v1/guides/distribution/sign-macos)
- [Apple Notarization Guide](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
