<script lang="ts">
  import { cn } from '$lib/utils';

  let {
    value = $bindable(''),
    rows = 18,
    placeholder = '',
    spellcheck,
    autocomplete,
    autocapitalize,
    wrap,
    readonly = false,
    class: className = '',
    oninput,
    onkeydown,
    onfocus,
    onblur
  } = $props<{
    value?: string;
    rows?: number;
    placeholder?: string;
    spellcheck?: boolean;
    autocomplete?: string;
    autocapitalize?: 'off' | 'none' | 'on' | 'sentences' | 'words' | 'characters';
    wrap?: 'soft' | 'hard' | 'off';
    readonly?: boolean;
    class?: string;
    oninput?: (event: Event) => void;
    onkeydown?: (event: KeyboardEvent) => void;
    onfocus?: (event: FocusEvent) => void;
    onblur?: (event: FocusEvent) => void;
  }>();
</script>

<textarea
  bind:value
  {rows}
  {placeholder}
  {spellcheck}
  {autocomplete}
  {autocapitalize}
  {wrap}
  {readonly}
  {oninput}
  {onkeydown}
  {onfocus}
  {onblur}
  class={cn(
    'focus-ring w-full resize-none rounded-[10px] border border-border bg-surface-2 px-3 py-2 font-mono text-[13px] leading-6 text-foreground placeholder:text-muted',
    className
  )}
></textarea>
