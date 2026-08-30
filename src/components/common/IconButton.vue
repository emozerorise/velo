<template>
  <button
    :title="title"
    :disabled="disabled"
    :class="[
      'inline-flex items-center justify-center rounded-lg transition-colors duration-150 active:scale-95 no-drag',
      disabled
        ? 'text-fg/20 pointer-events-none'
        : active
          ? 'text-accent bg-blue-500/15'
          : 'text-fg/75 hover:text-fg hover:bg-fg/10',
      sizeClasses[size] || sizeClasses.md,
      customClass,
    ]"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
interface Props {
  title?: string;
  size?: 'sm' | 'md' | 'lg';
  active?: boolean;
  disabled?: boolean;
  customClass?: string;
}

withDefaults(defineProps<Props>(), {
  title: '',
  size: 'md',
  active: false,
  disabled: false,
  customClass: '',
});

defineEmits<{
  (e: 'click', event: MouseEvent): void;
}>();

const sizeClasses = {
  sm: 'p-1.5 text-xs',
  md: 'p-2 text-sm',
  lg: 'p-3 text-base',
};
</script>
