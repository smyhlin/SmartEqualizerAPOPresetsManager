<script lang="ts">
  import { AlertTriangle, Download, HardDriveDownload, Pencil, Save, WandSparkles } from '@lucide/svelte';

  import Button from '$lib/components/ui/button.svelte';

  let {
    groupName = null,
    presetName = null,
    presetFilePath = null,
    draft = '',
    dirty = false,
    configPath = '',
    defaultConfigPath = '',
    needsConfigMigration = false,
    onSave,
    onApply,
    onExport,
    onEditConfig,
    onSwitchConfigPath
  } = $props<{
    groupName?: string | null;
    presetName?: string | null;
    presetFilePath?: string | null;
    draft?: string;
    dirty?: boolean;
    configPath?: string;
    defaultConfigPath?: string;
    needsConfigMigration?: boolean;
    onSave?: () => void;
    onApply?: () => void;
    onExport?: () => void;
    onEditConfig?: () => void;
    onSwitchConfigPath?: (value: { path: string }) => void;
  }>();
</script>

<section class="flex h-full min-h-0 flex-col overflow-hidden bg-surface">
  <div class="border-b border-border px-4 py-4">
    <div class="flex flex-col gap-4">
      <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div class="min-w-0 flex-1 space-y-3">
          <div class="flex flex-wrap items-center gap-2">
            <span class="inline-flex items-center rounded-full border border-accent/20 bg-accent-soft px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.16em] text-accent">
              Preset text editor
            </span>

            {#if groupName && presetName}
              <span class="inline-flex items-center rounded-full border border-border bg-surface-2 px-2.5 py-1 text-xs font-medium text-foreground">
                {groupName}
              </span>
              <span class="inline-flex items-center rounded-full border border-border bg-surface-2 px-2.5 py-1 text-xs font-medium text-foreground">
                {presetName}
              </span>
            {:else}
              <span class="inline-flex items-center rounded-full border border-border bg-surface-2 px-2.5 py-1 text-xs text-muted">
                Select a preset
              </span>
            {/if}

            <span
              class={`inline-flex items-center rounded-full px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.14em] ${
                dirty ? 'bg-warning-soft text-warning' : 'bg-success-soft text-success'
              }`}
            >
              {dirty ? 'Unsaved' : 'Synced'}
            </span>
          </div>

          <div class="max-w-2xl space-y-2 text-sm leading-6 text-muted">
            <p>Read-only preview for the current preset contents.</p>
            <p>
              Use <span class="font-medium text-foreground">Edit Config</span> to open the popup editor,
              then save or apply from the action bar.
            </p>
          </div>

          {#if presetFilePath}
            <div class="shell-surface-2 flex min-w-0 items-center gap-3 rounded-[12px] px-3 py-2 text-[11px] text-muted">
              <span class="shrink-0 uppercase tracking-[0.16em]">Preset file</span>
              <span class="min-w-0 truncate font-mono text-[12px] text-foreground">{presetFilePath}</span>
            </div>
          {/if}
        </div>

        {#if groupName && presetName}
          <div class="w-full shrink-0 max-w-[280px]">
            <div class="grid grid-cols-2 gap-2">
              <Button variant="outline" size="sm" class="w-full" onclick={() => onEditConfig?.()}>
                <Pencil size={14} />
                Edit Config
              </Button>
              <Button variant="outline" size="sm" class="w-full" onclick={() => onExport?.()}>
                <Download size={14} />
                Export
              </Button>
              <Button variant="secondary" size="sm" class="w-full" onclick={() => onSave?.()}>
                <Save size={14} />
                Save File
              </Button>
              <Button size="sm" class="w-full" onclick={() => onApply?.()}>
                <WandSparkles size={14} />
                Apply
              </Button>
            </div>

            <div class="mt-2 rounded-[12px] border border-border bg-surface-2 px-3 py-2 text-[11px] leading-5 text-muted">
              Save updates the preset file. Apply regenerates Equalizer APO&apos;s live config from the active preset.
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>

  {#if needsConfigMigration}
    <div class="border-b border-border bg-surface-2 px-3 py-3 text-sm">
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

  <div class="border-b border-border px-4 py-3">
    <div class="flex items-center gap-3 text-xs text-muted">
      <span class="shrink-0 uppercase tracking-[0.16em]">Live output</span>
      <div class="min-w-0 flex-1 truncate rounded-[10px] border border-border bg-surface-2 px-3 py-2 font-mono text-[12px] text-foreground">
        {configPath}
      </div>
    </div>
  </div>

  <div class="flex min-h-0 flex-1 p-4">
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[14px] border border-accent/20 bg-[#08131b] shadow-[inset_0_0_0_1px_rgba(132,204,22,0.05)]">
      <div class="flex items-center justify-between gap-3 border-b border-accent/10 px-4 py-2 text-[11px] uppercase tracking-[0.16em] text-[#6f8094]">
        <span>Preview</span>
        {#if groupName && presetName}
          <span class="rounded-full border border-border bg-surface/80 px-2 py-1 text-[10px] font-semibold text-foreground">
            {dirty ? 'Draft' : 'Saved'}
          </span>
        {/if}
      </div>

      {#if groupName && presetName}
        <pre class="min-h-0 flex-1 overflow-auto px-4 py-4 font-mono text-[13px] leading-6 text-[#dce6f5] whitespace-pre-wrap break-words">{draft || '// Empty preset file'}</pre>
      {:else}
        <div class="flex flex-1 items-center justify-center px-4 text-sm text-[#6f8094]">
          Select a preset to preview its config.
        </div>
      {/if}
    </div>
  </div>

  <div class="border-t border-border px-4 py-3 text-xs text-muted">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <span>{dirty ? 'Unsaved changes' : 'Saved to preset storage'}</span>
      <span>APO auto-reloads when `config.txt` changes</span>
    </div>
  </div>
</section>
