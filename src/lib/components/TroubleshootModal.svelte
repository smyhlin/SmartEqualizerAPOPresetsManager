<script lang="ts">
  import { Wrench, X } from '@lucide/svelte';

  import Button from '$lib/components/ui/button.svelte';
  import type { PresetLibrary } from '$lib/types';

  let {
    open = false,
    library = null,
    onClose,
    onInstall,
    onOpenSelector
  } = $props<{
    open?: boolean;
    library?: PresetLibrary | null;
    onClose?: () => void;
    onInstall?: () => Promise<unknown> | unknown;
    onOpenSelector?: () => Promise<unknown> | unknown;
  }>();

  const initialStatus =
    'Use the install action to redownload the official Equalizer APO installer and reopen Device Selector for playback and capture device selection.';

  let workingAction = $state<null | 'install' | 'selector'>(null);
  let statusTone = $state<'info' | 'success' | 'error'>('info');
  let statusMessage = $state(initialStatus);
  let previousOpen = false;

  $effect(() => {
    if (open && !previousOpen) {
      workingAction = null;
      statusTone = 'info';
      statusMessage = initialStatus;
    }

    previousOpen = open;
  });

  function close() {
    if (workingAction) {
      return;
    }

    onClose?.();
  }

  function getErrorMessage(error: unknown) {
    if (typeof error === 'string') {
      return error;
    }

    if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') {
      return error.message;
    }

    return 'An unexpected error occurred.';
  }

  async function handleInstall() {
    if (workingAction) {
      return;
    }

    workingAction = 'install';
    statusTone = 'info';
    statusMessage = library?.installedConfigPath
      ? 'Reinstalling Equalizer APO...'
      : 'Installing Equalizer APO...';

    try {
      await onInstall?.();
      statusTone = 'success';
      statusMessage = 'Equalizer APO finished installing and Device Selector opened.';
    } catch (error) {
      statusTone = 'error';
      statusMessage = getErrorMessage(error);
    } finally {
      workingAction = null;
    }
  }

  async function handleOpenSelector() {
    if (workingAction) {
      return;
    }

    workingAction = 'selector';
    statusTone = 'info';
    statusMessage = 'Opening Device Selector...';

    try {
      await onOpenSelector?.();
      statusTone = 'success';
      statusMessage = 'Device Selector opened.';
    } catch (error) {
      statusTone = 'error';
      statusMessage = getErrorMessage(error);
    } finally {
      workingAction = null;
    }
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (open && event.key === 'Escape') {
      close();
    }
  }}
/>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-4">
    <button
      type="button"
      class="absolute inset-0 z-0 bg-black/65 backdrop-blur-[2px]"
      aria-label="Close troubleshoot dialog"
      onclick={close}
    ></button>

    <div class="shell-surface relative z-10 flex w-full max-w-[860px] flex-col overflow-hidden rounded-[18px] shadow-[0_28px_80px_rgba(0,0,0,0.55)]">
      <div class="border-b border-border px-4 py-3">
        <div class="flex items-start justify-between gap-4">
          <div class="flex min-w-0 items-center gap-3">
            <div class="flex size-10 shrink-0 items-center justify-center rounded-[12px] border border-accent/30 bg-accent-soft text-accent">
              <Wrench size={18} />
            </div>
            <div class="min-w-0">
              <div class="text-sm font-semibold text-foreground">Troubleshoot Equalizer APO</div>
              <div class="mt-0.5 text-xs text-muted">
                Repair the install or reopen the official Device Selector
              </div>
            </div>
          </div>

          <Button variant="ghost" size="icon" onclick={close} ariaLabel="Close troubleshoot dialog" disabled={Boolean(workingAction)}>
            <X size={16} />
          </Button>
        </div>
      </div>

      <div class="space-y-4 px-4 py-4">
        <div class="shell-surface-2 rounded-[14px] border border-border px-4 py-4">
          <p class="text-sm leading-6 text-foreground/90">
            Use this panel when Equalizer APO is missing, needs a repair, or you want to reopen the
            official Device Selector for playback and capture device selection.
          </p>
        </div>

        <div class="grid gap-3 md:grid-cols-2">
          <div class="shell-surface-2 rounded-[14px] border border-border px-4 py-3">
            <div class="text-[11px] font-semibold uppercase tracking-[0.16em] text-muted">APO detection</div>
            <div class="mt-1 text-sm font-medium text-foreground">
              {library?.installedConfigPath ? 'Detected' : 'Not detected'}
            </div>
            <div class="mt-1 text-xs leading-5 text-muted">
              {library?.installedConfigPath
                ? 'Equalizer APO is installed and its registry path was found.'
                : 'No Equalizer APO install path is currently visible in the registry.'}
            </div>
          </div>

          <div class="shell-surface-2 rounded-[14px] border border-border px-4 py-3">
            <div class="text-[11px] font-semibold uppercase tracking-[0.16em] text-muted">Active config path</div>
            <div class="mt-1 break-all font-mono text-[12px] leading-5 text-foreground">
              {library?.configPath ?? 'Waiting for the current app snapshot...'}
            </div>
          </div>

          <div class="shell-surface-2 rounded-[14px] border border-border px-4 py-3">
            <div class="text-[11px] font-semibold uppercase tracking-[0.16em] text-muted">Detected install path</div>
            <div class="mt-1 break-all font-mono text-[12px] leading-5 text-foreground">
              {library?.installedConfigPath ?? 'Not detected'}
            </div>
          </div>

          <div class="shell-surface-2 rounded-[14px] border border-border px-4 py-3">
            <div class="text-[11px] font-semibold uppercase tracking-[0.16em] text-muted">Config migration</div>
            <div class="mt-1 text-sm font-medium text-foreground">
              {library?.needsConfigMigration ? 'Needs attention' : 'Writable / no migration needed'}
            </div>
            <div class="mt-1 text-xs leading-5 text-muted">
              {library?.needsConfigMigration
                ? 'Equalizer APO is pointing at a protected config location.'
                : 'The current config path should be writable by the app.'}
            </div>
          </div>
        </div>

        <div
          class={`rounded-[14px] border px-4 py-3 text-sm leading-6 ${
            statusTone === 'success'
              ? 'border-success/25 bg-success-soft text-success'
              : statusTone === 'error'
                ? 'border-danger/25 bg-danger-soft text-danger'
                : 'border-accent/20 bg-accent-soft text-foreground'
          }`}
        >
          {statusMessage}
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <Button
            variant="default"
            onclick={handleInstall}
            disabled={Boolean(workingAction)}
          >
            {#if workingAction === 'install'}
              Installing...
            {:else}
              {library?.installedConfigPath ? 'Reinstall APO' : 'Install APO'}
            {/if}
          </Button>

          <Button
            variant="secondary"
            onclick={handleOpenSelector}
            disabled={Boolean(workingAction)}
          >
            {#if workingAction === 'selector'}
              Opening...
            {:else}
              Open Device Selector
            {/if}
          </Button>
        </div>
      </div>

      <div class="border-t border-border px-4 py-3">
        <div class="flex items-center justify-between gap-3">
          <p class="text-xs leading-5 text-muted">
            The installer runs silently and then opens the official APO Device Selector.
          </p>
          <Button variant="secondary" onclick={close} disabled={Boolean(workingAction)}>
            Close
          </Button>
        </div>
      </div>
    </div>
  </div>
{/if}
