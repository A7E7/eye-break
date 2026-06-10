<script lang="ts">
  import { onMount } from "svelte";
  import { core } from "$lib/core";
  import type { Settings, Snapshot } from "$lib/types";

  const isWeb = core.platform === "web";
  const REPO = "A7E7/eye-break";
  const RELEASES_PAGE = `https://github.com/${REPO}/releases/latest`;

  let settings = $state<Settings | null>(null);
  let snap = $state<Snapshot | null>(null);
  let savedFlash = $state(false);

  // web-only
  let notifPerm = $state<NotificationPermission | "unsupported">("default");
  let downloads = $state<{ win?: string; macArm?: string; macIntel?: string }>({});

  const os = detectOs();

  onMount(() => {
    if (isWeb && typeof document !== "undefined") {
      document.documentElement.classList.add("web");
    }
    let unsub: (() => void) | undefined;
    (async () => {
      settings = await core.getSettings();
      snap = await core.getState();
      unsub = await core.subscribe((s) => (snap = s));
      if (isWeb) {
        notifPerm =
          typeof Notification === "undefined" ? "unsupported" : Notification.permission;
        loadDownloads();
      }
    })();
    return () => unsub?.();
  });

  async function save() {
    if (!settings) return;
    settings.workMinutes = Math.max(0.05, settings.workMinutes);
    settings.breakSeconds = Math.max(1, Math.round(settings.breakSeconds));
    settings.snoozeMinutes = Math.max(1, Math.round(settings.snoozeMinutes));
    settings.idleThresholdSeconds = Math.max(10, Math.round(settings.idleThresholdSeconds));
    await core.saveSettings(settings);
    savedFlash = true;
    setTimeout(() => {
      savedFlash = false;
      core.dismiss();
    }, 550);
  }

  function lookAway() {
    core.lookAwayNow();
  }

  function confirm() {
    core.confirmLooking();
  }

  function step(key: "workMinutes" | "breakSeconds" | "snoozeMinutes", delta: number, min: number) {
    if (!settings) return;
    const next = Math.round((Number(settings[key]) + delta) * 100) / 100;
    settings[key] = Math.max(min, next);
  }

  async function enableNotifications() {
    if (typeof Notification === "undefined") return;
    notifPerm = await Notification.requestPermission();
  }

  function detectOs(): "windows" | "mac" | "other" {
    if (typeof navigator === "undefined") return "other";
    const ua = navigator.userAgent;
    if (/Win/i.test(ua)) return "windows";
    if (/Mac/i.test(ua)) return "mac";
    return "other";
  }

  async function loadDownloads() {
    try {
      const r = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`);
      if (!r.ok) return;
      const data = await r.json();
      for (const a of data.assets ?? []) {
        const n: string = a.name;
        if (n.endsWith("-setup.exe")) downloads.win = a.browser_download_url;
        else if (n.endsWith("aarch64.dmg")) downloads.macArm = a.browser_download_url;
        else if (n.endsWith("x64.dmg")) downloads.macIntel = a.browser_download_url;
      }
    } catch {
      /* fall back to the releases page links */
    }
  }

  function fmt(secs: number): string {
    const s = Math.max(0, Math.floor(secs));
    return `${Math.floor(s / 60)}:${(s % 60).toString().padStart(2, "0")}`;
  }

  const status = $derived.by(() => {
    if (!snap) return { label: "Loading…", detail: "" };
    switch (snap.phase) {
      case "working":
        return snap.idlePaused
          ? { label: "Paused", detail: "you're away from the keyboard" }
          : { label: fmt(snap.remaining), detail: "until your next break" };
      case "awaitingConfirm":
        return { label: "Look away 👀", detail: "click “I'm looking” when you're ready" };
      case "break":
        return { label: `${Math.max(0, snap.remaining)}s`, detail: "keep looking into the distance" };
      case "paused":
        return { label: "Paused", detail: "reminders are off" };
    }
  });

  const awaiting = $derived(snap?.phase === "awaitingConfirm");
</script>

<div class="titlebar" data-tauri-drag-region>
  <span class="dot" data-tauri-drag-region></span>
  <span class="brand" data-tauri-drag-region>20·20·20</span>
  {#if !isWeb}
    <button class="close" onclick={() => core.dismiss()} aria-label="Hide">×</button>
  {/if}
</div>

<main class="card">
  <section class="status" class:alert={awaiting}>
    <div class="big">{status.label}</div>
    <div class="detail">{status.detail}</div>
    {#if awaiting}
      <button class="confirm" onclick={confirm}>I'm looking</button>
    {:else if snap?.phase !== "break"}
      <button class="lookaway" onclick={lookAway}>Look away now</button>
    {/if}
  </section>

  {#if isWeb && notifPerm !== "granted" && notifPerm !== "unsupported"}
    <button class="notif" onclick={enableNotifications}>
      🔔 Enable notifications
    </button>
  {/if}
  {#if isWeb && notifPerm === "unsupported"}
    <p class="hint">Your browser doesn't support notifications.</p>
  {/if}

  {#if settings}
    <section class="settings">
      <div class="field">
        <label for="work">Work interval</label>
        <div class="stepper">
          <button class="step" onclick={() => step("workMinutes", -1, 1)} aria-label="decrease">−</button>
          <input id="work" type="number" min="0.05" step="1" bind:value={settings.workMinutes} />
          <span class="unit">min</span>
          <button class="step" onclick={() => step("workMinutes", 1, 1)} aria-label="increase">+</button>
        </div>
      </div>

      <div class="field">
        <label for="brk">Break length</label>
        <div class="stepper">
          <button class="step" onclick={() => step("breakSeconds", -5, 5)} aria-label="decrease">−</button>
          <input id="brk" type="number" min="1" step="5" bind:value={settings.breakSeconds} />
          <span class="unit">sec</span>
          <button class="step" onclick={() => step("breakSeconds", 5, 5)} aria-label="increase">+</button>
        </div>
      </div>

      {#if !isWeb}
        <div class="divider"></div>

        <label class="toggle">
          <input type="checkbox" bind:checked={settings.startOnLogin} />
          <span class="track"><span class="knob"></span></span>
          <span class="toggle-label">Start on login</span>
        </label>

        <label class="toggle">
          <input type="checkbox" bind:checked={settings.skipOnIdle} />
          <span class="track"><span class="knob"></span></span>
          <span class="toggle-label">Pause when I'm idle</span>
        </label>

        <label class="toggle">
          <input type="checkbox" bind:checked={settings.soundOnReminder} />
          <span class="track"><span class="knob"></span></span>
          <span class="toggle-label">Play a sound</span>
        </label>

        <div class="field small">
          <label for="snooze">Snooze adds</label>
          <div class="stepper">
            <button class="step" onclick={() => step("snoozeMinutes", -1, 1)} aria-label="decrease">−</button>
            <input id="snooze" type="number" min="1" step="1" bind:value={settings.snoozeMinutes} />
            <span class="unit">min</span>
            <button class="step" onclick={() => step("snoozeMinutes", 1, 1)} aria-label="increase">+</button>
          </div>
        </div>
      {/if}

      <button class="save" class:saved={savedFlash} onclick={save}>
        {savedFlash ? "Saved ✓" : "Save"}
      </button>
    </section>

    {#if isWeb}
      <section class="download">
        <div class="dl-title">Want it always on, even when this tab is closed?</div>
        <div class="dl-sub">Get the lightweight desktop app — it lives in your system tray.</div>
        <div class="dl-buttons">
          <a
            class="dl"
            class:primary={os === "windows"}
            href={downloads.win ?? RELEASES_PAGE}
          >
            <span class="dl-os">Windows</span>
            <span class="dl-meta">.exe installer</span>
          </a>
          <a
            class="dl"
            class:primary={os === "mac"}
            href={downloads.macArm ?? RELEASES_PAGE}
          >
            <span class="dl-os">macOS (Apple Silicon)</span>
            <span class="dl-meta">.dmg</span>
          </a>
          <a class="dl" href={downloads.macIntel ?? RELEASES_PAGE}>
            <span class="dl-os">macOS (Intel)</span>
            <span class="dl-meta">.dmg</span>
          </a>
        </div>
      </section>
    {/if}
  {/if}
</main>

<style>
  :global(html, body) {
    margin: 0;
    background: transparent;
    overflow: hidden;
    user-select: none;
    font-family: "Inter", system-ui, -apple-system, "Segoe UI", sans-serif;
  }

  :global(*) {
    box-sizing: border-box;
  }

  /* Web: scrollable, centered card on a filled background (the desktop build
     is a fixed frameless window, so these only apply in the browser). */
  :global(html.web),
  :global(html.web body) {
    overflow: auto;
    min-height: 100%;
  }
  :global(html.web body) {
    background: linear-gradient(160deg, #11111b, #0b1120);
    padding-top: 18px;
  }
  :global(html.web) .titlebar,
  :global(html.web) .card {
    max-width: 400px;
    margin-left: auto;
    margin-right: auto;
  }

  .titlebar {
    height: 34px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    color: #cdd6f4;
  }
  .titlebar .dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: radial-gradient(circle at 30% 30%, #7ee8c8, #2dd4bf);
    box-shadow: 0 0 8px #2dd4bf99;
  }
  .titlebar .brand {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    opacity: 0.85;
  }
  .titlebar .close {
    margin-left: auto;
    width: 22px;
    height: 22px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: #cdd6f4;
    font-size: 16px;
    line-height: 1;
    cursor: pointer;
    opacity: 0.6;
    transition: all 0.15s;
  }
  .titlebar .close:hover {
    background: #f3868622;
    color: #f38686;
    opacity: 1;
  }

  .card {
    margin: 0 12px 14px;
    padding: 18px;
    border-radius: 18px;
    background: linear-gradient(160deg, #1e1e2eF2, #181825F2);
    border: 1px solid #ffffff14;
    box-shadow: 0 18px 50px #000000a0, inset 0 1px 0 #ffffff10;
    color: #cdd6f4;
    animation: rise 0.55s cubic-bezier(0.2, 0.8, 0.2, 1) both;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(14px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .status {
    text-align: center;
    padding: 14px 10px 18px;
    border-radius: 14px;
    background: radial-gradient(120% 100% at 50% 0%, #2dd4bf1f, transparent 70%);
    transition: background 0.4s;
  }
  .status.alert {
    background: radial-gradient(120% 100% at 50% 0%, #f9c74f2e, transparent 70%);
  }
  .status .big {
    font-size: 38px;
    font-weight: 700;
    letter-spacing: -0.01em;
    line-height: 1.1;
    animation: pop 0.4s ease both;
  }
  .status .detail {
    margin-top: 4px;
    font-size: 12.5px;
    opacity: 0.62;
  }
  @keyframes pop {
    from { transform: scale(0.92); opacity: 0.4; }
    to { transform: scale(1); opacity: 1; }
  }

  .confirm {
    margin-top: 14px;
    padding: 9px 22px;
    border: none;
    border-radius: 10px;
    font-size: 14px;
    font-weight: 600;
    color: #1e1e2e;
    background: linear-gradient(180deg, #fbe08a, #f9c74f);
    cursor: pointer;
    box-shadow: 0 6px 18px #f9c74f55;
    transition: transform 0.12s, box-shadow 0.12s;
  }
  .confirm:hover { transform: translateY(-1px); box-shadow: 0 10px 22px #f9c74f66; }
  .confirm:active { transform: translateY(0); }

  .lookaway {
    margin-top: 14px;
    padding: 9px 22px;
    border: 1px solid #2dd4bf66;
    border-radius: 10px;
    font-size: 13.5px;
    font-weight: 600;
    color: #7ee8c8;
    background: #2dd4bf18;
    cursor: pointer;
    transition: background 0.12s, transform 0.12s;
  }
  .lookaway:hover { background: #2dd4bf2e; transform: translateY(-1px); }
  .lookaway:active { transform: translateY(0); }

  .notif {
    width: 100%;
    margin-top: 12px;
    padding: 9px;
    border: 1px solid #2dd4bf44;
    border-radius: 10px;
    background: #2dd4bf12;
    color: #7ee8c8;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.12s;
  }
  .notif:hover { background: #2dd4bf24; }
  .hint {
    margin: 12px 0 0;
    font-size: 12px;
    opacity: 0.5;
    text-align: center;
  }

  .settings {
    margin-top: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .field label {
    font-size: 13.5px;
    opacity: 0.85;
  }
  .field.small label { font-size: 12.5px; opacity: 0.7; }

  :global(input[type="number"]::-webkit-outer-spin-button),
  :global(input[type="number"]::-webkit-inner-spin-button) {
    -webkit-appearance: none;
    margin: 0;
  }
  :global(input[type="number"]) {
    -moz-appearance: textfield;
    appearance: textfield;
  }

  .stepper {
    display: flex;
    align-items: center;
    border: 1px solid #ffffff1a;
    border-radius: 10px;
    background: #11111b;
    overflow: hidden;
    transition: border-color 0.15s;
  }
  .stepper:focus-within {
    border-color: #2dd4bf;
  }
  .stepper input {
    width: 46px;
    padding: 8px 0;
    text-align: center;
    border: none;
    background: transparent;
    color: #cdd6f4;
    font-size: 14px;
    font-variant-numeric: tabular-nums;
    font-family: inherit;
    outline: none;
  }
  .stepper .unit {
    font-size: 11px;
    opacity: 0.45;
    padding-right: 10px;
  }
  .step {
    width: 30px;
    height: 34px;
    border: none;
    background: transparent;
    color: #cdd6f4;
    font-size: 17px;
    line-height: 1;
    cursor: pointer;
    opacity: 0.65;
    transition: background 0.12s, color 0.12s, opacity 0.12s;
  }
  .step:hover {
    background: #ffffff10;
    color: #2dd4bf;
    opacity: 1;
  }
  .step:active {
    background: #2dd4bf22;
  }

  .divider {
    height: 1px;
    background: #ffffff12;
    margin: 2px 0;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
  }
  .toggle input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }
  .toggle .track {
    position: relative;
    width: 38px;
    height: 22px;
    border-radius: 22px;
    background: #313146;
    transition: background 0.2s;
    flex: none;
  }
  .toggle .knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #cdd6f4;
    transition: transform 0.2s;
  }
  .toggle input:checked + .track {
    background: linear-gradient(180deg, #2dd4bf, #14b8a6);
  }
  .toggle input:checked + .track .knob {
    transform: translateX(16px);
  }
  .toggle-label {
    font-size: 13.5px;
    opacity: 0.9;
  }

  .save {
    margin-top: 6px;
    padding: 11px;
    border: none;
    border-radius: 11px;
    font-size: 14px;
    font-weight: 600;
    color: #1e1e2e;
    background: linear-gradient(180deg, #7ee8c8, #2dd4bf);
    cursor: pointer;
    transition: transform 0.12s, filter 0.2s;
  }
  .save:hover { filter: brightness(1.05); }
  .save:active { transform: scale(0.98); }
  .save.saved {
    background: linear-gradient(180deg, #a6e3a1, #94d88c);
  }

  .download {
    margin: 16px 12px 14px;
    padding: 16px;
    border-radius: 14px;
    background: #11111b88;
    border: 1px solid #ffffff10;
    color: #cdd6f4;
  }
  .dl-title {
    font-size: 14px;
    font-weight: 600;
  }
  .dl-sub {
    margin-top: 3px;
    font-size: 12px;
    opacity: 0.6;
  }
  .dl-buttons {
    margin-top: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .dl {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid #ffffff1a;
    background: #1e1e2e;
    color: #cdd6f4;
    text-decoration: none;
    transition: border-color 0.12s, transform 0.12s;
  }
  .dl:hover { border-color: #2dd4bf66; transform: translateY(-1px); }
  .dl.primary {
    border-color: #2dd4bf;
    background: #2dd4bf16;
  }
  .dl-os { font-size: 13.5px; font-weight: 600; }
  .dl-meta { font-size: 11px; opacity: 0.5; }
</style>
