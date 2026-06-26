# Node-IDE

Visual node-edge workflow IDE (v1 walking skeleton).

## Development

```bash
npm install
npm run tauri dev
```

## CLI (auxiliary)

```bash
cargo run --manifest-path src-tauri/Cargo.toml -- --cli run --kind constant --value "hello"
cargo run --manifest-path src-tauri/Cargo.toml -- --cli state
cargo run --manifest-path src-tauri/Cargo.toml -- --cli migrate
```

## Persistence

Graph state is saved to:

- macOS: `~/Library/Application Support/dev.nodeide/project.json`

## Updater signing

Generate a signing keypair for in-app updates:

```bash
npm run tauri signer generate -- -w ~/.tauri/node-ide.key
```

Set the public key in `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`) and store the private key in GitHub Actions secrets as `TAURI_SIGNING_PRIVATE_KEY`.

Update the GitHub endpoint in `src-tauri/tauri.conf.json` to your repository:

```
https://github.com/<OWNER>/<REPO>/releases/latest/download/latest.json
```

## v1 dogfooding checklist

1. Place Constant or Echo node on canvas and Run
2. Quit and relaunch — graph persists in app data dir
3. Tag `v0.1.0` and `v0.1.1` releases to validate updater path
4. Run CLI commands for headless regression
