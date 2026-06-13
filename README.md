# Eye Break — a tiny 20·20·20 reminder

A lightweight, cross-platform tray app that nudges you to follow the **20-20-20 rule**:
every 20 minutes, look ~20 ft (6 m) into the distance for 20 seconds.

- Lives entirely in the **system tray / menu bar** — no main window.
- Fires a **system notification**; you confirm you're looking away (which starts the
  break countdown), then a second notification tells you the break is over.
- Both durations are **customizable**, plus optional: start-on-login, pause-when-idle,
  a sound cue, and tray pause/snooze.
- Built with **Tauri 2** (Rust core + a small Svelte settings panel). ~10 MB exe, ~37 MB RAM.

## Using it

- The icon sits in the system tray (Windows 11: under the **`^` "show hidden icons"** chevron).
- **Left-click** the icon → open the settings panel. **Right-click** → menu
  (I'm looking · Pause · Snooze · Settings · Quit).
- When a reminder is showing, confirm via the toast's **"I'm looking"** button, the tray
  **"👀 I'm looking"** item, or a left-click on the tray icon.
- Settings persist automatically to `…/com.eyebreak.app/settings.json` (per-OS app config dir).

## How it works

A 1 Hz background thread in Rust drives the state machine
(`Working → AwaitingConfirm → Break → Working`, plus `Paused`). State is the single source of
truth; the WebView only renders it. See:

- `src-tauri/src/state.rs` — phases + shared state
- `src-tauri/src/timer.rs` — the per-second tick / transitions
- `src-tauri/src/notify.rs` — notifications (Windows toast w/ button + reminder scenario;
  `tauri-plugin-notification` elsewhere)
- `src-tauri/src/tray.rs` — tray icon + menu
- `src-tauri/src/commands.rs` — confirm / pause / snooze / settings, shared by tray + frontend
- `src/routes/+page.svelte` — the settings panel

## Develop / build

```sh
npm install
npm run tauri dev      # run with hot reload (needs the Vite dev server)
npm run tauri build    # standalone eye-break.exe + MSI/NSIS installers in src-tauri/target/release
```

To regenerate the icons from the source artwork: `npm run tauri icon app-icon.svg`.

## Releases / distribution

Installers are built by CI ([`.github/workflows/release.yml`](.github/workflows/release.yml)) on
Windows **and** macOS runners. Cut a release by pushing a version tag:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The workflow builds the Windows `.msi`/NSIS `-setup.exe` and the macOS `.dmg` (both Apple Silicon
and Intel) and attaches them to a **draft GitHub Release** for you to review and publish. Users then
download the installer for their OS — no app store required.

Signing is optional and disabled by default (builds work unsigned):

- **Windows** — unsigned installers trigger a one-time SmartScreen "More info → Run anyway". Add an
  OV/EV code-signing cert to remove it.
- **macOS** — Gatekeeper blocks unsigned apps; an Apple Developer account ($99/yr) lets CI
  codesign + notarize for a clean install. See the commented secrets in the workflow. For an
  unsigned local build, first remove the quarantine flag:
  `xattr -dr com.apple.quarantine "/Applications/Eye Break.app"`.

### macOS notes

- The app is a **menu-bar agent** (`LSUIElement`): it shows **only** in the menu bar — no Dock
  icon and no window on launch. Click the menu-bar eye to open Settings.
- Reminders go through **Notification Center**, so on first run macOS asks for notification
  permission. If you don't see reminders, allow them under **System Settings → Notifications →
  Eye Break** (the app holds in "look away" until you confirm via the menu-bar item).

## Known follow-ups

- **Toast branding (Windows):** notifications currently use the well-known PowerShell AUMID so they
  reliably appear without extra setup; they're therefore attributed to "Windows PowerShell". To
  brand them as *Eye Break*, register the app's own AUMID against its installed Start-Menu shortcut
  and pass it to `ToastManager::new` in `notify.rs`.
- **Auto-update:** add `tauri-plugin-updater` + a signing key (see the workflow comments) so
  installed apps self-update from GitHub Releases.
