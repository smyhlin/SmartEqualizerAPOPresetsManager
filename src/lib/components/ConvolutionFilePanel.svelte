<script lang="ts">
  import { Check, FolderOpen, RefreshCw, Trash2 } from '@lucide/svelte';
  import { resolve } from '@tauri-apps/api/path';

  import Button from '$lib/components/ui/button.svelte';
  import { extractConvolutionPath } from '$lib/convolution';
  import { revealPathInExplorer } from '$lib/tauri';
  import { cn } from '$lib/utils';

  let {
    draft = '',
    configPath = null,
    panelKey = '',
    presetError = null,
    onToggleConvolution,
  } = $props<{
    draft?: string;
    configPath?: string | null;
    panelKey?: string;
    presetError?: string | null;
    onToggleConvolution?: (value: { enabled: boolean }) => Promise<boolean> | boolean;
  }>();

  let statusMessage = $state<string | null>(null);
  let statusTone = $state<'info' | 'success' | 'error'>('info');
  let busy = $state(false);
  let activePanelKey = '';

  let convolutionPath = $derived.by(() => extractConvolutionPath(draft));

  $effect(() => {
    const nextKey = panelKey ?? '';
    if (nextKey !== activePanelKey) {
      activePanelKey = nextKey;
      statusMessage = null;
      statusTone = 'info';
      busy = false;
    }
  });

  function setStatus(message: string | null, tone: 'info' | 'success' | 'error' = 'info') {
    statusMessage = message;
    statusTone = tone;
  }

  function getErrorMessage(error: unknown) {
    if (typeof error === 'string') {
      return error;
    }

    if (
      error &&
      typeof error === 'object' &&
      'message' in error &&
      typeof error.message === 'string'
    ) {
      return error.message;
    }

    return 'An unexpected error occurred.';
  }

  function isAbsolutePath(path: string) {
    return /^[a-zA-Z]:[\\/]/.test(path) || path.startsWith('\\\\');
  }

  async function applyToggle(enabled: boolean) {
    if (!onToggleConvolution) {
      return false;
    }

    busy = true;
    try {
      const success = await onToggleConvolution({ enabled });
      if (success) {
        setStatus(
          enabled ? 'Linked convolution WAV.' : 'Removed convolution WAV.',
          'success'
        );
      }
      return success;
    } catch (error) {
      setStatus(getErrorMessage(error), 'error');
      return false;
    } finally {
      busy = false;
    }
  }

  async function handleRevealWav() {
    if (!convolutionPath) {
      return;
    }

    busy = true;
    try {
      const targetPath =
        isAbsolutePath(convolutionPath) || !configPath
          ? convolutionPath
          : await resolve(configPath, convolutionPath);
      await revealPathInExplorer(targetPath);
      setStatus('Opened the convolution file location.', 'success');
    } catch (error) {
      setStatus(getErrorMessage(error), 'error');
    } finally {
      busy = false;
    }
  }

  async function handleToggleFromCheckbox(event: Event) {
    const checked = (event.currentTarget as HTMLInputElement).checked;
    if (checked === Boolean(convolutionPath)) {
      return;
    }

    await applyToggle(checked);
  }

  async function handleAttach() {
    await applyToggle(true);
  }

  async function handleRemove() {
    await applyToggle(false);
  }
</script>

<section class="shell-surface-2 group rounded-[12px] p-3 shadow-sm transition-all focus-within:ring-1 focus-within:ring-border/50 hover:bg-surface-2/40">
  <div class="flex items-center gap-3.5">
    <label
      class={cn(
        'group/cb inline-flex size-9 shrink-0 items-center justify-center rounded-[10px] border transition-all focus-within:ring-2 focus-within:ring-accent/25',
        convolutionPath
          ? 'border-accent/35 bg-accent/10 text-accent ring-1 ring-inset ring-accent/10'
          : 'border-border/60 bg-surface-3 text-muted hover:border-accent/30 hover:bg-accent/5'
      )}
      title="Convolution mode"
    >
      <input
        type="checkbox"
        checked={Boolean(convolutionPath)}
        disabled={busy}
        onchange={handleToggleFromCheckbox}
        aria-label="Convolution mode"
        class="sr-only"
      />
      <span
        aria-hidden="true"
        class={cn(
          'inline-flex size-4 items-center justify-center rounded-[4px] border transition-all',
          convolutionPath
            ? 'border-accent bg-accent text-background shadow-[0_0_0_1px_rgba(163,230,53,0.18)]'
            : 'border-border bg-background/60 text-transparent group-hover/cb:border-accent/40 group-hover/cb:bg-accent/10'
        )}
      >
        <Check size={11} strokeWidth={3.5} class={convolutionPath ? 'scale-100 opacity-100' : 'scale-50 opacity-0 transition-transform'} />
      </span>
    </label>

    <div class="min-w-0 flex-1">
      <p class="mb-0.5 text-[10px] font-bold uppercase tracking-[0.16em] text-muted">
        Convolution WAV
      </p>

      {#if convolutionPath}
        <p class="truncate font-mono text-[11.5px] font-medium text-foreground/95" title={convolutionPath}>
          {convolutionPath}
        </p>
        {#if configPath}
          <p class="mt-0.5 truncate text-[10px] text-muted/60">
            relative to <span class="font-mono text-muted/75">{configPath}</span>
          </p>
        {/if}
      {:else}
        <p class="truncate text-[11px] text-muted/80">
          No impulse response linked.
        </p>
      {/if}

      {#if presetError}
        <p class="mt-1 truncate text-[10.5px] text-danger">
          {presetError}
        </p>
      {/if}
    </div>

    <div class="flex shrink-0 items-center gap-1.5 opacity-90 transition-opacity group-hover:opacity-100">
      {#if convolutionPath}
        <div class="flex items-center gap-0.5 rounded-[9px] bg-surface-3/50 p-0.5 border border-border/30">
          <Button size="icon" variant="ghost" class="size-7 rounded-[6px] text-muted hover:bg-surface-2 hover:text-foreground" onclick={handleRevealWav} title="Open in Explorer" disabled={busy}>
            <FolderOpen size={13} strokeWidth={2.5} />
          </Button>
          <Button size="icon" variant="ghost" class="size-7 rounded-[6px] text-muted hover:bg-surface-2 hover:text-foreground" onclick={handleAttach} title="Change WAV" disabled={busy}>
            <RefreshCw size={13} strokeWidth={2.5} />
          </Button>
          <div class="mx-0.5 h-3.5 w-px bg-border/60"></div>
          <Button size="icon" variant="ghost" class="size-7 rounded-[6px] text-danger/80 hover:bg-danger-soft hover:text-danger" onclick={handleRemove} title="Remove WAV" disabled={busy}>
            <Trash2 size={13} strokeWidth={2.5} />
          </Button>
        </div>
      {:else}
        <Button size="sm" variant="secondary" class="h-7 rounded-[8px] px-3.5 text-[11px] font-medium shadow-sm border border-border/40" disabled={busy} onclick={handleAttach}>
          Attach WAV
        </Button>
      {/if}
    </div>
  </div>

  {#if statusMessage}
    <p
      class={cn(
        'mt-2.5 text-[10.5px] leading-snug px-1',
        statusTone === 'error'
          ? 'text-danger'
          : statusTone === 'success'
            ? 'text-success'
            : 'text-muted'
      )}
    >
      {statusMessage}
    </p>
  {/if}
</section>
