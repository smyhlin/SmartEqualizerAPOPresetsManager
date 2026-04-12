<script lang="ts">
  import { FolderPlus, GripVertical, Pencil, Trash2 } from '@lucide/svelte';
  import { draggable, droppable, type DragDropState } from '@thisux/sveltednd';

  import EmojiPicker from '$lib/components/EmojiPicker.svelte';
  import Button from '$lib/components/ui/button.svelte';
  import Input from '$lib/components/ui/input.svelte';
  import type { PresetGroup, PresetItem } from '$lib/types';
  import { cn } from '$lib/utils';

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
    onMovePreset?: (value: { oldGroup: string; newGroup: string; name: string }) => void;
    onEmojiChange?: (value: { groupName: string; emoji: string | null }) => void;
  }>();

  const GROUP_CONTAINER = 'sidebar-groups';

  let creating = $state(false);
  let newGroupName = $state('');
  let newGroupEmoji = $state<string | null>(null);
  let creatingEmojiOpen = $state(false);
  let editingGroupName = $state<string | null>(null);
  let editingValue = $state('');
  let emojiPickerGroupName = $state<string | null>(null);

  let visibleGroups = $derived.by(() => {
    const query = search.trim().toLowerCase();
    if (!query) return groups;
    return groups.filter((g: PresetGroup) =>
      g.name.toLowerCase().includes(query) ||
      g.presets.some((p: PresetItem) =>
        p.name.toLowerCase().includes(query) || p.content.toLowerCase().includes(query)
      )
    );
  });

  // ── Create / rename ────────────────────────────────────────────────────────

  function startCreate() {
    creating = true;
    newGroupName = '';
    newGroupEmoji = null;
    creatingEmojiOpen = false;
    emojiPickerGroupName = null;
  }

  function submitCreate() {
    const trimmed = newGroupName.trim();
    if (!trimmed) return;
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

  // ── Group reorder drop ─────────────────────────────────────────────────────

  function handleGroupDrop(state: DragDropState<PresetGroup>) {
    const { draggedItem, targetContainer } = state;
    if (!targetContainer || targetContainer !== GROUP_CONTAINER) return;

    const targetGroupName = state.targetElement?.closest('[data-group-name]')?.getAttribute('data-group-name');
    if (!targetGroupName || targetGroupName === draggedItem.name) return;

    const nextOrder = groups.map((g: PresetGroup) => g.name);
    const sourceIndex = nextOrder.indexOf(draggedItem.name);
    const targetIndex = nextOrder.indexOf(targetGroupName);
    if (sourceIndex < 0 || targetIndex < 0) return;

    nextOrder.splice(sourceIndex, 1);
    nextOrder.splice(targetIndex, 0, draggedItem.name);
    onReorder?.(nextOrder);
  }

  // ── Preset drop onto group card ───────────────────────────────────────────
  // Each group card is a droppable zone with container ID = `drop-group-<name>`.
  // PresetWorkspace drags use `presets-<groupName>` as source container.

  function handlePresetDropOnGroup(targetGroupName: string, state: DragDropState<PresetItem>) {
    const { draggedItem, sourceContainer } = state;

    const oldGroup = sourceContainer.startsWith('presets-')
      ? sourceContainer.slice('presets-'.length)
      : sourceContainer;

    if (oldGroup === targetGroupName) return;

    onMovePreset?.({
      oldGroup,
      newGroup: targetGroupName,
      name: draggedItem.name,
    });
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
          onkeydown={(e) => { if (e.key === 'Enter') submitCreate(); if (e.key === 'Escape') creating = false; }}
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

    {#each visibleGroups as group (group.name)}
      <!--
        Each group card:
        - draggable (for group reordering) via the grip handle only
        - droppable (for preset cross-group move) via its own unique container ID
      -->
      <div
        data-group-name={group.name}
        use:draggable={{
          container: GROUP_CONTAINER,
          dragData: group,
          handle: '.group-drag-handle',
          attributes: { draggingClass: 'opacity-30 grayscale' }
        }}
        use:droppable={{
          container: `drop-group-${group.name}`,
          callbacks: {
            onDrop: (state: DragDropState<PresetItem>) => handlePresetDropOnGroup(group.name, state)
          },
          attributes: { dragOverClass: 'border-accent bg-accent/10 shadow-[inset_3px_0_0_0_var(--color-accent)]' }
        }}
        role="listitem"
        class={cn(
          'mb-2 rounded-[10px] border p-2 transition-colors duration-150',
          selectedGroupName === group.name
            ? 'border-accent/60 bg-surface-2'
            : 'border-border bg-surface-2 hover:bg-surface-3'
        )}
      >
        {#if editingGroupName === group.name}
          <Input
            bind:value={editingValue}
            onkeydown={(e) => {
              if (e.key === 'Enter') submitRename(group.name);
              if (e.key === 'Escape') editingGroupName = null;
            }}
          />
          <div class="mt-2 flex justify-end gap-2">
            <Button size="sm" variant="ghost" onclick={() => (editingGroupName = null)}>Cancel</Button>
            <Button size="sm" onclick={() => submitRename(group.name)}>Save</Button>
          </div>
        {:else}
          <div class="flex items-center gap-2">
            <!-- Grip handle: only this initiates a GROUP drag -->
            <div
              class="group-drag-handle shrink-0 cursor-grab touch-none text-muted/40 hover:text-muted active:cursor-grabbing"
              aria-label="Drag to reorder group"
            >
              <GripVertical size={14} />
            </div>

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
