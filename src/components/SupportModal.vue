<!--
 Copyright Atemukesu
 SPDX-License-Identifier: GPL-3.0
-->

<script setup>
const props = defineProps({
  visible: Boolean,
});

const emit = defineEmits(['close']);

const sponsorUrl = 'https://www.ifdian.net/a/atommix';
const authorName = 'Atemukesu';
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div class="bg-white rounded-xl p-6 w-[420px] shadow-xl">
        <div class="flex items-center justify-between mb-5">
          <h2 class="text-lg font-semibold text-gray-800">赞助支持</h2>
          <button @click="emit('close')" class="text-gray-500 hover:text-gray-700 transition-colors p-1">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>

        <div class="text-center space-y-4">
          <div class="w-16 h-16 mx-auto rounded-full bg-pink-100 flex items-center justify-center">
            <svg class="w-8 h-8 text-pink-500" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
            </svg>
          </div>
          <p class="text-sm text-gray-600">如果这个工具对你有帮助，欢迎赞助支持我！</p>
          <a :href="sponsorUrl" target="_blank" rel="noopener noreferrer"
            class="inline-flex items-center gap-2 px-6 py-3 rounded-xl bg-pink-500 hover:bg-pink-600 text-white font-medium transition-colors shadow-lg shadow-pink-500/20">
            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
            </svg>
            去爱发电支持
          </a>
          <p class="text-xs text-gray-400">作者: {{ authorName }}</p>
        </div>
      </div>
    </div>
  </Teleport>
</template>
