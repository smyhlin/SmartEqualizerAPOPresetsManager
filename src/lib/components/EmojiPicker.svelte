<script lang="ts">
  import Button from '$lib/components/ui/button.svelte';
  import { cn } from '$lib/utils';

  const EMOJIS = [
    '🎚️',
    '🎛️',
    '🎧',
    '🎵',
    '🎼',
    '🔊',
    '🔉',
    '🔈',
    '🎹',
    '🎤',
    '🟢',
    '🟡',
    '🟣',
    '🔵',
    '⚪',
    '⚫'
  ] as const;

  let {
    selected = null,
    onPick,
    onClear
  } = $props<{
    selected?: string | null;
    onPick?: (emoji: string) => void;
    onClear?: () => void;
  }>();
</script>

<div class="shell-surface-2 mt-2 rounded-[10px] p-2">
  <div class="mb-2 flex items-center justify-between gap-2">
    <div class="text-[11px] font-semibold uppercase tracking-[0.16em] text-muted">Emoji</div>
    <Button size="sm" variant="ghost" onclick={() => onClear?.()}>
      Clear
    </Button>
  </div>

  <div class="grid grid-cols-8 gap-1">
    {#each EMOJIS as emoji}
      <button
        type="button"
        onclick={() => onPick?.(emoji)}
        class={cn(
          'focus-ring inline-flex h-8 items-center justify-center rounded-[8px] border text-[16px] transition-colors',
          selected === emoji
            ? 'border-accent/60 bg-accent-soft'
            : 'border-border bg-surface hover:bg-surface-3'
        )}
      >
        {emoji}
      </button>
    {/each}
  </div>
</div>
