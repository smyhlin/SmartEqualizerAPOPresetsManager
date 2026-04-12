<script lang="ts">
  import { CheckCircle2, FilePlus2, FolderInput, Pencil, Trash2, Zap } from '@lucide/svelte';

  import Button from '$lib/components/ui/button.svelte';
  import Input from '$lib/components/ui/input.svelte';
  import type { PresetGroup, PresetItem } from '$lib/types';
  import { cn } from '$lib/utils';

  type MovePreset = {
    oldGroup: string;
    newGroup: string;
    name: string;
  };

  let {
    group = null,
    selectedPresetName = null,
    search = '',
    presetFilePath = null,
    onSelect,
    onCreate,
    onRename,
    onDelete,
    onApply,
    onMove,
    onImport
  } = $props<{
    group: PresetGroup | null;
    selectedPresetName?: string | null;
    search?: string;
    presetFilePath?: string | null;
    onSelect?: (name: string) => void;
    onCreate?: (name: string) => void;
    onRename?: (value: { oldName: string; newName: string }) => void;
    onDelete?: (name: string) => void;
    onApply?: (name: string) => void;
    onMove?: (value: { oldGroup: string; newGroup: string; name: string; targetIndex?: number }) => void;
    onImport?: () => void;
  }>();

  const PRESET_DRAG_TYPE = 'application/x-smart-eq-preset';

  let creating = $state(false);
  let newPresetName = $state('');
  let editingPresetName = $state<string | null>(null);
  let editingValue = $state('');

  let visiblePresets = $derived.by(() => {
    const list = group?.presets ?? [];
    const query = search.trim().toLowerCase();
    if (!query) {
      return list;
    }

    return list.filter((preset: PresetItem) => {
      return (
        preset.name.toLowerCase().includes(query) ||
        preset.content.toLowerCase().includes(query)
      );
    });
  });

  function groupLabel(value: PresetGroup | null) {
    if (!value) {
      return 'Presets';
    }

    return `${value.emoji ? `${value.emoji} ` : ''}${value.name}`;
  }

  function submitCreate() {
    const trimmed = newPresetName.trim();
    if (!trimmed) {
      return;
    }

    onCreate?.(trimmed);
    creating = false;
    newPresetName = '';
  }

  function startRename(name: string) {
    editingPresetName = name;
    editingValue = name;
  }

  function submitRename(oldName: string) {
    const trimmed = editingValue.trim();
    if (!trimmed || trimmed === oldName) {
      editingPresetName = null;
      editingValue = '';
      return;
    }

    onRename?.({ oldName, newName: trimmed });
    editingPresetName = null;
    editingValue = '';
  }

  function handleDragStart(event: DragEvent, name: string) {
    if (!group) {
      return;
    }

    const payload = JSON.stringify({ oldGroup: group.name, name });
    event.dataTransfer?.setData(PRESET_DRAG_TYPE, payload);
    event.dataTransfer?.setData('text/plain', payload);
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
    }
  }

  function handleDrop(event: DragEvent, targetIndex?: number) {
    event.preventDefault();
    if (!group) {
      return;
    }

    const payload = event.dataTransfer?.getData(PRESET_DRAG_TYPE);
    if (!payload) {
      return;
    }

    const selection = JSON.parse(payload) as MovePreset;
    onMove?.({
      oldGroup: selection.oldGroup,
      newGroup: group.name,
      name: selection.name,
      targetIndex
    });
  }

  function isPresetActive(name: string): boolean {
    return group?.activePreset === name;
  }

  function getPresetPreview(content: string): string {
    const normalized = content.replace(/\s+/g, ' ').trim();
    return normalized || 'Empty preset file';
  }
</script>

<section class="flex h-full min-h-0 flex-col overflow-hidden border-r border-border bg-surface">
  <div class="border-b border-border px-3 py-3">
    <div class="flex items-center justify-between gap-3">
      <div>
        <div class="text-sm font-medium text-foreground">
          {groupLabel(group)}
        </div>
        <div class="mt-0.5 text-xs text-muted">
          {group ? `${visiblePresets.length} visible` : 'Select a group'}
        </div>
        {#if presetFilePath}
          <div class="mt-1 truncate text-[11px] text-muted">
            File: <span class="font-mono text-foreground">{presetFilePath}</span>
          </div>
        {/if}
      </div>

      {#if group}
        <div class="flex items-center gap-2">
          <Button
            size="icon"
            variant="secondary"
            title="Import presets into this group"
            ariaLabel="Import presets into this group"
            onclick={() => onImport?.()}
          >
            <FolderInput size={16} />
          </Button>
          <Button
            size="icon"
            variant="secondary"
            title="Add preset"
            ariaLabel="Add preset"
            onclick={() => (creating = !creating)}
          >
            <FilePlus2 size={16} />
          </Button>
        </div>
      {/if}
    </div>

    {#if creating && group}
      <div class="mt-3 rounded-[10px] border border-border bg-surface-2 p-2">
        <Input
          bind:value={newPresetName}
          placeholder="Preset name"
          onkeydown={(event) => {
            if (event.key === 'Enter') submitCreate();
            if (event.key === 'Escape') creating = false;
          }}
        />
        <div class="mt-2 flex justify-end gap-2">
          <Button size="sm" variant="ghost" onclick={() => (creating = false)}>Cancel</Button>
          <Button size="sm" onclick={submitCreate}>Create</Button>
        </div>
      </div>
    {/if}
  </div>

  {#if !group}
    <div class="flex min-h-0 flex-1 items-center justify-center px-4 text-sm text-muted">
      Choose a group to edit its presets.
    </div>
  {:else}
    <div
      class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto p-3 pr-4 [scrollbar-gutter:stable]"
      role="list"
      aria-label="Presets"
      ondragover={(event) => event.preventDefault()}
      ondrop={(event) => handleDrop(event, visiblePresets.length)}
    >
      {#if visiblePresets.length === 0}
        <div class="rounded-[10px] border border-dashed border-border bg-surface-2 px-3 py-4 text-sm text-muted">
          No presets match the current search.
        </div>
      {/if}

      {#each visiblePresets as preset, index}
        <div
          role="listitem"
          draggable="true"
          ondragstart={(event) => handleDragStart(event, preset.name)}
          ondragover={(event) => event.preventDefault()}
          ondrop={(event) => handleDrop(event, index)}
          class={cn(
            'mb-3 rounded-[10px] border p-3 transition-colors',
            selectedPresetName === preset.name
              ? 'border-accent/60 bg-surface-2'
              : 'border-border bg-surface-2 hover:bg-surface-3'
          )}
        >
          {#if editingPresetName === preset.name}
            <Input
              bind:value={editingValue}
              onkeydown={(event) => {
                if (event.key === 'Enter') submitRename(preset.name);
                if (event.key === 'Escape') editingPresetName = null;
              }}
            />
            <div class="mt-2 flex justify-end gap-2">
              <Button size="sm" variant="ghost" onclick={() => (editingPresetName = null)}>Cancel</Button>
              <Button size="sm" onclick={() => submitRename(preset.name)}>Save</Button>
            </div>
          {:else}
            <div class="flex min-w-0 items-start gap-3">
              <button
                type="button"
                class={cn(
                  'focus-ring mt-0.5 inline-flex size-9 shrink-0 items-center justify-center rounded-[10px] border transition-all duration-150',
                  isPresetActive(preset.name)
                    ? 'border-accent/60 bg-accent/12 text-accent shadow-[0_0_0_1px_rgba(163,230,53,0.18),0_0_20px_rgba(163,230,53,0.18)]'
                    : 'border-border bg-background text-muted hover:border-accent/30 hover:bg-accent/8 hover:text-foreground'
                )}
                title={isPresetActive(preset.name) ? 'Active preset' : 'Activate preset'}
                aria-label={isPresetActive(preset.name) ? 'Active preset' : 'Activate preset'}
                aria-pressed={isPresetActive(preset.name)}
                onclick={(event) => {
                  event.stopPropagation();
                  onApply?.(preset.name);
                }}
              >
                <Zap size={14} />
              </button>

              <button
                type="button"
                onclick={() => onSelect?.(preset.name)}
                class="flex min-w-0 flex-1 items-start text-left"
              >
                <span class="min-w-0 flex-1">
                  <span class="flex items-center gap-2">
                    <span class="truncate text-sm font-medium text-foreground">{preset.name}</span>
                    {#if group.activePreset === preset.name}
                      <span class="inline-flex items-center gap-1 rounded-full border border-success/30 bg-success-soft px-2 py-0.5 text-[11px] text-success">
                        <CheckCircle2 size={12} />
                        Active
                      </span>
                    {/if}
                  </span>
                  <span class="mt-1 block truncate text-xs leading-5 text-muted">
                    {getPresetPreview(preset.content)}
                  </span>
                </span>
              </button>

              <div class="flex shrink-0 items-center gap-1">
                <Button size="icon" variant="ghost" onclick={() => startRename(preset.name)}>
                  <Pencil size={14} />
                </Button>
                <Button
                  size="icon"
                  variant="ghost"
                  class="text-danger hover:bg-danger-soft hover:text-danger"
                  onclick={() => onDelete?.(preset.name)}
                >
                  <Trash2 size={14} />
                </Button>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</section>
