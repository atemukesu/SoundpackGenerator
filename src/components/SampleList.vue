<script setup>
import { computed, ref } from 'vue';
import { midiNoteName, getInstrumentName, getDrumKeyName } from '../constants/gm.js';

const props = defineProps({
  instrumentId: Number,
  samples: Array,
  playingNote: Number,
});

const emit = defineEmits(['play', 'edit', 'delete', 'add-custom', 'batch-edit', 'batch-delete']);

const instrumentName = computed(() => getInstrumentName(props.instrumentId));
const isDrumKit = computed(() => props.instrumentId === 128);

const deleteConfirmIndex = ref(-1);
const selectedNotes = ref(new Set());
const batchGain = ref(0);
const batchRegionStart = ref(0);
const batchRegionEnd = ref(0);
const batchUseRegion = ref(false);

const sortedSamples = computed(() => {
  return [...(props.samples || [])].sort((a, b) => a.note - b.note);
});

const allSelected = computed(() => {
  return sortedSamples.value.length > 0 && selectedNotes.value.size === sortedSamples.value.length;
});

const toggleAll = () => {
  if (allSelected.value) {
    selectedNotes.value = new Set();
  } else {
    selectedNotes.value = new Set(sortedSamples.value.map(s => s.note));
  }
};

const toggleNote = (note) => {
  const next = new Set(selectedNotes.value);
  if (next.has(note)) next.delete(note);
  else next.add(note);
  selectedNotes.value = next;
};

const handleDelete = (index) => {
  deleteConfirmIndex.value = -1;
  emit('delete', index);
};

const applyBatchEdit = () => {
  const notes = Array.from(selectedNotes.value);
  if (notes.length === 0) return;
  const region = batchUseRegion.value ? { start: batchRegionStart.value, end: batchRegionEnd.value } : null;
  emit('batch-edit', { notes, gain: batchGain.value, region });
  selectedNotes.value = new Set();
  batchGain.value = 0;
  batchUseRegion.value = false;
};

const confirmBatchDelete = ref(false);

const applyBatchDelete = () => {
  const notes = Array.from(selectedNotes.value);
  if (notes.length === 0) return;
  emit('batch-delete', notes);
  selectedNotes.value = new Set();
  confirmBatchDelete.value = false;
};
</script>

<template>
  <div class="flex-1 flex flex-col min-h-0">
    <div class="px-6 py-4 border-b border-gray-200 bg-white flex items-center justify-between flex-shrink-0">
      <div>
        <h2 class="text-sm font-semibold text-gray-800">乐器 {{ instrumentId }} - {{ instrumentName }} 样本</h2>
        <p class="text-xs text-gray-500 mt-0.5">已加载 {{ sortedSamples.length }} 个样本</p>
      </div>
      <div class="flex items-center gap-2">
        <label v-if="sortedSamples.length > 0" class="flex items-center gap-1.5 cursor-pointer text-xs text-gray-500 hover:text-gray-700">
          <input type="checkbox" :checked="allSelected" @change="toggleAll" class="w-3.5 h-3.5 rounded border-gray-300 text-blue-500 focus:ring-blue-400" />
          全选
        </label>
        <button @click="emit('add-custom')" class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-xs font-medium transition-colors">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
          </svg>
          添加音频
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto bg-gray-50">
      <div v-if="sortedSamples.length === 0" class="h-full flex items-center justify-center">
        <div class="text-center">
          <div class="w-16 h-16 mx-auto mb-3 rounded-xl bg-white border border-gray-200 flex items-center justify-center">
            <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"/>
            </svg>
          </div>
          <p class="text-sm text-gray-500">暂无样本。请从 SF2 生成或添加自定义音频。</p>
        </div>
      </div>

      <div v-else class="p-4 space-y-2">
        <div v-for="(sample, index) in sortedSamples" :key="sample.note" class="flex items-center gap-3 p-3 bg-white rounded-xl border border-gray-200 hover:border-gray-300 transition-colors">
          <input type="checkbox" :checked="selectedNotes.has(sample.note)" @change="toggleNote(sample.note)" class="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-400 flex-shrink-0" />

          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="text-xs font-mono text-gray-500 w-10">{{ sample.note }}</span>
              <span class="text-sm font-medium text-gray-800">{{ midiNoteName(sample.note) }}</span>
              <span v-if="isDrumKit" class="text-xs text-amber-600 font-medium">{{ getDrumKeyName(sample.note) }}</span>
              <span class="px-1.5 py-0.5 rounded text-xs" :class="sample.source === 'sf2' ? 'bg-blue-50 text-blue-600' : 'bg-green-50 text-green-600'">{{ sample.source === 'sf2' ? 'SF2' : '自定义' }}</span>
            </div>
            <p class="text-xs text-gray-400 mt-0.5 truncate">{{ sample.file }}</p>
          </div>

          <div class="flex items-center gap-1">
            <button @click="emit('play', sample)" class="p-1.5 rounded-lg hover:bg-gray-100 transition-colors" :class="playingNote === sample.note ? 'text-blue-500 bg-blue-50' : 'text-gray-500 hover:text-gray-700'" :title="playingNote === sample.note ? '暂停' : '播放'">
              <svg v-if="playingNote === sample.note" class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z"/></svg>
              <svg v-else class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
            </button>
            <button @click="emit('edit', sample)" class="p-1.5 rounded-lg hover:bg-gray-100 text-gray-500 hover:text-gray-700 transition-colors" title="编辑">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/></svg>
            </button>

            <div v-if="deleteConfirmIndex !== index">
              <button @click="deleteConfirmIndex = index" class="p-1.5 rounded-lg hover:bg-red-50 text-gray-500 hover:text-red-500 transition-colors" title="删除">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
              </button>
            </div>
            <div v-else class="flex items-center gap-1">
              <button @click="handleDelete(index)" class="px-2 py-1 rounded bg-red-500 hover:bg-red-600 text-white text-xs font-medium transition-colors">确认</button>
              <button @click="deleteConfirmIndex = -1" class="px-2 py-1 rounded bg-gray-200 hover:bg-gray-300 text-gray-700 text-xs font-medium transition-colors">取消</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 批量操作栏 -->
    <div v-if="selectedNotes.size > 0" class="flex-shrink-0 px-6 py-3 border-t border-gray-200 bg-white space-y-2">
      <div class="flex items-center justify-between">
        <span class="text-xs text-gray-500 font-medium">已选 {{ selectedNotes.size }} 个样本</span>
        <button @click="selectedNotes = new Set(); confirmBatchDelete = false" class="text-xs text-gray-400 hover:text-gray-600 transition-colors">取消选择</button>
      </div>

      <div class="flex items-center gap-4 flex-wrap">
        <!-- 增益 -->
        <div class="flex items-center gap-2 min-w-0">
          <label class="text-xs text-gray-500 whitespace-nowrap">增益:</label>
          <input v-model.number="batchGain" type="range" min="-20" max="20" step="0.5" class="w-24" />
          <span class="text-xs font-mono text-gray-600 w-10 text-right">{{ batchGain }} dB</span>
        </div>

        <!-- 区域裁剪 -->
        <label class="flex items-center gap-1.5 cursor-pointer text-xs text-gray-500 whitespace-nowrap">
          <input v-model="batchUseRegion" type="checkbox" class="w-3.5 h-3.5 rounded border-gray-300 text-blue-500 focus:ring-blue-400" />
          裁剪
        </label>
        <div v-if="batchUseRegion" class="flex items-center gap-2 text-xs text-gray-500">
          <input v-model.number="batchRegionStart" type="number" min="0" step="0.01" class="w-16 bg-gray-50 border border-gray-200 rounded px-1.5 py-1 text-xs font-mono text-gray-700 outline-none focus:border-blue-400" />
          <span>~</span>
          <input v-model.number="batchRegionEnd" type="number" min="0" step="0.01" class="w-16 bg-gray-50 border border-gray-200 rounded px-1.5 py-1 text-xs font-mono text-gray-700 outline-none focus:border-blue-400" />
          <span class="text-gray-400">秒</span>
        </div>

        <div class="flex-1"></div>

        <!-- 操作按钮 -->
        <div v-if="!confirmBatchDelete" class="flex items-center gap-2">
          <button @click="confirmBatchDelete = true" class="px-3 py-1.5 rounded-lg bg-red-50 hover:bg-red-100 text-red-600 text-xs font-medium transition-colors">批量删除</button>
          <button @click="applyBatchEdit" class="px-3 py-1.5 rounded-lg bg-blue-500 hover:bg-blue-600 text-white text-xs font-medium transition-colors">批量应用</button>
        </div>
        <div v-else class="flex items-center gap-2">
          <span class="text-xs text-red-600 font-medium">确认删除 {{ selectedNotes.size }} 个？</span>
          <button @click="applyBatchDelete" class="px-3 py-1.5 rounded-lg bg-red-500 hover:bg-red-600 text-white text-xs font-medium transition-colors">确认删除</button>
          <button @click="confirmBatchDelete = false" class="px-3 py-1.5 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-600 text-xs font-medium transition-colors">取消</button>
        </div>
      </div>
    </div>
  </div>
</template>
