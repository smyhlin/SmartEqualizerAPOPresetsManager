<script lang="ts">
  import { FolderPlus, Pencil, Trash2 } from '@lucide/svelte';

  import EmojiPicker from '$lib/components/EmojiPicker.svelte';
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
    groups = [],
    selectedGroupName = null,
    search = '',
    onSelect,
    onCreate,
    onRename,
    onDelete,
    onReorder,
    onMovePreset,
    onEmojiChange
  } = $props<{
    groups: PresetGroup[];
    selectedGroupName?: string | null;
    search?: string;
    onSelect?: (name: string) => void;
    onCreate?: (value: { name: string; emoji: string | null }) => void;
    onRename?: (value: { oldName: string; newName: string }) => void;
    onDelete?: (name: string) => void;
    onReorder?: (order: string[]) => void;
    onMovePreset?: (value: MovePreset) => void;
    onEmojiChange?: (value: { groupName: string; emoji: string | null }) => void;
  }>();

  const GROUP_DRAG_TYPE = 'application/x-smart-eq-group';
  const PRESET_DRAG_TYPE = 'application/x-smart-eq-preset';

  let creating = $state(false);
  let newGroupName = $state('');
  let newGroupEmoji = $state<string | null>(null);
  let creatingEmojiOpen = $state(false);
  let editingGroupName = $state<string | null>(null);
  let editingValue = $state('');
  let emojiPickerGroupName = $state<string | null>(null);

  let visibleGroups = $derived.by(() => {
    const query = search.trim().toLowerCase();
    if (!query) {
      return groups;
    }

    return groups.filter((group: PresetGroup) => {
      return (
        group.name.toLowerCase().includes(query) ||
        group.presets.some((preset: PresetItem) => {
          return (
            preset.name.toLowerCase().includes(query) ||
            preset.content.toLowerCase().includes(query)
          );
        })
      );
    });
  });

  function startCreate() {
    creating = true;
    newGroupName = '';
    newGroupEmoji = null;
    creatingEmojiOpen = false;
    emojiPickerGroupName = null;
  }

  function submitCreate() {
    const trimmed = newGroupName.trim();
    if (!trimmed) {
      return;
    }

    onCreate?.({ name: trimmed, emoji: newGroupEmoji });
    creating = false;
    creatingEmojiOpen = false;
    newGroupName = '';
    newGroupEmoji = null;
  }

  function startRename(groupName: string) {
    editingGroupName = groupName;
    editingValue = groupName;
  }

  function submitRename(groupName: string) {
    const trimmed = editingValue.trim();
    if (!trimmed || trimmed === groupName) {
      editingGroupName = null;
      editingValue = '';
      return;
    }

    onRename?.({ oldName: groupName, newName: trimmed });
    editingGroupName = null;
    editingValue = '';
  }

  function toggleCreatingEmojiPicker() {
    creatingEmojiOpen = !creatingEmojiOpen;
  }

  function toggleGroupEmojiPicker(groupName: string) {
    emojiPickerGroupName = emojiPickerGroupName === groupName ? null : groupName;
  }

  function setGroupEmoji(groupName: string, emoji: string | null) {
    onEmojiChange?.({ groupName, emoji });
    emojiPickerGroupName = null;
  }

  function handleDragStart(event: DragEvent, groupName: string) {
    event.dataTransfer?.setData(GROUP_DRAG_TYPE, groupName);
    event.dataTransfer?.setData('text/plain', groupName);
    if (event.dataTransfer && event.currentTarget instanceof HTMLElement) {
      event.dataTransfer.setDragImage(event.currentTarget, 14, 14);
    }
  }

  function handleDrop(event: DragEvent, targetGroupName: string) {
    event.preventDefault();

    const presetPayload = event.dataTransfer?.getData(PRESET_DRAG_TYPE);
    if (presetPayload) {
      const selection = JSON.parse(presetPayload) as MovePreset;
      onMovePreset?.({
        oldGroup: selection.oldGroup,
        newGroup: targetGroupName,
        name: selection.name
      });
      return;
    }

    const draggedGroupName = event.dataTransfer?.getData(GROUP_DRAG_TYPE);
    if (!draggedGroupName || draggedGroupName === targetGroupName) {
      return;
    }

    const nextOrder = groups.map((group: PresetGroup) => group.name);
    const sourceIndex = nextOrder.indexOf(draggedGroupName);
    const targetIndex = nextOrder.indexOf(targetGroupName);
    if (sourceIndex < 0 || targetIndex < 0) {
      return;
    }

    nextOrder.splice(sourceIndex, 1);
    nextOrder.splice(targetIndex, 0, draggedGroupName);
    onReorder?.(nextOrder);
  }
</script>

<aside class="flex h-full min-h-0 flex-col overflow-hidden border-r border-border bg-surface">
  <div class="border-b border-border px-3 py-3">
    <div class="flex items-center justify-between gap-3">
      <div>
        <div class="text-sm font-medium text-foreground">Groups</div>
        <div class="mt-0.5 text-xs text-muted">{visibleGroups.length} visible</div>
      </div>

      <Button size="icon" variant="secondary" onclick={startCreate}>
        <FolderPlus size={16} />
      </Button>
    </div>

    {#if creating}
      <div class="mt-3 rounded-[10px] border border-border bg-surface-2 p-2">
        <Input
          bind:value={newGroupName}
          placeholder="Group name"
          onkeydown={(event) => {
            if (event.key === 'Enter') submitCreate();
            if (event.key === 'Escape') creating = false;
          }}
        />
        <div class="mt-2 flex justify-end gap-2">
          <Button size="sm" variant="ghost" onclick={() => (creating = false)}>Cancel</Button>
          <Button size="sm" onclick={submitCreate}>Create</Button>
        </div>
        <div class="mt-2 flex items-center gap-2">
          <Button size="icon" variant="outline" onclick={toggleCreatingEmojiPicker}>
            {newGroupEmoji ?? '＋'}
          </Button>
          <div class="text-xs text-muted">
            {newGroupEmoji ? 'Emoji selected' : 'Pick an emoji for this group'}
          </div>
        </div>
        {#if creatingEmojiOpen}
          <EmojiPicker
            selected={newGroupEmoji}
            onPick={(emoji) => (newGroupEmoji = emoji)}
            onClear={() => (newGroupEmoji = null)}
          />
        {/if}
      </div>
    {/if}
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto p-2" role="list" aria-label="Groups">
    {#if visibleGroups.length === 0}
      <div class="rounded-[10px] border border-dashed border-border bg-surface-2 px-3 py-4 text-sm text-muted">
        No groups match the current search.
      </div>
    {/if}

    {#each visibleGroups as group}
      <div
        role="listitem"
        draggable="true"
        ondragstart={(event) => handleDragStart(event, group.name)}
        ondragover={(event) => event.preventDefault()}
        ondrop={(event) => handleDrop(event, group.name)}
        class={cn(
          'mb-2 rounded-[10px] border p-2 transition-colors',
          selectedGroupName === group.name
            ? 'border-accent/60 bg-surface-2'
            : 'border-border bg-surface-2 hover:bg-surface-3'
        )}
      >
        {#if editingGroupName === group.name}
          <Input
            bind:value={editingValue}
            onkeydown={(event) => {
              if (event.key === 'Enter') submitRename(group.name);
              if (event.key === 'Escape') editingGroupName = null;
            }}
          />
          <div class="mt-2 flex justify-end gap-2">
            <Button size="sm" variant="ghost" onclick={() => (editingGroupName = null)}>Cancel</Button>
            <Button size="sm" onclick={() => submitRename(group.name)}>Save</Button>
          </div>
        {:else}
          <div class="flex items-start gap-2">
            <Button
              size="icon"
              variant="outline"
              class="shrink-0"
              title={`Change icon for ${group.name}`}
              ariaLabel={`Change icon for ${group.name}`}
              onclick={() => toggleGroupEmojiPicker(group.name)}
            >
              {group.emoji ?? '＋'}
            </Button>

            <button
              type="button"
              onclick={() => onSelect?.(group.name)}
              class="min-w-0 flex-1 text-left"
            >
              <span class="block truncate text-sm font-medium text-foreground">{group.name}</span>
              <span class="mt-1 flex min-w-0 items-center gap-2 text-xs text-muted">
                <span class="shrink-0">{group.presets.length} presets</span>
                {#if group.activePreset}
                  <span class="shrink-0 rounded-full border border-accent/30 bg-accent-soft px-1.5 py-0 text-[10px] uppercase tracking-wider text-accent">
                    Active
                  </span>
                {/if}
              </span>
            </button>

            <div class="flex items-center gap-1">
              <Button size="icon" variant="ghost" onclick={() => startRename(group.name)}>
                <Pencil size={14} />
              </Button>
              <Button
                size="icon"
                variant="ghost"
                class="text-danger hover:bg-danger-soft hover:text-danger"
                onclick={() => onDelete?.(group.name)}
              >
                <Trash2 size={14} />
              </Button>
            </div>
          </div>
          {#if emojiPickerGroupName === group.name}
            <EmojiPicker
              selected={group.emoji}
              onPick={(emoji) => setGroupEmoji(group.name, emoji)}
              onClear={() => setGroupEmoji(group.name, null)}
            />
          {/if}
        {/if}
      </div>
    {/each}
  </div>
</aside>
