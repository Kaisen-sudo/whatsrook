<script lang="ts">
  import { fade, fly, slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { generateSessionToken, type WhatsRookConfig } from '$lib/crypto';

  // Config matching whatsrook CLI
  let config = $state<WhatsRookConfig>({
    phone: '',
    business: false,
    auth: 'qr',
    client: 'default',
    db: '',
    autoupdate: true,
    verbose: false
  });

  let step = $state(1);
  const totalSteps = 5;
  let activeTooltip = $state<string | null>(null);
  let isGenerating = $state(false);
  let generatedToken = $state('');
  let copied = $state(false);
  let errorMsg = $state('');

  // Custom Dropdown State
  let isDropdownOpen = $state(false);
  const clientOptions = [
    { value: 'default', label: 'Default (Web Browser)', desc: 'Standard browser session' },
    { value: 'android', label: 'Android Phone', desc: 'Simulates official Android app' },
    { value: 'ios', label: 'iPhone / iPad', desc: 'Simulates official iOS app' }
  ];

  let selectedClientLabel = $derived(
    clientOptions.find((o) => o.value === config.client)?.label ?? 'Default (Web Browser)'
  );

  function selectClient(val: 'default' | 'android' | 'ios') {
    config.client = val;
    isDropdownOpen = false;
  }

  // Database URL Validator
  function isValidPostgresUrl(url: string): boolean {
    const trimmed = url.trim();
    if (!trimmed) return false;
    const pattern = /^(postgres|postgresql):\/\/(?:([^:]+)(?::([^@]*))?@)?([a-zA-Z0-9.-]+)(?::(\d+))?\/([a-zA-Z0-9_-]+)(?:\?.*)?$/;
    return pattern.test(trimmed);
  }

  function nextFromStep1() {
    isDropdownOpen = false;
    step = 2;
  }

  function nextFromStep2() {
    errorMsg = '';
    const cleanPhone = config.phone.replace(/[\s\-()]/g, '');
    if (config.auth === 'pair') {
      if (!cleanPhone) {
        errorMsg = 'Please enter your phone number with your country code to get a pairing code.';
        return;
      }
      if (!/^\+?\d{8,15}$/.test(cleanPhone)) {
        errorMsg = 'Please enter a valid phone number (e.g., +2348012345678).';
        return;
      }
    }
    step = 3;
  }

  function nextFromStep3(skip = false) {
    errorMsg = '';
    if (!skip) {
      const trimmed = config.db.trim();
      if (!trimmed) {
        errorMsg = 'Please enter a database URL or click "Skip this step".';
        return;
      }
      if (!isValidPostgresUrl(trimmed)) {
        errorMsg = 'Please enter a valid PostgreSQL URL (e.g. postgresql://user:pass@host:5432/dbname).';
        return;
      }
      config.db = trimmed;
    } else {
      config.db = '';
    }
    step = 4;
  }

  function prevStep() {
    errorMsg = '';
    isDropdownOpen = false;
    if (step > 1) step -= 1;
  }

  async function handleFinalize() {
    errorMsg = '';
    isGenerating = true;
    step = 5;

    try {
      const [token] = await Promise.all([
        generateSessionToken(config),
        new Promise((resolve) => setTimeout(resolve, 800)) // Ghost shimmer duration
      ]);
      generatedToken = token;
    } catch {
      errorMsg = 'Failed to generate session ID. Please check your inputs.';
    } finally {
      isGenerating = false;
    }
  }

  function copyToken() {
    if (!generatedToken) return;
    navigator.clipboard.writeText(generatedToken);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function reset() {
    config = {
      phone: '',
      business: false,
      auth: 'qr',
      client: 'default',
      db: '',
      autoupdate: true,
      verbose: false
    };
    step = 1;
    generatedToken = '';
    copied = false;
    errorMsg = '';
    isGenerating = false;
    isDropdownOpen = false;
  }
</script>

<svelte:window onclick={(e) => {
  const target = e.target as HTMLElement;
  if (!target.closest('.custom-dropdown-container')) {
    isDropdownOpen = false;
  }
}} />

<div class="viewport-wrapper">
  <div class="card-frame">
    <!-- Top Progress Bar -->
    <div class="progress-bar-track">
      <div 
        class="progress-bar-fill" 
        style="width: {(step / totalSteps) * 100}%;"
      ></div>
    </div>

    {#if errorMsg}
      <div class="error-banner" transition:slide={{ duration: 180, easing: cubicOut }}>
        {errorMsg}
      </div>
    {/if}

    <div class="step-slider">
      <!-- STEP 1: Account Type & Appearance -->
      {#if step === 1}
        <div 
          class="step-body"
          in:fly={{ x: 16, duration: 240, delay: 100, easing: cubicOut }}
          out:fly={{ x: -16, duration: 140, easing: cubicOut }}
        >
          <div class="header-row">
            <h2 class="display-sm">WhatsApp Account</h2>
            <div 
              class="tooltip-wrapper"
              onmouseenter={() => activeTooltip = 'account'}
              onmouseleave={() => activeTooltip = null}
            >
              <button type="button" class="tooltip-trigger" aria-label="Help">?</button>
              {#if activeTooltip === 'account'}
                <div class="uber-tooltip" transition:fade={{ duration: 120 }}>
                  Choose whether you are connecting a personal WhatsApp number or a business account.
                </div>
              {/if}
            </div>
          </div>
          <p class="body-sm-muted">Choose your WhatsApp type and device profile</p>

          <div class="field-label">Account type</div>
          <div class="pill-toggle-grid">
            <button 
              type="button"
              class="segment-pill" 
              class:selected={!config.business} 
              onclick={() => config.business = false}>
              Regular WhatsApp
            </button>
            <button 
              type="button"
              class="segment-pill" 
              class:selected={config.business} 
              onclick={() => config.business = true}>
              WhatsApp Business
            </button>
          </div>

          <div class="field-label" style="margin-top: 20px;">Device appearance</div>
          <!-- Custom Uber Dropdown -->
          <div class="custom-dropdown-container">
            <button 
              type="button" 
              class="custom-dropdown-btn" 
              class:open={isDropdownOpen}
              onclick={() => isDropdownOpen = !isDropdownOpen}
            >
              <span>{selectedClientLabel}</span>
              <svg class="chevron" class:rotated={isDropdownOpen} viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
                <path d="M3.2 5.5l4.8 4.8 4.8-4.8.7.7-5.5 5.5-5.5-5.5.7-.7z"/>
              </svg>
            </button>

            {#if isDropdownOpen}
              <div class="custom-dropdown-menu" transition:slide={{ duration: 160, easing: cubicOut }}>
                {#each clientOptions as opt}
                  <button 
                    type="button" 
                    class="dropdown-item" 
                    class:active={config.client === opt.value}
                    onclick={() => selectClient(opt.value as 'default' | 'android' | 'ios')}
                  >
                    <div class="dropdown-item-title">{opt.label}</div>
                    <div class="dropdown-item-desc">{opt.desc}</div>
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <button class="btn-primary" onclick={nextFromStep1} style="margin-top: 24px;">Continue</button>
        </div>

      <!-- STEP 2: How to Login (QR vs Pairing Code) -->
      {:else if step === 2}
        <div 
          class="step-body"
          in:fly={{ x: 16, duration: 240, delay: 100, easing: cubicOut }}
          out:fly={{ x: -16, duration: 140, easing: cubicOut }}
        >
          <div class="header-row">
            <h2 class="display-sm">Link Your Account</h2>
            <div 
              class="tooltip-wrapper"
              onmouseenter={() => activeTooltip = 'auth'}
              onmouseleave={() => activeTooltip = null}
            >
              <button type="button" class="tooltip-trigger" aria-label="Help">?</button>
              {#if activeTooltip === 'auth'}
                <div class="uber-tooltip" transition:fade={{ duration: 120 }}>
                  With QR code, your bot shows a code on terminal to scan. With Pairing code, it gives you an 8-digit number to enter directly into WhatsApp.
                </div>
              {/if}
            </div>
          </div>
          <p class="body-sm-muted">Select how you want to link your bot</p>

          <div class="pill-toggle-grid">
            <button 
              type="button"
              class="segment-pill" 
              class:selected={config.auth === 'qr'} 
              onclick={() => config.auth = 'qr'}>
              Scan QR Code
            </button>
            <button 
              type="button"
              class="segment-pill" 
              class:selected={config.auth === 'pair'} 
              onclick={() => config.auth = 'pair'}>
              Use Pairing Code
            </button>
          </div>

          <div class="field-label" style="margin-top: 20px;">
            Phone number {config.auth === 'qr' ? '(optional for labels)' : '(required for pairing)'}
          </div>
          <div class="input-row">
            <input
              type="tel"
              placeholder="+234 801 234 5678"
              bind:value={config.phone}
              class="form-input"
              onkeydown={(e) => e.key === 'Enter' && nextFromStep2()}
            />
          </div>

          <div class="button-row">
            <button class="btn-subtle" onclick={prevStep}>Back</button>
            <button class="btn-primary" onclick={nextFromStep2}>Next</button>
          </div>
        </div>

      <!-- STEP 3: Database Storage (With URL validation & fallback skip) -->
      {:else if step === 3}
        <div 
          class="step-body"
          in:fly={{ x: 16, duration: 240, delay: 100, easing: cubicOut }}
          out:fly={{ x: -16, duration: 140, easing: cubicOut }}
        >
          <div class="header-row">
            <h2 class="display-sm">Database Storage</h2>
            <div 
              class="tooltip-wrapper"
              onmouseenter={() => activeTooltip = 'db'}
              onmouseleave={() => activeTooltip = null}
            >
              <button type="button" class="tooltip-trigger" aria-label="Help">?</button>
              {#if activeTooltip === 'db'}
                <div class="uber-tooltip" transition:fade={{ duration: 120 }}>
                  Enter a PostgreSQL connection string to persist your contacts and messages in the cloud. If you skip, the bot automatically uses local file storage.
                </div>
              {/if}
            </div>
          </div>
          <p class="body-sm-muted">Store your chats and sessions in the cloud</p>

          <div class="field-label">PostgreSQL / Supabase URL</div>
          <div class="input-row">
            <input
              type="text"
              placeholder="postgresql://postgres:pass@db.example.com:5432/postgres"
              bind:value={config.db}
              class="form-input"
              onkeydown={(e) => e.key === 'Enter' && nextFromStep3(false)}
            />
          </div>

          <div class="helper-line">
            <span class="body-sm">Need a free database?</span>
            <a href="https://supabase.com" target="_blank" rel="noreferrer" class="link-blue">
              Get one on Supabase
            </a>
          </div>

          <div class="button-row-stacked">
            <button class="btn-primary" onclick={() => nextFromStep3(false)}>
              Save Database
            </button>
            <button class="btn-subtle" onclick={() => nextFromStep3(true)}>
              Skip this step (Use local storage)
            </button>
          </div>
        </div>

      <!-- STEP 4: Preferences -->
      {:else if step === 4}
        <div 
          class="step-body"
          in:fly={{ x: 16, duration: 240, delay: 100, easing: cubicOut }}
          out:fly={{ x: -16, duration: 140, easing: cubicOut }}
        >
          <div class="header-row">
            <h2 class="display-sm">Preferences</h2>
            <div 
              class="tooltip-wrapper"
              onmouseenter={() => activeTooltip = 'pref'}
              onmouseleave={() => activeTooltip = null}
            >
              <button type="button" class="tooltip-trigger" aria-label="Help">?</button>
              {#if activeTooltip === 'pref'}
                <div class="uber-tooltip" transition:fade={{ duration: 120 }}>
                  Keep automatic updates turned on to ensure security patches and protocol fixes are applied seamlessly.
                </div>
              {/if}
            </div>
          </div>
          <p class="body-sm-muted">Configure basic bot behavior</p>

          <div class="preference-box">
            <label class="switch-row">
              <div>
                <div class="switch-title">Auto-updates</div>
                <div class="switch-desc">Check for bot updates on startup</div>
              </div>
              <input type="checkbox" bind:checked={config.autoupdate} class="native-toggle" />
            </label>

            <label class="switch-row">
              <div>
                <div class="switch-title">Detailed logs</div>
                <div class="switch-desc">Show connection events in terminal</div>
              </div>
              <input type="checkbox" bind:checked={config.verbose} class="native-toggle" />
            </label>
          </div>

          <div class="button-row" style="margin-top: 24px;">
            <button class="btn-subtle" onclick={prevStep}>Back</button>
            <button class="btn-primary" onclick={handleFinalize}>
              Generate ID
            </button>
          </div>
        </div>

      <!-- STEP 5: Final Result with Ghost Shimmer -->
      {:else if step === 5}
        <div 
          class="step-body"
          in:fly={{ x: 16, duration: 240, delay: 100, easing: cubicOut }}
          out:fly={{ x: -16, duration: 140, easing: cubicOut }}
        >
          <h2 class="display-sm">Your Session ID</h2>
          <p class="body-sm-muted">Copy and paste this into your bot configuration</p>

          {#if isGenerating}
            <!-- Shimmer Ghost Loader -->
            <div class="skeleton-container" aria-busy="true">
              <div class="skeleton-line shimmer" style="width: 100%;"></div>
              <div class="skeleton-line shimmer" style="width: 82%;"></div>
              <div class="skeleton-line shimmer" style="width: 50%;"></div>
            </div>
            <div class="button-row" style="margin-top: 20px;">
              <div class="skeleton-btn shimmer"></div>
              <div class="skeleton-btn shimmer"></div>
            </div>
          {:else}
            <div class="token-container" in:fade={{ duration: 180 }}>
              <code>{generatedToken}</code>
            </div>

            <div class="button-row">
              <button class="btn-subtle" onclick={reset}>Create Another</button>
              <button class="btn-primary" onclick={copyToken}>
                {copied ? 'Copied to Clipboard!' : 'Copy Session ID'}
              </button>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .viewport-wrapper {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-canvas);
    padding: 24px;
  }

  .card-frame {
    width: 100%;
    max-width: 440px;
    background-color: var(--color-canvas);
    border-radius: var(--radius-xl);
    padding: 32px;
    box-shadow: var(--shadow-level-2);
    border: 1px solid var(--color-canvas-soft);
    position: relative;
  }

  /* Progress Track */
  .progress-bar-track {
    width: 100%;
    height: 3px;
    background-color: var(--color-canvas-soft);
    border-radius: var(--radius-pill);
    overflow: hidden;
    margin-bottom: 24px;
  }
  .progress-bar-fill {
    height: 100%;
    background-color: var(--color-primary);
    transition: width 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .step-slider {
    position: relative;
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .display-sm {
    font-size: 21px;
    font-weight: 600;
    line-height: 28px;
    letter-spacing: -0.02em;
    color: var(--color-ink);
  }

  .body-sm-muted {
    font-size: 14px;
    color: var(--color-body);
    margin-top: 4px;
    margin-bottom: 20px;
  }

  .field-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-body);
    margin-bottom: 8px;
  }

  /* Segmented Pill Selector */
  .pill-toggle-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    background-color: var(--color-canvas-soft);
    padding: 4px;
    border-radius: var(--radius-pill-tab);
    gap: 4px;
  }
  .segment-pill {
    background: transparent;
    border: none;
    border-radius: var(--radius-pill-tab);
    padding: 10px 0;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-ink);
    cursor: pointer;
    transition: background 0.15s ease, box-shadow 0.15s ease;
  }
  .segment-pill.selected {
    background-color: var(--color-canvas);
    box-shadow: var(--shadow-level-3);
  }

  /* Custom Dropdown Component */
  .custom-dropdown-container {
    position: relative;
    width: 100%;
  }

  .custom-dropdown-btn {
    width: 100%;
    background-color: var(--color-canvas-soft);
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    padding: 14px 16px;
    font-size: 15px;
    font-weight: 500;
    font-family: inherit;
    color: var(--color-ink);
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    transition: background 0.15s ease, border-color 0.15s ease;
  }
  .custom-dropdown-btn:hover {
    background-color: var(--color-surface-pressed);
  }
  .custom-dropdown-btn.open {
    border-color: var(--color-hairline-mid);
    background-color: var(--color-canvas);
    box-shadow: var(--shadow-level-3);
  }

  .chevron {
    transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    color: var(--color-body);
  }
  .chevron.rotated {
    transform: rotate(180deg);
  }

  .custom-dropdown-menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    width: 100%;
    background-color: var(--color-canvas);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-level-2);
    border: 1px solid var(--color-canvas-soft);
    padding: 6px;
    z-index: 100;
    overflow: hidden;
  }

  .dropdown-item {
    width: 100%;
    border: none;
    background: transparent;
    border-radius: var(--radius-md);
    padding: 10px 12px;
    text-align: left;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.12s ease;
  }
  .dropdown-item:hover {
    background-color: var(--color-canvas-soft);
  }
  .dropdown-item.active {
    background-color: var(--color-canvas-soft);
  }
  .dropdown-item-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-ink);
  }
  .dropdown-item-desc {
    font-size: 12px;
    color: var(--color-body);
    margin-top: 2px;
  }

  /* Input Rows */
  .input-row {
    background-color: var(--color-canvas-soft);
    border-radius: var(--radius-md);
    padding: 14px 16px;
    margin-bottom: 16px;
  }
  .form-input {
    width: 100%;
    border: none;
    background: transparent;
    font-size: 15px;
    font-family: inherit;
    color: var(--color-ink);
    outline: none;
  }

  .helper-line {
    margin-top: -6px;
    margin-bottom: 24px;
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 13px;
  }
  .body-sm {
    color: var(--color-body);
  }
  .link-blue {
    color: var(--color-link);
    text-decoration: underline;
  }

  /* Preference Controls */
  .preference-box {
    border-radius: var(--radius-lg);
    background-color: var(--color-canvas-soft);
    padding: 6px 16px;
  }
  .switch-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 0;
    cursor: pointer;
  }
  .switch-row:not(:last-child) {
    border-bottom: 1px solid var(--color-surface-pressed);
  }
  .switch-title {
    font-size: 14px;
    font-weight: 500;
  }
  .switch-desc {
    font-size: 12px;
    color: var(--color-body);
  }
  .native-toggle {
    width: 18px;
    height: 18px;
    accent-color: var(--color-primary);
    cursor: pointer;
  }

  /* Buttons */
  .btn-primary {
    background-color: var(--color-primary);
    color: var(--color-on-primary);
    font-size: 15px;
    font-weight: 500;
    border-radius: var(--radius-pill);
    padding: 12px 20px;
    border: none;
    cursor: pointer;
    width: 100%;
    transition: background-color 0.15s ease;
  }
  .btn-primary:active {
    background-color: var(--color-black-elevated);
  }
  .btn-subtle {
    background-color: var(--color-canvas-soft);
    color: var(--color-ink);
    font-size: 15px;
    font-weight: 500;
    border-radius: var(--radius-pill);
    padding: 12px 20px;
    border: none;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }
  .btn-subtle:active {
    background-color: var(--color-surface-pressed);
  }

  .button-row {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: 12px;
    margin-top: 8px;
  }
  .button-row-stacked {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  /* Tooltip */
  .tooltip-wrapper {
    position: relative;
    display: inline-flex;
  }
  .tooltip-trigger {
    width: 22px;
    height: 22px;
    border-radius: var(--radius-full);
    background-color: var(--color-canvas-soft);
    border: none;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-body);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .uber-tooltip {
    position: absolute;
    bottom: calc(100% + 8px);
    right: 0;
    width: 220px;
    background-color: var(--color-ink);
    color: var(--color-on-dark);
    font-size: 12px;
    line-height: 16px;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-level-3);
    z-index: 120;
    pointer-events: none;
  }

  /* Shimmer Ghost Loaders */
  .skeleton-container {
    background-color: var(--color-canvas-soft);
    border-radius: var(--radius-md);
    padding: 24px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .skeleton-line {
    height: 12px;
    border-radius: 4px;
  }
  .skeleton-btn {
    height: 44px;
    border-radius: var(--radius-pill);
  }

  .shimmer {
    background: linear-gradient(
      90deg,
      #e2e2e2 0%,
      #ffffff 50%,
      #e2e2e2 100%
    );
    background-size: 250% 100%;
    animation: uberShimmer 1.1s infinite linear;
  }

  @keyframes uberShimmer {
    0% { background-position: -250% 0; }
    100% { background-position: 250% 0; }
  }

  /* Result Container */
  .token-container {
    background-color: var(--color-canvas-soft);
    border-radius: var(--radius-md);
    padding: 16px;
    margin-bottom: 20px;
    word-break: break-all;
    max-height: 140px;
    overflow-y: auto;
  }
  .token-container code {
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 18px;
    color: var(--color-ink);
  }

  .error-banner {
    background-color: var(--color-canvas-soft);
    border-left: 3px solid var(--color-primary);
    padding: 10px 14px;
    font-size: 13px;
    margin-bottom: 16px;
  }
</style>