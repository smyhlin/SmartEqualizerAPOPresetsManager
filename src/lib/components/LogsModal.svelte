<script lang="ts">
  import { FileText, FolderOpen, RefreshCw, X } from '@lucide/svelte';

  import Button from '$lib/components/ui/button.svelte';

  let {
    open = false,
    loading = false,
    logPath = '',
    content = '',
    exists = false,
    onClose,
    onRefresh,
    onOpenLocation,
  } = $props<{
    open?: boolean;
    loading?: boolean;
    logPath?: string;
    content?: string;
    exists?: boolean;
    onClose?: () => void;
    onRefresh?: () => Promise<unknown> | unknown;
    onOpenLocation?: () => Promise<unknown> | unknown;
  }>();

  let localValue = $state('');
  let textAreaElement = $state<HTMLTextAreaElement | null>(null);
  let activeKey = '';

  $effect(() => {
    const nextKey = open ? `${logPath}::${content}` : '';

    if (!open) {
      activeKey = '';
      return;
    }

    if (nextKey !== activeKey) {
      localValue = content;
      queueMicrotask(() => {
        textAreaElement?.focus();
      });
      activeKey = nextKey;
      return;
    }

    if (localValue !== content) {
      localValue = content;
    }
  });

  function close() {
    onClose?.();
  }

  function refresh() {
    void onRefresh?.();
  }

  function openLocation() {
    void onOpenLocation?.();
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
      aria-label="Close logs viewer"
      onclick={close}
    ></button>

    <div class="shell-surface relative z-10 flex h-[min(92vh,980px)] w-full max-w-[1080px] flex-col overflow-hidden rounded-[18px] shadow-[0_28px_80px_rgba(0,0,0,0.55)]">
      <div class="border-b border-border px-4 py-3">
        <div class="flex items-start justify-between gap-4">
          <div class="flex min-w-0 items-center gap-3">
            <div class="flex size-10 shrink-0 items-center justify-center rounded-[12px] border border-accent/30 bg-accent-soft text-accent">
              <FileText size={18} />
            </div>
            <div class="min-w-0">
              <div class="text-sm font-semibold text-foreground">Application Logs</div>
              <div class="mt-0.5 text-xs text-muted">
                Recent activity, install output, and backend errors
              </div>
            </div>
          </div>

          <Button variant="ghost" size="icon" onclick={close} ariaLabel="Close logs viewer">
            <X size={16} />
          </Button>
        </div>
      </div>

      <div class="border-b border-border px-4 py-3">
        <div class="flex flex-wrap items-center gap-2 text-xs text-muted">
          <span class="rounded-full bg-surface-3 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.12em] text-foreground/70">
            {exists ? 'Log file present' : 'Log file not yet created'}
          </span>
          <span class="font-mono text-foreground">{logPath || 'Waiting for a log snapshot...'}</span>
        </div>
      </div>

      <div class="flex min-h-0 flex-1 p-4">
        <div class="flex min-h-0 flex-1 overflow-hidden rounded-[14px] border border-accent/20 bg-[#08131b] shadow-[inset_0_0_0_1px_rgba(132,204,22,0.05)]">
          <textarea
            bind:this={textAreaElement}
            bind:value={localValue}
            readonly
            wrap="off"
            spellcheck="false"
            autocomplete="off"
            autocapitalize="off"
            class="h-full min-h-0 flex-1 resize-none rounded-none border-0 bg-transparent px-4 py-4 font-mono text-[12px] leading-5 text-[#dce6f5] caret-[#84cc16] shadow-none outline-none placeholder:text-[#6f8094] focus:outline-none"
            placeholder={loading ? 'Loading logs...' : 'No log output available yet.'}
          ></textarea>
        </div>
      </div>

      <div class="border-t border-border px-4 py-3">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex flex-wrap items-center gap-2">
            <Button variant="secondary" onclick={refresh} disabled={loading}>
              <RefreshCw size={14} class={loading ? 'animate-spin' : ''} />
              {#if loading}
                Refreshing...
              {:else}
                Refresh Logs
              {/if}
            </Button>

            <Button variant="secondary" onclick={openLocation} disabled={loading}>
              <FolderOpen size={14} />
              Open Logs Folder
            </Button>
          </div>

          <Button variant="secondary" onclick={close}>Close</Button>
        </div>
      </div>
    </div>
  </div>
{/if}
