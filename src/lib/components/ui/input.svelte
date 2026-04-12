<script lang="ts">
  import { cn } from '$lib/utils';

  let {
    type = 'text',
    value = $bindable(''),
    placeholder = '',
    class: className = '',
    disabled = false,
    oninput,
    onkeydown,
    onfocus,
    onblur
  } = $props<{
    type?: string;
    value?: string;
    placeholder?: string;
    class?: string;
    disabled?: boolean;
    oninput?: (event: Event) => void;
    onkeydown?: (event: KeyboardEvent) => void;
    onfocus?: (event: FocusEvent) => void;
    onblur?: (event: FocusEvent) => void;
  }>();
</script>

<input
  bind:value
  {type}
  {placeholder}
  {disabled}
  {oninput}
  {onkeydown}
  {onfocus}
  {onblur}
  class={cn(
    'focus-ring h-9 w-full rounded-[10px] border border-border bg-surface-2 px-3 text-sm text-foreground placeholder:text-muted',
    className
  )}
/>
