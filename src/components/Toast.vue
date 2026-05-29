<script setup>
import { ref, onMounted } from 'vue';

const toasts = ref([]);
let idCounter = 0;

const addToast = (message, type = 'error') => {
  const id = idCounter++;
  toasts.value.push({ id, message, type });
  setTimeout(() => {
    removeToast(id);
  }, 4000);
};

const removeToast = (id) => {
  toasts.value = toasts.value.filter(t => t.id !== id);
};

defineExpose({ addToast });

onMounted(() => {
  window.$showToast = (message, type = 'error') => addToast(message, type);
});
</script>

<template>
  <div class="fixed top-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="pointer-events-auto flex items-center gap-3 px-4 py-3 rounded-xl shadow-lg border max-w-sm"
        :class="toast.type === 'error' ? 'bg-red-50 border-red-200' : toast.type === 'warning' ? 'bg-amber-50 border-amber-200' : 'bg-green-50 border-green-200'"
      >
        <svg
          v-if="toast.type === 'error'"
          class="w-5 h-5 text-red-500 flex-shrink-0"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <svg
          v-else-if="toast.type === 'warning'"
          class="w-5 h-5 text-amber-500 flex-shrink-0"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01M10.29 3.86l-8.6 14.86A2 2 0 003.4 21h17.2a2 2 0 001.71-3.14l-8.6-14.86a2 2 0 00-3.42 0z"/>
        </svg>
        <svg
          v-else
          class="w-5 h-5 text-green-500 flex-shrink-0"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <p class="text-sm text-gray-700 flex-1">{{ toast.message }}</p>
        <button @click="removeToast(toast.id)" class="text-gray-400 hover:text-gray-600 flex-shrink-0">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-enter-active {
  transition: all 0.3s ease-out;
}
.toast-leave-active {
  transition: all 0.2s ease-in;
}
.toast-enter-from {
  opacity: 0;
  transform: translateX(20px);
}
.toast-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
.toast-move {
  transition: transform 0.3s ease;
}
</style>