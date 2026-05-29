<!--
 Copyright Atemukesu
 SPDX-License-Identifier: GPL-3.0
-->

<script setup>
import { ref, computed, watch } from 'vue';
import { INSTRUMENTS, getDrumKeyName } from '../constants/gm.js';

const props = defineProps({
  visible: Boolean,
  instrumentId: Number,
});

const emit = defineEmits(['close', 'generate']);

const sf2Path = ref('');
const noteStart = ref(48);
const noteEnd = ref(72);
const noteStep = ref(5);
const gain = ref(0);
const regionStart = ref(0);
const regionEnd = ref(0);
const useRegion = ref(false);
const maxCores = ref(4);

const selectedInstruments = ref([]);
const selectAll = ref(false);

const isGenerating = ref(false);

const hasDrumKit = computed(() => selectedInstruments.value.includes(128));

const noteList = computed(() => {
  const notes = [];
  for (let n = noteStart.value; n <= noteEnd.value; n += noteStep.value) {
    const drumName = hasDrumKit.value ? getDrumKeyName(n) : '';
    notes.push(drumName ? `${n} - ${drumName}` : n);
  }
  return notes;
});

const selectSf2 = async () => {
  const { open } = await import('@tauri-apps/plugin-dialog');
  const selected = await open({
    multiple: false,
    filters: [{ name: 'SoundFont', extensions: ['sf2', 'sf3'] }],
  });
  if (selected) sf2Path.value = selected;
};

const toggleAllInstruments = () => {
  if (selectAll.value) {
    selectedInstruments.value = INSTRUMENTS.map(i => i.id);
  } else {
    selectedInstruments.value = [];
  }
};

const handleGenerate = () => {
  if (!sf2Path.value || selectedInstruments.value.length === 0) return;
  isGenerating.value = true;

  emit('generate', {
    sf2Path: sf2Path.value,
    noteStart: noteStart.value,
    noteEnd: noteEnd.value,
    noteStep: noteStep.value,
    gain: gain.value,
    region: useRegion.value ? { start: regionStart.value, end: regionEnd.value } : null,
    instruments: selectedInstruments.value,
    maxCores: maxCores.value,
  });
};

watch(() => props.visible, (v) => {
  if (v) {
    sf2Path.value = '';
    noteStart.value = 48;
    noteEnd.value = 72;
    noteStep.value = 5;
    gain.value = 0;
    useRegion.value = false;
    regionStart.value = 0;
    regionEnd.value = 0;
    maxCores.value = 4;
    selectedInstruments.value = [];
    selectAll.value = false;
    isGenerating.value = false;
  }
});

watch(selectAll, toggleAllInstruments);
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div class="bg-white rounded-xl p-6 w-[800px] shadow-xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between mb-5">
          <div>
            <h2 class="text-lg font-semibold text-gray-800">从 SF2 生成</h2>
            <p class="text-xs text-gray-500 mt-0.5">批量生成音色样本</p>
          </div>
          <button @click="emit('close')" class="text-gray-500 hover:text-gray-700 transition-colors p-1">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>

        <div class="space-y-4">
          <div>
            <label class="block text-xs text-gray-500 mb-1.5">SF2 音源文件</label>
            <div class="flex gap-2">
              <input
                :value="sf2Path"
                readonly
                placeholder="选择 SF2 文件..."
                class="flex-1 bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-700 placeholder-gray-400 outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 transition-all"
              />
              <button @click="selectSf2" class="px-3 py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-xs font-medium transition-colors">浏览</button>
            </div>
          </div>

          <div class="bg-blue-50 rounded-lg p-3 border border-blue-100">
            <p class="text-xs text-blue-600 font-medium mb-1">生成规则</p>
            <p class="text-sm text-blue-700">每个乐器将使用对应的 Preset ID 生成（乐器 0 → Preset 0，乐器 128 → 鼓组）</p>
          </div>

          <div v-if="hasDrumKit" class="bg-amber-50 rounded-lg p-3 border border-amber-200">
            <p class="text-xs text-amber-600 font-medium mb-1">鼓组提示</p>
            <p class="text-sm text-amber-700">标准鼓组 (ID 128) 建议使用 35–81 的音符范围，采样间隔设为 1，每个音符对应不同打击乐器</p>
          </div>

          <div class="border-t border-gray-100 pt-4">
            <h3 class="text-xs font-medium text-gray-700 mb-3">音符采样设置</h3>
            <div class="grid grid-cols-3 gap-4">
              <div>
                <label class="block text-xs text-gray-500 mb-1.5">起始音符 (MIDI)</label>
                <input
                  v-model.number="noteStart"
                  type="number"
                  min="0"
                  max="127"
                  class="w-full bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-700 outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 transition-all"
                />
              </div>
              <div>
                <label class="block text-xs text-gray-500 mb-1.5">结束音符 (MIDI)</label>
                <input
                  v-model.number="noteEnd"
                  type="number"
                  min="0"
                  max="127"
                  class="w-full bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-700 outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 transition-all"
                />
              </div>
              <div>
                <label class="block text-xs text-gray-500 mb-1.5">采样间隔</label>
                <input
                  v-model.number="noteStep"
                  type="number"
                  min="1"
                  max="127"
                  class="w-full bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-700 outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 transition-all"
                />
              </div>
            </div>
            <p class="text-xs text-gray-400 mt-2">
              将生成 {{ noteList.length }} 个音符:
              <span v-if="noteList.length <= 60">{{ noteList.join(', ') }}</span>
              <span v-else>{{ noteList.slice(0, 30).join(', ') }}...（共 {{ noteList.length }} 个）</span>
            </p>
          </div>

          <div>
            <label class="block text-xs text-gray-500 mb-1.5">增益 (dB)</label>
            <div class="flex items-center gap-3">
              <input
                v-model.number="gain"
                type="range"
                min="-20"
                max="20"
                step="0.5"
                class="flex-1"
              />
              <span class="text-xs font-mono text-gray-600 w-12 text-right">{{ gain }} dB</span>
            </div>
          </div>

          <div>
            <label class="block text-xs text-gray-500 mb-1.5">最大并发核心数</label>
            <div class="flex items-center gap-3">
              <input
                v-model.number="maxCores"
                type="range"
                min="1"
                max="16"
                step="1"
                class="flex-1"
              />
              <span class="text-xs font-mono text-gray-600 w-8 text-right">{{ maxCores }}</span>
            </div>
          </div>

          <div class="border-t border-gray-100 pt-3">
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                v-model="useRegion"
                type="checkbox"
                class="w-4 h-4 rounded-md border-gray-300 text-blue-500 focus:ring-blue-400"
              />
              <span class="text-xs text-gray-600">使用自定义音频区域</span>
            </label>

            <div v-if="useRegion" class="grid grid-cols-2 gap-4 mt-3">
              <div>
                <label class="block text-xs text-gray-500 mb-1.5">起始时间 (秒)</label>
                <input
                  v-model.number="regionStart"
                  type="number"
                  min="0"
                  step="0.01"
                  class="w-full bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-700 outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 transition-all"
                />
              </div>
              <div>
                <label class="block text-xs text-gray-500 mb-1.5">结束时间 (秒)</label>
                <input
                  v-model.number="regionEnd"
                  type="number"
                  min="0"
                  step="0.01"
                  class="w-full bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-700 outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 transition-all"
                />
              </div>
            </div>
          </div>

          <div class="border-t border-gray-100 pt-4">
            <div class="flex items-center justify-between mb-3">
              <h3 class="text-xs font-medium text-gray-700">选择乐器</h3>
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="selectAll"
                  type="checkbox"
                  class="w-4 h-4 rounded-md border-gray-300 text-blue-500 focus:ring-blue-400"
                />
                <span class="text-xs text-gray-500">全选</span>
              </label>
            </div>
            <div class="max-h-40 overflow-y-auto bg-gray-50 rounded-lg border border-gray-200 p-2">
              <div class="grid grid-cols-2 gap-1">
                <label
                  v-for="inst in INSTRUMENTS"
                  :key="inst.id"
                  class="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-gray-100 cursor-pointer"
                >
                  <input
                    v-model="selectedInstruments"
                    :value="inst.id"
                    type="checkbox"
                    class="w-3.5 h-3.5 rounded-md border-gray-300 text-blue-500 focus:ring-blue-400"
                  />
                  <span class="text-xs text-gray-600">{{ inst.id }} - {{ inst.name }}</span>
                </label>
              </div>
            </div>
            <p class="text-xs text-gray-400 mt-1">已选择 {{ selectedInstruments.length }} 个乐器</p>
          </div>
        </div>

        <div class="flex justify-end gap-3 mt-6 pt-4 border-t border-gray-100">
          <button @click="emit('close')" class="px-4 py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-sm font-medium transition-colors">取消</button>
          <button
            @click="handleGenerate"
            :disabled="!sf2Path || isGenerating || selectedInstruments.length === 0"
            class="px-4 py-2 rounded-lg bg-blue-500 hover:bg-blue-600 text-white text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {{ isGenerating ? '生成中...' : '开始生成' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
