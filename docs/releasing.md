# Release procedure

## Local build

1. Run the automated checks in [development.md](development.md).
2. Complete the affected native checks in `docs/macos-testing.md`.
3. Run `npm run app:build`.
4. Open the generated app bundle.
5. Make sure that its controls load without a Vite server.

The build output matches the current Mac architecture. An Apple Silicon build does not run on an Intel Mac.
An Intel build can run on Apple Silicon through Rosetta.

For a universal build:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm run tauri -- build --target universal-apple-darwin
```

## Signing and notarization

Distribution outside the local Mac requires an Apple Developer account and a Developer ID Application certificate.
Tauri reads signing and notarization credentials from environment variables.

1. Set `APPLE_SIGNING_IDENTITY` to the Developer ID Application identity.
2. Set `APPLE_ID` to the Apple account email.
3. Set `APPLE_PASSWORD` to an app-specific password.
4. Set `APPLE_TEAM_ID` to the developer team identifier.
5. Run the release build.
6. Make sure that notarization succeeds before distribution.

Keep credentials outside the repository. The [Tauri macOS signing guide](https://v2.tauri.app/distribute/sign/macos/) describes other credential methods.

For a signed build, run these checks against its actual path:

```sh
codesign --verify --deep --strict --verbose=2 src-tauri/target/release/bundle/macos/Kroneum.app
spctl --assess --type execute --verbose=2 src-tauri/target/release/bundle/macos/Kroneum.app
```

## Assets

The repository includes generated assets. Regeneration uses Node.js and the macOS `iconutil` command:

```sh
npm run assets
```

The generator uses the shared vector artwork and resvg to create the app icons at every native size.
It also creates the monochrome menu-bar template and a four-second WAV chime.
The chime is original synthesized audio. The other sounds come from the local macOS installation.
See [branding.md](branding.md) for the source artwork, generated variants, and visual review procedure.
