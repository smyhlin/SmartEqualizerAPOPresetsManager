<script lang="ts">
  import type { Snippet } from 'svelte';

  import { cn } from '$lib/utils';

  type Variant = 'default' | 'secondary' | 'ghost' | 'outline' | 'danger';
  type Size = 'default' | 'sm' | 'icon';

  type ButtonProps = {
    type?: 'button' | 'submit' | 'reset';
    variant?: Variant;
    size?: Size;
    disabled?: boolean;
    class?: string;
    title?: string;
    ariaLabel?: string;
    onclick?: (event: MouseEvent) => void;
    onkeydown?: (event: KeyboardEvent) => void;
    children?: Snippet;
  };

  let {
    type = 'button',
    variant = 'default',
    size = 'default',
    disabled = false,
    class: className = '',
    title,
    ariaLabel,
    onclick,
    onkeydown,
    children
  }: ButtonProps = $props();

  const variants: Record<Variant, string> = {
    default: 'bg-accent text-accent-foreground hover:bg-lime-400',
    secondary: 'bg-surface-2 text-foreground hover:bg-surface-3',
    ghost: 'bg-transparent text-muted hover:bg-surface-2 hover:text-foreground',
    outline: 'border border-border bg-surface text-foreground hover:bg-surface-2',
    danger: 'bg-danger text-white hover:bg-red-500'
  };

  const sizes: Record<Size, string> = {
    default: 'h-9 px-3.5 text-sm',
    sm: 'h-8 px-3 text-xs',
    icon: 'h-9 w-9 px-0'
  };
</script>

<button
  {type}
  {disabled}
  {title}
  aria-label={ariaLabel}
  {onclick}
  {onkeydown}
  class={cn(
    'focus-ring inline-flex items-center justify-center gap-2 rounded-[10px] border border-transparent font-medium transition-colors duration-150 disabled:pointer-events-none disabled:opacity-45',
    variants[variant as Variant],
    sizes[size as Size],
    className
  )}
>
  {@render children?.()}
</button>
