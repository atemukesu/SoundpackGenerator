<script setup>
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import WaveSurfer from 'wavesurfer.js';
import RegionsPlugin from 'wavesurfer.js/dist/plugins/regions.esm.js';

const props = defineProps({
  audioUrl: String,
  initialRegion: Object,
});

const emit = defineEmits(['update:region']);

const waveformRef = ref(null);
const currentTime = ref(0);
const duration = ref(0);
const regionStart = ref(0);
const regionEnd = ref(0);
const isSeeking = ref(false);
const isPlaying = ref(false);
const isLooping = ref(false);
let wavesurfer = null;
let wsRegions = null;
let regionUpdating = false;

const durationMs = computed(() => Math.max(1, Math.round((duration.value || 0) * 1000)));
const currentMs = computed(() => Math.round(currentTime.value * 1000));

onMounted(() => {
  wavesurfer = WaveSurfer.create({
    container: waveformRef.value,
    waveColor: '#3b82f6',
    progressColor: '#2563eb',
    cursorColor: '#f59e0b',
    barWidth: 3,
    barGap: 1,
    barRadius: 2,
    responsive: true,
    height: 200,
    normalize: false,
    barHeight: 10,
    partialRender: true,
    backgroundColor: '#ffffff',
    minPxPerSec: 50,
    autoCenter: false,
    fillParent: false,
    cursorWidth: 1,
  });

  wsRegions = wavesurfer.registerPlugin(RegionsPlugin.create());

  wsRegions.on('region-updated', (r) => {
    if (regionUpdating) return;
    regionStart.value = r.start;
    regionEnd.value = r.end;
    emit('update:region', { start: r.start, end: r.end });
  });

  wavesurfer.on('timeupdate', (time) => {
    if (!isSeeking.value) currentTime.value = time;
    // 检查是否需要循环
    if (isLooping.value && wsRegions) {
      const regions = wsRegions.getRegions();
      if (regions.length > 0) {
        const r = regions[0];
        // 如果接近选区末尾（距离小于 0.02 秒），则跳回选区开头
        if (time >= r.end - 0.02 && time <= r.end + 0.02) {
          wavesurfer.setTime(r.start);
        }
      }
    }
  });

  wavesurfer.on('play', () => {
    isPlaying.value = true;
  });

  wavesurfer.on('pause', () => {
    isPlaying.value = false;
    isLooping.value = false;
  });

  wavesurfer.on('finish', () => {
    isPlaying.value = false;
    const regions = wsRegions?.getRegions();
    if (regions && regions.length > 0 && isLooping.value) {
      const r = regions[0];
      setTimeout(() => {
        if (wavesurfer && isLooping.value) {
          wavesurfer.setTime(r.start);
          wavesurfer.play(r.start, r.end);
        }
      }, 10);
    } else {
      isLooping.value = false;
    }
  });

  wavesurfer.on('decode', () => {
    duration.value = wavesurfer.getDuration();
    wsRegions.clearRegions();
    const start = props.initialRegion?.start ?? 0;
    const end = props.initialRegion?.end ?? duration.value;
    regionStart.value = start;
    regionEnd.value = end;
    wsRegions.addRegion({ start, end, color: 'rgba(59, 130, 246, 0.15)', resize: true, drag: false, style: { border: '2px solid #3b82f6', borderTop: '2px solid #3b82f6', borderBottom: '2px solid #3b82f6' } });
  });

  if (props.audioUrl) {
    wavesurfer.load(props.audioUrl);
  }

  document.addEventListener('keydown', handleKeydown);
});

watch(() => props.audioUrl, (newUrl) => {
  if (wavesurfer && newUrl) {
    wavesurfer.load(newUrl);
  }
});

watch(() => props.initialRegion, (r) => {
  if (r && wsRegions) {
    const regions = wsRegions.getRegions();
    if (regions.length > 0) {
      regionUpdating = true;
      regions[0].setOptions({ start: r.start, end: r.end });
      regionStart.value = r.start;
      regionEnd.value = r.end;
      regionUpdating = false;
    }
  }
}, { deep: true });

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
  if (wavesurfer) {
    isLooping.value = false;
    wavesurfer.destroy();
  }
});

function handleKeydown(e) {
  if (e.code === 'Space' && !e.target.matches('input, textarea')) {
    e.preventDefault();
    playPause();
  }
}

function playPause() {
  if (wavesurfer) {
    isLooping.value = false;
    wavesurfer.playPause();
  }
}

function seekTo(ms) {
  if (wavesurfer) {
    const sec = ms / 1000;
    currentTime.value = sec;
    wavesurfer.setTime(sec);
  }
}

function zoomIn() {
  if (wavesurfer) {
    const cur = wavesurfer.options.minPxPerSec || 50;
    wavesurfer.zoom(cur * 1.5);
  }
}

function zoomOut() {
  if (wavesurfer) {
    const cur = wavesurfer.options.minPxPerSec || 50;
    wavesurfer.zoom(Math.max(10, cur / 1.5));
  }
}

function resetZoom() {
  if (wavesurfer) wavesurfer.zoom(50);
}

function resetRegion() {
  if (!wavesurfer) return;
  const dur = wavesurfer.getDuration();
  wsRegions?.clearRegions();
  wsRegions?.addRegion({ start: 0, end: dur, color: 'rgba(59, 130, 246, 0.15)', resize: true, drag: false, style: { border: '2px solid #3b82f6', borderTop: '2px solid #3b82f6', borderBottom: '2px solid #3b82f6' } });
  regionStart.value = 0;
  regionEnd.value = dur;
  emit('update:region', { start: 0, end: dur });
}

function playRegion() {
  if (!wavesurfer) return;
  const regions = wsRegions?.getRegions();
  if (regions?.length > 0) {
    const r = regions[0];
    isLooping.value = true;
    wavesurfer.setTime(r.start);
    wavesurfer.play(r.start, r.end);
  }
}

function onStartInput() {
  if (!wsRegions) return;
  const regions = wsRegions.getRegions();
  if (regions.length > 0) {
    const s = Math.max(0, Math.min(regionStart.value, regionEnd.value - 0.01));
    regionStart.value = s;
    regionUpdating = true;
    regions[0].setOptions({ start: s });
    regionUpdating = false;
    emit('update:region', { start: s, end: regionEnd.value });
  }
}

function onEndInput() {
  if (!wsRegions) return;
  const regions = wsRegions.getRegions();
  if (regions.length > 0) {
    const e = Math.min(duration.value, Math.max(regionEnd.value, regionStart.value + 0.01));
    regionEnd.value = e;
    regionUpdating = true;
    regions[0].setOptions({ end: e });
    regionUpdating = false;
    emit('update:region', { start: regionStart.value, end: e });
  }
}

function formatTime(seconds) {
  if (isNaN(seconds) || seconds < 0) return '0:00.000';
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  const ms = Math.floor((seconds % 1) * 1000);
  return `${m}:${String(s).padStart(2, '0')}.${String(ms).padStart(3, '0')}`;
}
</script>

<template>
  <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
    <!-- 工具栏 -->
    <div class="flex items-center justify-between px-4 py-2 bg-gray-50 border-b border-gray-200">
      <div class="flex items-center gap-2">
        <button @click="playPause" class="flex items-center gap-1 px-2.5 py-1 rounded-md bg-white border border-gray-200 hover:bg-gray-100 text-gray-700 text-xs font-medium transition-colors" title="空格键播放/暂停">
          <svg v-if="isPlaying" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor"><path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z"/></svg>
          <svg v-else class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
          {{ isPlaying ? '暂停' : '播放' }}
        </button>
        <button @click="playRegion" class="flex items-center gap-1 px-2.5 py-1 rounded-md bg-white border border-gray-200 hover:bg-gray-100 text-gray-700 text-xs font-medium transition-colors" title="循环播放选区">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
            <path d="M1 4v6h6M23 20v-6h-6"/>
            <path d="M20.49 9A9 9 0 005.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 013.51 15"/>
          </svg>
          选区循环
        </button>
      </div>

      <div class="flex items-center gap-2">
        <button @click="zoomOut" class="p-1 rounded hover:bg-gray-200 text-gray-500 transition-colors" title="缩小">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"/></svg>
        </button>
        <button @click="resetZoom" class="px-1.5 py-0.5 rounded text-xs text-gray-500 hover:bg-gray-200 transition-colors" title="重置缩放">自动</button>
        <button @click="zoomIn" class="p-1 rounded hover:bg-gray-200 text-gray-500 transition-colors" title="放大">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/></svg>
        </button>
      </div>
    </div>

    <!-- 播放进度条 -->
    <div v-if="duration > 0" class="px-4 pt-2 bg-white">
      <div class="flex items-center gap-3">
        <span class="text-xs font-mono text-gray-400 w-14 select-none">{{ formatTime(currentTime) }}</span>
        <input
          type="range"
          :min="0"
          :max="durationMs"
          :value="currentMs"
          @input="seekTo(parseInt($event.target.value))"
          @mousedown="isSeeking = true"
          @mouseup="isSeeking = false"
          class="flex-1 h-1.5 accent-blue-500 cursor-pointer"
        />
        <span class="text-xs font-mono text-gray-400 w-14 select-none text-right">{{ formatTime(duration) }}</span>
      </div>
    </div>

    <!-- 波形 -->
    <div class="rounded-lg overflow-hidden">
      <div ref="waveformRef" class="w-full cursor-text"></div>
    </div>

    <!-- 底部精确控制 -->
    <div class="px-4 py-2 bg-gray-50 border-t border-gray-200 flex items-center justify-between gap-4 flex-wrap">
      <button @click="resetRegion" class="flex items-center gap-1 px-2 py-1 rounded-md bg-white border border-gray-200 hover:bg-gray-100 text-gray-600 text-xs transition-colors">
        重置选区
      </button>
      <div class="flex items-center gap-2 text-xs text-gray-500">
        <span>起点</span>
        <input
          :value="regionStart.toFixed(3)"
          @input="regionStart = parseFloat($event.target.value) || 0; onStartInput()"
          type="number"
          min="0"
          :max="regionEnd - 0.01"
          step="0.001"
          class="w-22 bg-white border border-gray-200 rounded px-2 py-1 text-xs font-mono text-gray-700 outline-none focus:border-blue-400"
        />
        <span>终点</span>
        <input
          :value="regionEnd.toFixed(3)"
          @input="regionEnd = parseFloat($event.target.value) || 0; onEndInput()"
          type="number"
          :min="regionStart + 0.01"
          :max="duration"
          step="0.001"
          class="w-22 bg-white border border-gray-200 rounded px-2 py-1 text-xs font-mono text-gray-700 outline-none focus:border-blue-400"
        />
        <span class="text-gray-400 font-mono">({{ (regionEnd - regionStart).toFixed(3) }}s)</span>
      </div>
    </div>
  </div>
</template>