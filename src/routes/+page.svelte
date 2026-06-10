<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  type Settings = {
    workMinutes: number;
    breakSeconds: number;
    startOnLogin: boolean;
    skipOnIdle: boolean;
    idleThresholdSeconds: number;
    soundOnReminder: boolean;
    snoozeMinutes: number;
  };

  type Snapshot = {
    phase: "working" | "awaitingConfirm" | "break" | "paused";
    remaining: number;
    idlePaused: boolean;
    settings: Settings;
  };

  let settings = $state<Settings | null>(null);
  let snap = $state<Snapshot | null>(null);
  let savedFlash = $state(false);

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      settings = await invoke<Settings>("get_settings");
      snap = await invoke<Snapshot>("get_state");
      unlisten = await listen<Snapshot>("state-changed", (e) => {
        snap = e.payload;
      });
    })();
    return () => unlisten?.();
  });

  async function save() {
    if (!settings) return;
    // Clamp to sane minimums before persisting.
    settings.workMinutes = Math.max(0.05, settings.workMinutes);
    settings.breakSeconds = Math.max(1, Math.round(settings.breakSeconds));
    settings.snoozeMinutes = Math.max(1, Math.round(settings.snoozeMinutes));
    settings.idleThresholdSeconds = Math.max(10, Math.round(settings.idleThresholdSeconds));
    await invoke("save_settings", { settings });
    savedFlash = true;
    // Brief confirmation, then tuck the window back to the tray.
    setTimeout(() => {
      savedFlash = false;
      getCurrentWindow().hide();
    }, 550);
  }

  function lookAway() {
    invoke("look_away_now");
  }

  function step(key: "workMinutes" | "breakSeconds" | "snoozeMinutes", delta: number, min: number) {
    if (!settings) return;
    const next = Math.round((Number(settings[key]) + delta) * 100) / 100;
    settings[key] = Math.max(min, next);
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

  function closeWindow() {
    getCurrentWindow().hide();
  }
</script>

<div class="titlebar" data-tauri-drag-region>
  <span class="dot" data-tauri-drag-region></span>
  <span class="brand" data-tauri-drag-region>20·20·20</span>
  <button class="close" onclick={closeWindow} aria-label="Hide">×</button>
</div>

<main class="card">
  <section class="status" class:alert={awaiting}>
    <div class="big">{status.label}</div>
    <div class="detail">{status.detail}</div>
    {#if awaiting}
      <button class="confirm" onclick={() => invoke("confirm_looking")}>I'm looking</button>
    {:else if snap?.phase !== "break"}
      <button class="lookaway" onclick={lookAway}>Look away now</button>
    {/if}
  </section>

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

      <button class="save" class:saved={savedFlash} onclick={save}>
        {savedFlash ? "Saved ✓" : "Save"}
      </button>
    </section>
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

  /* Hide the native number spinners everywhere. */
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
</style>
