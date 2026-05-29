<script setup>
import { ref, watch } from 'vue';
import WaveformEditor from './WaveformEditor.vue';

const props = defineProps({
  visible: Boolean,
  audioUrl: String,
  sampleInfo: Object,
  initialRegion: Object,
});

const emit = defineEmits(['close', 'save', 'update:region']);

const gain = ref(0);
const region = ref(null);

watch(() => props.visible, (v) => {
  if (v) {
    gain.value = 0;
    region.value = props.initialRegion ? { ...props.initialRegion } : null;
  }
});

const handleSave = () => {
  emit('save', { gain: gain.value, region: region.value });
};

const handleRegionUpdate = (r) => {
  region.value = r;
  emit('update:region', r);
};
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div class="bg-white rounded-xl p-6 w-[760px] shadow-xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between mb-5">
          <div>
            <h2 class="text-lg font-semibold text-gray-800">编辑样本</h2>
            <p class="text-xs text-gray-500 mt-0.5">
              乐器 {{ sampleInfo?.note }} - {{ sampleInfo?.file }}
            </p>
          </div>
          <button @click="emit('close')" class="text-gray-500 hover:text-gray-700 transition-colors p-1">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>

        <div class="space-y-4">
          <WaveformEditor
            :audioUrl="audioUrl"
            :initialRegion="initialRegion"
            @update:region="handleRegionUpdate"
          />

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
        </div>

        <div class="flex justify-end gap-3 mt-6 pt-4 border-t border-gray-100">
          <button @click="emit('close')" class="px-4 py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-sm font-medium transition-colors">取消</button>
          <button @click="handleSave" class="px-4 py-2 rounded-lg bg-blue-500 hover:bg-blue-600 text-white text-sm font-medium transition-colors">保存更改</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
