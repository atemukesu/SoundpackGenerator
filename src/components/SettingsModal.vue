<script setup>
import { ref, watch, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

const props = defineProps({
  visible: Boolean,
});

const emit = defineEmits(['close', 'save', 'error']);

const fluidsynthPath = ref('');
const ffmpegPath = ref('');
const fluidsynthAvailable = ref(false);
const ffmpegAvailable = ref(false);
const checking = ref(true);
const savedSettings = ref(null);

const fluidsynthPathValid = ref(true);
const ffmpegPathValid = ref(true);

let fluidsynthCheckTimeout = null;
let ffmpegCheckTimeout = null;

const fluidsynthReady = computed(() => fluidsynthAvailable.value || (fluidsynthPathValid.value && !!fluidsynthPath.value.trim()));
const ffmpegReady = computed(() => ffmpegAvailable.value || (ffmpegPathValid.value && !!ffmpegPath.value.trim()));

const fluidsynthPathEmpty = computed(() => !fluidsynthAvailable.value && !fluidsynthPath.value.trim());
const ffmpegPathEmpty = computed(() => !ffmpegAvailable.value && !ffmpegPath.value.trim());

const fluidsynthPathInvalid = computed(() => !fluidsynthAvailable.value && fluidsynthPath.value.trim() && !fluidsynthPathValid.value);
const ffmpegPathInvalid = computed(() => !ffmpegAvailable.value && ffmpegPath.value.trim() && !ffmpegPathValid.value);

const loadSettings = async () => {
  try {
    const s = await invoke('get_settings');
    savedSettings.value = s;
    fluidsynthPath.value = s.fluidsynth_path || '';
    ffmpegPath.value = s.ffmpeg_path || '';
  } catch {
    savedSettings.value = { fluidsynth_path: '', ffmpeg_path: '' };
  }
};

onMounted(async () => {
  try {
    fluidsynthAvailable.value = await invoke('check_command_available', { cmd: 'fluidsynth' });
    ffmpegAvailable.value = await invoke('check_command_available', { cmd: 'ffmpeg' });
  } finally {
    checking.value = false;
  }
});

watch(() => props.visible, async (v) => {
  if (v) {
    await loadSettings();
  }
});

watch(fluidsynthPath, (newPath) => {
  if (!newPath.trim() || fluidsynthAvailable.value) {
    fluidsynthPathValid.value = true;
    return;
  }
  clearTimeout(fluidsynthCheckTimeout);
  fluidsynthCheckTimeout = setTimeout(async () => {
    try {
      const exists = await invoke('check_file_executable', { path: newPath });
      fluidsynthPathValid.value = exists;
    } catch {
      fluidsynthPathValid.value = false;
    }
  }, 500);
});

watch(ffmpegPath, (newPath) => {
  if (!newPath.trim() || ffmpegAvailable.value) {
    ffmpegPathValid.value = true;
    return;
  }
  clearTimeout(ffmpegCheckTimeout);
  ffmpegCheckTimeout = setTimeout(async () => {
    try {
      const exists = await invoke('check_file_executable', { path: newPath });
      ffmpegPathValid.value = exists;
    } catch {
      ffmpegPathValid.value = false;
    }
  }, 500);
});

const selectFluidsynth = async () => {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'FluidSynth', extensions: ['exe'] }],
  });
  if (selected) {
    const filename = selected.split(/[\\/]/).pop();
    if (filename === 'fluidsynth.exe') {
      fluidsynthPath.value = selected;
    } else {
      emit('error', '请选择名为 fluidsynth.exe 的文件');
    }
  }
};

const selectFfmpeg = async () => {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'FFmpeg', extensions: ['exe'] }],
  });
  if (selected) {
    const filename = selected.split(/[\\/]/).pop();
    if (filename === 'ffmpeg.exe') {
      ffmpegPath.value = selected;
    } else {
      emit('error', '请选择名为 ffmpeg.exe 的文件');
    }
  }
};

const handleSave = async () => {
  const fPath = fluidsynthPath.value.trim();
  const fmPath = ffmpegPath.value.trim();

  if (!fluidsynthAvailable.value && !fPath) {
    emit('error', '未检测到 FluidSynth 环境变量，请配置自定义路径');
    return;
  }
  if (!ffmpegAvailable.value && !fmPath) {
    emit('error', '未检测到 FFmpeg 环境变量，请配置自定义路径');
    return;
  }

  if (!fluidsynthAvailable.value && fPath) {
    const valid = await invoke('check_file_executable', { path: fPath });
    if (!valid) {
      emit('error', 'FluidSynth 自定义路径无效或文件不可执行');
      return;
    }
  }

  if (!ffmpegAvailable.value && fmPath) {
    const valid = await invoke('check_file_executable', { path: fmPath });
    if (!valid) {
      emit('error', 'FFmpeg 自定义路径无效或文件不可执行');
      return;
    }
  }

  try {
    await invoke('set_settings', {
      s: {
        fluidsynth_path: fPath,
        ffmpeg_path: fmPath,
      },
    });

    emit('save', {
      fluidsynthPath: fPath,
      ffmpegPath: fmPath,
      fluidsynthAvailable: fluidsynthAvailable.value,
      ffmpegAvailable: ffmpegAvailable.value,
    });
  } catch (err) {
    emit('error', '保存失败: ' + err);
  }
};
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div class="bg-white rounded-xl p-6 w-[500px] shadow-xl">
        <div class="flex items-center justify-between mb-5">
          <div>
            <h2 class="text-lg font-semibold text-gray-800">设置</h2>
            <p class="text-xs text-gray-500 mt-0.5">配置依赖工具路径</p>
          </div>
          <button @click="emit('close')" class="text-gray-500 hover:text-gray-700 transition-colors p-1">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>

        <div v-if="checking" class="flex items-center justify-center py-8">
          <div class="text-sm text-gray-500">检测中...</div>
        </div>

        <div v-else class="space-y-5">
          <div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
            <div class="flex items-center justify-between mb-3">
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full" :class="fluidsynthReady ? 'bg-green-500' : 'bg-red-500'"></div>
                <span class="text-sm font-medium text-gray-700">FluidSynth</span>
              </div>
              <span class="text-xs px-2 py-0.5 rounded-full" :class="fluidsynthReady ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'">
                {{ fluidsynthReady ? (fluidsynthAvailable ? '已检测到' : '已设置') : '未设置' }}
              </span>
            </div>

            <p v-if="fluidsynthReady" class="text-xs text-green-600 mb-3">
              {{ fluidsynthAvailable ? '已在环境变量中检测到，无需额外配置。' : '已配置自定义路径。' }}
            </p>

            <div>
              <label class="block text-xs text-gray-500 mb-1.5">自定义路径</label>
              <div class="flex gap-2">
                <input
                  v-model="fluidsynthPath"
                  type="text"
                  :placeholder="fluidsynthAvailable ? '已检测到系统命令，无需填写' : '未检测到环境变量，请选择路径'"
                  :disabled="fluidsynthAvailable"
                  class="flex-1 bg-white border rounded-lg px-3 py-2 text-sm text-gray-700 placeholder-gray-400 outline-none transition-all"
                  :class="{
                    'border-red-300 focus:border-red-400 focus:ring-1 focus:ring-red-400': fluidsynthPathEmpty || fluidsynthPathInvalid,
                    'border-gray-200 focus:border-blue-400 focus:ring-1 focus:ring-blue-400': !fluidsynthPathEmpty && !fluidsynthPathInvalid,
                    'opacity-50 cursor-not-allowed': fluidsynthAvailable,
                  }"
                />
                <button @click="selectFluidsynth" :disabled="fluidsynthAvailable" class="px-3 py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-xs font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed">浏览</button>
              </div>
              <p v-if="!fluidsynthPathValid && !fluidsynthAvailable && fluidsynthPath.trim()" class="text-xs text-red-500 mt-1">
                路径无效或文件不可执行
              </p>
            </div>
          </div>

          <div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
            <div class="flex items-center justify-between mb-3">
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full" :class="ffmpegReady ? 'bg-green-500' : 'bg-red-500'"></div>
                <span class="text-sm font-medium text-gray-700">FFmpeg</span>
              </div>
              <span class="text-xs px-2 py-0.5 rounded-full" :class="ffmpegReady ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'">
                {{ ffmpegReady ? (ffmpegAvailable ? '已检测到' : '已设置') : '未设置' }}
              </span>
            </div>

            <p v-if="ffmpegReady" class="text-xs text-green-600 mb-3">
              {{ ffmpegAvailable ? '已在环境变量中检测到，无需额外配置。' : '已配置自定义路径。' }}
            </p>

            <div>
              <label class="block text-xs text-gray-500 mb-1.5">自定义路径</label>
              <div class="flex gap-2">
                <input
                  v-model="ffmpegPath"
                  type="text"
                  :placeholder="ffmpegAvailable ? '已检测到系统命令，无需填写' : '未检测到环境变量，请选择路径'"
                  :disabled="ffmpegAvailable"
                  class="flex-1 bg-white border rounded-lg px-3 py-2 text-sm text-gray-700 placeholder-gray-400 outline-none transition-all"
                  :class="{
                    'border-red-300 focus:border-red-400 focus:ring-1 focus:ring-red-400': ffmpegPathEmpty || ffmpegPathInvalid,
                    'border-gray-200 focus:border-blue-400 focus:ring-1 focus:ring-blue-400': !ffmpegPathEmpty && !ffmpegPathInvalid,
                    'opacity-50 cursor-not-allowed': ffmpegAvailable,
                  }"
                />
                <button @click="selectFfmpeg" :disabled="ffmpegAvailable" class="px-3 py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-xs font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed">浏览</button>
              </div>
              <p v-if="!ffmpegPathValid && !ffmpegAvailable && ffmpegPath.trim()" class="text-xs text-red-500 mt-1">
                路径无效或文件不可执行
              </p>
            </div>
          </div>
        </div>

        <div class="flex justify-end gap-3 mt-6 pt-4 border-t border-gray-100">
          <button @click="emit('close')" class="px-4 py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-sm font-medium transition-colors">取消</button>
          <button
            @click="handleSave"
            class="px-4 py-2 rounded-lg bg-blue-500 hover:bg-blue-600 text-white text-sm font-medium transition-colors"
          >
            保存设置
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
