<script lang="ts">
  import { AlertTriangle, HardDriveDownload, Save, WandSparkles, X } from '@lucide/svelte';

  import Button from '$lib/components/ui/button.svelte';

  let {
    open = false,
    groupName = null,
    presetName = null,
    presetFilePath = null,
    draft = '',
    dirty = false,
    configPath = '',
    defaultConfigPath = '',
    needsConfigMigration = false,
    onDraftChange,
    onSave,
    onApply,
    onClose,
    onSwitchConfigPath
  } = $props<{
    open?: boolean;
    groupName?: string | null;
    presetName?: string | null;
    presetFilePath?: string | null;
    draft?: string;
    dirty?: boolean;
    configPath?: string;
    defaultConfigPath?: string;
    needsConfigMigration?: boolean;
    onDraftChange?: (value: string) => void;
    onSave?: () => void;
    onApply?: () => void;
    onClose?: () => void;
    onSwitchConfigPath?: (value: { path: string }) => void;
  }>();

  let localValue = $state('');
  let editorElement = $state<HTMLTextAreaElement | null>(null);
  let activeKey = '';

  $effect(() => {
    const nextKey = open ? `${groupName ?? ''}::${presetName ?? ''}::${presetFilePath ?? ''}` : '';

    if (open && nextKey !== activeKey) {
      localValue = draft;
      queueMicrotask(() => {
        editorElement?.focus();
        editorElement?.setSelectionRange(editorElement.value.length, editorElement.value.length);
      });
    }

    if (!open) {
      activeKey = '';
      return;
    }

    activeKey = nextKey;
  });

  function close() {
    onClose?.();
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
      aria-label="Close config editor"
      onclick={close}
    ></button>

    <div class="shell-surface relative z-10 flex h-[min(92vh,980px)] w-full max-w-[1200px] flex-col overflow-hidden rounded-[18px] shadow-[0_28px_80px_rgba(0,0,0,0.55)]">
      <div class="border-b border-border px-4 py-3">
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0">
            <div class="text-sm font-medium text-foreground">Edit config</div>
            <div class="mt-0.5 truncate text-xs text-muted">
              {#if groupName && presetName}
                {groupName} / {presetName}
              {:else}
                Select a preset to edit
              {/if}
            </div>
            {#if presetFilePath}
              <div class="mt-1 truncate text-[11px] text-muted">
                File: <span class="font-mono text-foreground">{presetFilePath}</span>
              </div>
            {/if}
          </div>

          <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => onSave?.()}>
              <Save size={14} />
              Save File
            </Button>
            <Button onclick={() => onApply?.()}>
              <WandSparkles size={14} />
              Apply
            </Button>
            <Button variant="ghost" size="icon" onclick={close} ariaLabel="Close editor">
              <X size={16} />
            </Button>
          </div>
        </div>
      </div>

      {#if needsConfigMigration}
        <div class="border-b border-border bg-surface-2 px-4 py-3 text-sm">
          <div class="flex items-start gap-3">
            <div class="mt-0.5 inline-flex size-8 shrink-0 items-center justify-center rounded-[8px] border border-warning/30 bg-warning-soft text-warning">
              <AlertTriangle size={15} />
            </div>
            <div class="min-w-0 flex-1">
              <div class="font-medium text-foreground">Equalizer APO is using a protected config path.</div>
              <div class="mt-1 text-sm text-muted">
                Switch ConfigPath to the writable app folder before applying presets.
              </div>
              <div class="mt-2 flex items-center gap-2">
                <div class="min-w-0 flex-1 truncate rounded-[8px] border border-border bg-background px-3 py-2 font-mono text-[12px] text-foreground">
                  {configPath}
                </div>
                <Button
                  variant="outline"
                  onclick={() => onSwitchConfigPath?.({ path: defaultConfigPath })}
                >
                  <HardDriveDownload size={14} />
                  Use AppData
                </Button>
              </div>
            </div>
          </div>
        </div>
      {/if}

      <div class="border-b border-border px-4 py-2 text-xs text-muted">
        Writing active output to <span class="font-mono text-foreground">{configPath}</span>
      </div>

      <div class="flex min-h-0 flex-1 p-4">
        <div class="flex min-h-0 flex-1 overflow-hidden rounded-[14px] border border-accent/20 bg-[#08131b] shadow-[inset_0_0_0_1px_rgba(132,204,22,0.05)]">
          <textarea
            bind:this={editorElement}
            bind:value={localValue}
            rows="1"
            placeholder="// Edit the preset text here"
            spellcheck="false"
            autocomplete="off"
            autocapitalize="off"
            wrap="soft"
            class="h-full min-h-0 flex-1 resize-none rounded-none border-0 bg-transparent px-4 py-4 font-mono text-[13px] leading-6 text-[#dce6f5] caret-[#84cc16] shadow-none outline-none placeholder:text-[#6f8094] focus:outline-none"
            oninput={() => onDraftChange?.(localValue)}
          ></textarea>
        </div>
      </div>

      <div class="border-t border-border px-4 py-2 text-xs text-muted">
        {#if dirty}
          Unsaved changes
        {:else}
          Saved to preset storage
        {/if}
        <span class="mx-2 text-border">•</span>
        Close keeps your draft in memory until you save or apply it
      </div>
    </div>
  </div>
{/if}
