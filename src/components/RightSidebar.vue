<script setup>
import { ref, computed } from 'vue';
import { INSTRUMENTS } from '../constants/gm.js';

defineProps({
  selectedId: {
    type: Number,
    default: 0,
  },
});

const emit = defineEmits(['select']);

const searchQuery = ref('');

const filteredInstruments = computed(() => {
  if (!searchQuery.value) return INSTRUMENTS;
  const query = searchQuery.value.toLowerCase();
  return INSTRUMENTS.filter((inst) => inst.name.toLowerCase().includes(query));
});
</script>

<template>
  <aside class="w-56 flex flex-col h-full bg-white border-r border-gray-200">
    <div class="px-4 py-3 border-b border-gray-200">
      <h2 class="text-xs font-semibold uppercase tracking-widest text-gray-500">乐器</h2>
    </div>
    
    <div class="px-3 py-2 border-b border-gray-100">
      <div class="relative">
        <svg class="absolute left-2 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
        </svg>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索..."
          class="w-full bg-gray-50 border border-gray-200 rounded-lg pl-8 pr-3 py-1.5 text-xs text-gray-700 placeholder-gray-400 outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 transition-all"
        />
      </div>
    </div>
    
    <div class="flex-1 overflow-y-auto">
      <div
        v-for="inst in filteredInstruments"
        :key="inst.id"
        @click="emit('select', inst.id)"
        :class="[
          'flex items-center gap-2 px-3 py-2 cursor-pointer transition-colors text-sm',
          selectedId === inst.id ? 'bg-blue-50 text-blue-700' : 'text-gray-600 hover:bg-gray-50'
        ]"
      >
        <span class="w-8 text-right text-xs font-mono text-gray-400">{{ inst.id }}</span>
        <span class="truncate">{{ inst.name }}</span>
      </div>
    </div>
    
    <div class="px-4 py-2 border-t border-gray-200 bg-gray-50">
      <div class="flex items-center justify-between text-xs text-gray-500">
        <span>{{ filteredInstruments.length }} 个乐器</span>
        <span>GM 标准</span>
      </div>
    </div>
  </aside>
</template>