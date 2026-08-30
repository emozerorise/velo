<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="isOpen"
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70"
        @click.self="$emit('close')"
      >
        <div
          class="relative w-full max-w-lg overflow-hidden bg-surface border border-fg/10 rounded-2xl shadow-[0_24px_60px_-16px_rgba(0,0,0,0.95)] text-fg animate-in fade-in zoom-in-95 duration-150"
        >
          <!-- Header -->
          <div class="flex items-center justify-between px-6 py-4 border-b border-fg/10">
            <h3 class="text-[15px] font-semibold text-fg">
              {{ title }}
            </h3>
            <button
              class="p-1 rounded-lg text-fg/50 hover:text-fg hover:bg-fg/10 transition-colors"
              @click="$emit('close')"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- Body -->
          <div class="px-6 py-4 max-h-[70vh] overflow-y-auto">
            <slot />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
defineProps<{
  isOpen: boolean;
  title: string;
}>();

defineEmits<{
  (e: 'close'): void;
}>();
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
