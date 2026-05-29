<script setup>
import { ref, reactive, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import { INSTRUMENTS } from './constants/gm.js';

import RightSidebar from './components/RightSidebar.vue';
import SampleList from './components/SampleList.vue';
import EditorModal from './components/EditorModal.vue';
import Sf2GeneratorDialog from './components/Sf2GeneratorDialog.vue';
import SettingsModal from './components/SettingsModal.vue';
import PackCreateDialog from './components/PackCreateDialog.vue';
import Toast from './components/Toast.vue';

const toastRef = ref(null);
const showToast = (message, type = 'error') => {
  toastRef.value?.addToast(message, type);
};

const settings = reactive({
  fluidsynthPath: '',
  ffmpegPath: '',
  fluidsynthAvailable: false,
  ffmpegAvailable: false,
});

const isFluidsynthReady = computed(() => settings.fluidsynthAvailable || !!settings.fluidsynthPath?.trim());
const isFfmpegReady = computed(() => settings.ffmpegAvailable || !!settings.ffmpegPath?.trim());

const currentPack = ref(null);
const selectedInstrumentId = ref(0);
const allSamples = reactive({});

const playingSample = ref(null);
const loadingPlayback = ref(false);
let currentAudio = null;

const currentSamples = computed(() => {
  return allSamples[selectedInstrumentId.value] || [];
});

const showSf2Dialog = ref(false);
const showEditorModal = ref(false);
const showSettings = ref(false);
const showPackCreate = ref(false);
const sf2DialogRef = ref(null);
const packCreateRef = ref(null);

const showPitchInput = ref(false);
const pitchInputValue = ref('');
const pitchInputResolve = ref(null);

const requestPitch = (filename, defaultPitch = 60) => {
  return new Promise((resolve) => {
    pitchInputValue.value = String(defaultPitch);
    showPitchInput.value = { filename, defaultPitch };
    pitchInputResolve.value = resolve;
  });
};

const confirmPitchInput = () => {
  const val = parseInt(pitchInputValue.value);
  showPitchInput.value = false;
  if (pitchInputResolve.value) {
    pitchInputResolve.value(isNaN(val) ? 60 : val);
    pitchInputResolve.value = null;
  }
};

const cancelPitchInput = () => {
  showPitchInput.value = false;
  if (pitchInputResolve.value) {
    pitchInputResolve.value(null);
    pitchInputResolve.value = null;
  }
};

const editorAudioUrl = ref('');
const editorSampleInfo = ref(null);
const editorRegion = ref(null);

const progress = ref(-1);
const isGenerating = ref(false);
const activeTasks = ref([]);
const showTaskModal = ref(false);

onMounted(async () => {
  await listen('generation-progress', (event) => {
    progress.value = event.payload;
    const currentTask = activeTasks.value.find(t => t.status === 'generating');
    if (currentTask) currentTask.progress = event.payload;
    if (progress.value >= 100) {
      setTimeout(() => { progress.value = -1; }, 800);
    }
  });

  settings.fluidsynthAvailable = await invoke('check_command_available', { cmd: 'fluidsynth' });
  settings.ffmpegAvailable = await invoke('check_command_available', { cmd: 'ffmpeg' });

  try {
    const s = await invoke('get_settings');
    settings.fluidsynthPath = s.fluidsynth_path || '';
    settings.ffmpegPath = s.ffmpeg_path || '';
  } catch {
    // ignore
  }
});

const selectInstrument = (id) => {
  selectedInstrumentId.value = id;
};

const openSf2Dialog = () => {
  showSf2Dialog.value = true;
};

const handleExport = async () => {
  if (!currentPack.value) return;
  try {
    const outputPath = await save({
      defaultPath: `${currentPack.value.name}.zip`,
      filters: [{ name: 'ZIP 文件', extensions: ['zip'] }],
    });
    if (!outputPath) return;
    showToast('正在导出...', 'success');
    await invoke('export_to_zip', {
      appPath: currentPack.value.path,
      outputPath,
    });
    showToast('导出成功', 'success');
  } catch (err) {
    showToast('导出失败: ' + err);
  }
};

const handleSf2Generate = async (params) => {
  showToast('正在生成音频，生成过程可能大量占用系统资源导致计算机卡顿，请耐心等待', 'warning');
  isGenerating.value = true;
  activeTasks.value = params.instruments.map((id) => ({
    id,
    name: INSTRUMENTS.find(i => i.id === id)?.name || `Instrument ${id}`,
    progress: 0,
    status: 'pending',
  }));
  showTaskModal.value = true;

  try {
    const totalInstruments = params.instruments.length;
    let completed = 0;

    for (const instId of params.instruments) {
      const task = activeTasks.value.find(t => t.id === instId);
      if (task) task.status = 'generating';

      await invoke('generate_instrument', {
        fluidsynthPath: settings.fluidsynthPath,
        sf2Path: params.sf2Path,
        bank: instId === 128 ? 128 : 0,
        preset: instId === 128 ? 0 : instId,
        instrumentId: String(instId),
        noteStart: params.noteStart,
        noteEnd: params.noteEnd,
        noteStep: params.noteStep,
        outPath: currentPack.value.path,
        gain: params.gain,
        region: params.region,
        maxCores: params.maxCores,
      });

      if (task) { task.status = 'done'; task.progress = 100; }

      const samples = [];
      for (let n = params.noteStart; n <= params.noteEnd; n += params.noteStep) {
        samples.push({
          note: n,
          file: `${instId}.${n}.ogg`,
          source: 'sf2',
        });
      }
      allSamples[instId] = samples;
      completed++;
    }

    showSf2Dialog.value = false;
    showToast(`已为 ${totalInstruments} 个乐器生成 ${params.instruments.reduce((sum, instId) => sum + allSamples[instId]?.length || 0, 0)} 个样本`, 'success');
  } catch (err) {
    showToast('生成失败: ' + err);
    const currentTask = activeTasks.value.find(t => t.status === 'generating');
    if (currentTask) currentTask.status = 'error';
  } finally {
    isGenerating.value = false;
  }
};

const addCustomAudio = async () => {
  const selected = await open({
    multiple: true,
    filters: [{ name: '音频文件', extensions: ['wav', 'mp3', 'ogg', 'flac', 'aiff'] }],
  });
  if (!selected) return;

  const paths = Array.isArray(selected) ? selected : [selected];
  const instId = selectedInstrumentId.value;

  if (!allSamples[instId]) allSamples[instId] = [];

  showToast(`正在处理 ${paths.length} 个音频文件，请稍候...`, 'success');

  for (const p of paths) {
    let pitch = 60;
    const existing = new Set((allSamples[instId] || []).map(s => s.note));
    while (existing.has(pitch) && pitch < 127) pitch++;

    const promptPitch = await requestPitch(p.split(/[\\/]/).pop(), pitch);
    if (promptPitch === null) continue;
    pitch = promptPitch;

    try {
      await invoke('convert_custom_audio', {
        inputPath: p,
        outPath: currentPack.value.path,
        instrumentId: String(instId),
        pitch: pitch,
        gain: 0.0,
        region: null,
      });

      allSamples[instId].push({
        note: pitch,
        file: `${instId}.${pitch}.ogg`,
        source: 'custom',
      });
    } catch (err) {
      showToast('转码失败: ' + err);
    }
  }
};

const handlePlay = async (sample) => {
  if (currentAudio && playingSample.value?.note === sample.note) {
    currentAudio.pause();
    currentAudio.onended = null;
    currentAudio = null;
    playingSample.value = null;
    loadingPlayback.value = false;
    return;
  }

  if (currentAudio) {
    currentAudio.pause();
    currentAudio.onended = null;
    currentAudio = null;
  }

  loadingPlayback.value = true;
  playingSample.value = sample;
  try {
    const url = await invoke('preview_existing_sample', {
      appPath: currentPack.value.path,
      instrumentId: String(selectedInstrumentId.value),
      pitch: sample.note,
    });
    const audio = new Audio(url);
    currentAudio = audio;
    audio.play();
    audio.onended = () => {
      currentAudio = null;
      playingSample.value = null;
      loadingPlayback.value = false;
    };
  } catch (err) {
    console.error('播放失败', err);
    currentAudio = null;
    playingSample.value = null;
    loadingPlayback.value = false;
  }
};

const handleEdit = async (sample) => {
  editorSampleInfo.value = sample;
  editorRegion.value = null;
  try {
    const url = await invoke('preview_existing_sample', {
      appPath: currentPack.value.path,
      instrumentId: String(selectedInstrumentId.value),
      pitch: sample.note,
    });
    editorAudioUrl.value = url;
  } catch {
    editorAudioUrl.value = '';
  }
  showEditorModal.value = true;
};

const handleEditorSave = async (editData) => {
  if (!editorSampleInfo.value) {
    showEditorModal.value = false;
    return;
  }
  try {
    await invoke('reprocess_sample', {
      appPath: currentPack.value.path,
      instrumentId: String(selectedInstrumentId.value),
      pitch: editorSampleInfo.value.note,
      gain: editData.gain,
      region: editData.region,
    });
  } catch (err) {
    showToast('重新处理失败: ' + err);
  }
  showEditorModal.value = false;
};

const handleBatchEdit = async ({ notes, gain, region }) => {
  const total = notes.length;
  let completed = 0;
  showToast(`正在批量处理 ${total} 个样本...`, 'success');
  for (const pitch of notes) {
    try {
      await invoke('reprocess_sample', {
        appPath: currentPack.value.path,
        instrumentId: String(selectedInstrumentId.value),
        pitch,
        gain,
        region: region || null,
      });
    } catch (err) {
      showToast(`音符 ${pitch} 处理失败: ${err}`);
    }
    completed++;
  }
  showToast(`已更新 ${completed} 个样本`, 'success');
};

const handleBatchDelete = async (notes) => {
  let removed = 0;
  for (const pitch of notes) {
    try {
      await invoke('delete_sample', {
        appPath: currentPack.value.path,
        instrumentId: String(selectedInstrumentId.value),
        pitch,
      });
      const samples = allSamples[selectedInstrumentId.value];
      if (samples) {
        const idx = samples.findIndex(s => s.note === pitch);
        if (idx !== -1) samples.splice(idx, 1);
      }
      removed++;
    } catch (err) {
      showToast(`删除音符 ${pitch} 失败: ${err}`);
    }
  }
  showToast(`已删除 ${removed} 个样本`, 'success');
};

const handleDelete = async (index) => {
  const instId = selectedInstrumentId.value;
  const samples = allSamples[instId];
  if (!samples) return;
  const sample = samples[index];
  if (!sample) return;
  try {
    await invoke('delete_sample', {
      appPath: currentPack.value.path,
      instrumentId: String(instId),
      pitch: sample.note,
    });
  } catch (err) {
    console.error('删除文件失败:', err);
  }
  samples.splice(index, 1);
};

const handleSaveSettings = (s) => {
  settings.fluidsynthPath = s.fluidsynthPath;
  settings.ffmpegPath = s.ffmpegPath;
  settings.fluidsynthAvailable = s.fluidsynthAvailable;
  settings.ffmpegAvailable = s.ffmpegAvailable;
  showSettings.value = false;
};

const handlePackCreate = async (packInfo) => {
  try {
    const createdPath = await invoke('create_soundpack', {
      parentPath: packInfo.path,
      name: packInfo.name,
      description: packInfo.description,
      packFormat: 15,
    });

    currentPack.value = {
      name: packInfo.name,
      description: packInfo.description,
      path: createdPath,
    };

    showPackCreate.value = false;
  } catch (err) {
    showToast('音色包创建失败: ' + err);
  }
};

const handleOpenPack = async () => {
  const selected = await open({
    directory: true,
  });
  if (!selected) return;

  try {
    await invoke('validate_soundpack', { path: selected });
  } catch (err) {
    showToast('无效的音色包: ' + err);
    return;
  }

  try {
    const info = await invoke('refresh_pack', { path: selected });
    currentPack.value = {
      name: info.displayName,
      description: '',
      path: selected,
    };

    allSamples[selectedInstrumentId.value] = [];
    const instruments = info.availableInstruments || {};
    for (const [instId, notes] of Object.entries(instruments)) {
      const samples = notes.map(n => ({
        note: n,
        file: `${instId}.${n}.ogg`,
        source: 'existing',
      }));
      allSamples[parseInt(instId)] = samples;
    }
  } catch (err) {
    showToast('打开失败: ' + err);
  }
};

const closePack = () => {
  currentPack.value = null;
  selectedInstrumentId.value = 0;
  Object.keys(allSamples).forEach(key => delete allSamples[key]);
};

const handleRefresh = async () => {
  if (!currentPack.value) return;
  try {
    const info = await invoke('refresh_pack', { path: currentPack.value.path });
    currentPack.value.name = info.displayName;

    Object.keys(allSamples).forEach(key => delete allSamples[key]);
    const instruments = info.availableInstruments || {};
    for (const [instId, notes] of Object.entries(instruments)) {
      const samples = notes.map(n => ({
        note: n,
        file: `${instId}.${n}.ogg`,
        source: 'existing',
      }));
      allSamples[parseInt(instId)] = samples;
    }

    showToast('已刷新', 'success');
  } catch (err) {
    showToast('刷新失败: ' + err);
  }
};
</script>

<template>
  <div class="h-screen w-screen flex overflow-hidden bg-gray-50">

    <!-- ==================== 初始界面：未打开音色包 ==================== -->
    <div v-if="!currentPack" class="flex-1 flex flex-col items-center justify-center">
      <div class="text-center">
        <div class="w-24 h-24 mx-auto mb-8 rounded-3xl bg-white shadow-lg flex items-center justify-center">
          <svg class="w-12 h-12 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
              d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
          </svg>
        </div>
        <h1 class="text-2xl font-bold text-gray-800 mb-2">音色包生成器</h1>
        <p class="text-gray-500 text-sm mb-10">从 SF2 音源创建 Minecraft 音色包</p>

        <div class="flex justify-center gap-4">
          <div class="relative group">
            <button @click="showPackCreate = true" :disabled="!isFfmpegReady"
              class="flex items-center gap-3 px-6 py-4 rounded-xl font-medium transition-colors shadow-lg"
              :class="isFfmpegReady ? 'bg-blue-500 hover:bg-blue-600 text-white shadow-blue-500/20' : 'bg-gray-300 text-gray-500 cursor-not-allowed shadow-none'">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              创建新音色包
            </button>
            <div v-if="!isFfmpegReady"
              class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1.5 bg-gray-800 text-white text-xs rounded-lg whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
              请先在设置中配置 FFmpeg 路径
            </div>
          </div>

          <div class="relative group">
            <button @click="handleOpenPack" :disabled="!isFfmpegReady"
              class="flex items-center gap-3 px-6 py-4 rounded-xl font-medium transition-colors shadow-md border"
              :class="isFfmpegReady ? 'bg-white hover:bg-gray-100 text-gray-700 border-gray-200' : 'bg-gray-200 text-gray-400 cursor-not-allowed border-gray-200 shadow-none'">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
              打开已有音色包
            </button>
            <div v-if="!isFfmpegReady"
              class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1.5 bg-gray-800 text-white text-xs rounded-lg whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
              请先在设置中配置 FFmpeg 路径
            </div>
          </div>
        </div>

        <div class="mt-8 flex justify-center gap-4 text-xs text-gray-400">
          <button @click="showSettings = true" class="hover:text-gray-600 transition-colors">设置</button>
        </div>
      </div>
    </div>

    <!-- ==================== 音色包编辑器：已打开音色包 ==================== -->
    <template v-else>
      <!-- 左侧：乐器侧边栏 -->
      <RightSidebar :selectedId="selectedInstrumentId" @select="selectInstrument" />

      <!-- 右侧：主区域 -->
      <div class="flex-1 flex flex-col min-w-0">

        <!-- 顶栏 -->
        <header
          class="flex items-center justify-between px-6 py-3 border-b border-gray-200 bg-white flex-shrink-0 shadow-sm">
          <div class="flex items-center gap-4">
            <button @click="closePack" class="text-gray-400 hover:text-gray-600 transition-colors">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <div>
              <h1 class="text-sm font-semibold text-gray-800">{{ currentPack.name }}</h1>
              <p class="text-xs text-gray-400">{{ currentPack.path }}</p>
            </div>

            <button @click="handleRefresh"
              class="text-gray-400 hover:text-gray-600 transition-colors p-1.5 rounded-lg hover:bg-gray-100" title="刷新">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </button>

            <div v-if="progress >= 0" class="flex items-center gap-2">
              <div class="w-24 h-1.5 bg-gray-200 rounded-full overflow-hidden">
                <div class="h-full rounded-full transition-all duration-300 bg-blue-500"
                  :style="{ width: progress + '%' }"></div>
              </div>
              <span class="text-xs font-mono text-gray-500">{{ Math.round(progress) }}%</span>
            </div>
          </div>

          <div class="flex items-center gap-2">

            <div class="relative group">
              <button @click="openSf2Dialog" :disabled="!isFluidsynthReady"
                class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors shadow-sm"
                :class="isFluidsynthReady ? 'bg-blue-500 hover:bg-blue-600 text-white' : 'bg-gray-300 text-gray-500 cursor-not-allowed'">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                </svg>
                从 SF2 生成
              </button>
              <div v-if="!isFluidsynthReady"
                class="absolute top-full left-1/2 -translate-x-1/2 mt-2 px-3 py-1.5 bg-gray-800 text-white text-xs rounded-lg whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                请先在设置中配置 FluidSynth 路径
              </div>
            </div>

            <button @click="handleExport"
              class="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-sm font-medium transition-colors">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2M12 17V7m0 0 4 4m-4-4-4 4" />
              </svg>
              导出为 zip 包
            </button>

            <button @click="showSettings = true"
              class="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-sm font-medium transition-colors">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
              设置
            </button>
          </div>
        </header>

        <!-- 样本列表 -->
        <SampleList :instrumentId="selectedInstrumentId" :samples="currentSamples" :playingNote="playingSample?.note"
          @play="handlePlay" @edit="handleEdit" @delete="handleDelete" @add-custom="addCustomAudio"
          @batch-edit="handleBatchEdit" @batch-delete="handleBatchDelete" />
      </div>
    </template>

    <!-- ==================== 对话框 ==================== -->
    <PackCreateDialog ref="packCreateRef" :visible="showPackCreate" @close="showPackCreate = false"
      @create="handlePackCreate" />

    <Sf2GeneratorDialog ref="sf2DialogRef" :visible="showSf2Dialog" :instrumentId="selectedInstrumentId"
      @close="showSf2Dialog = false" @generate="handleSf2Generate" />

    <EditorModal :visible="showEditorModal" :audioUrl="editorAudioUrl" :sampleInfo="editorSampleInfo"
      :initialRegion="editorRegion" @close="showEditorModal = false" @save="handleEditorSave"
      @update:region="(r) => editorRegion = r" />

    <SettingsModal :visible="showSettings" @close="showSettings = false" @save="handleSaveSettings"
      @error="(msg) => showToast(msg)" />

    <Toast ref="toastRef" />

    <!-- 后台任务指示器 -->
    <Teleport to="body">
      <button v-if="isGenerating" @click="showTaskModal = true"
        class="fixed bottom-6 right-6 z-50 flex items-center gap-2 px-4 py-2.5 rounded-full shadow-lg bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium transition-colors">
        <svg class="w-4 h-4 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        {{activeTasks.filter(t => t.status === 'done').length}}/{{ activeTasks.length }} 个任务进行中
      </button>
    </Teleport>

    <!-- 任务详情弹窗 -->
    <div v-if="showTaskModal" class="fixed inset-0 z-[70] flex items-center justify-center bg-black/50"
      @click.self="showTaskModal = false">
      <div class="bg-white rounded-xl p-5 w-[480px] shadow-xl max-h-[70vh] flex flex-col">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-base font-semibold text-gray-800">生成任务</h2>
          <button @click="showTaskModal = false" class="text-gray-400 hover:text-gray-600 p-1">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="flex-1 overflow-y-auto space-y-2 -mx-1 px-1">
          <div v-for="task in activeTasks" :key="task.id" class="flex items-center gap-3 px-3 py-2.5 rounded-lg"
            :class="task.status === 'generating' ? 'bg-blue-50' : task.status === 'done' ? 'bg-green-50' : task.status === 'error' ? 'bg-red-50' : 'bg-gray-50'">
            <svg v-if="task.status === 'done'" class="w-5 h-5 text-green-500 flex-shrink-0" fill="none"
              stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <svg v-else-if="task.status === 'error'" class="w-5 h-5 text-red-500 flex-shrink-0" fill="none"
              stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <svg v-else-if="task.status === 'generating'" class="w-5 h-5 text-blue-500 flex-shrink-0 animate-spin"
              fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <svg v-else class="w-5 h-5 text-gray-300 flex-shrink-0" fill="none" stroke="currentColor"
              viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <span class="text-sm font-medium truncate"
                  :class="task.status === 'generating' ? 'text-blue-700' : task.status === 'done' ? 'text-green-700' : task.status === 'error' ? 'text-red-700' : 'text-gray-500'">
                  {{ task.id }} - {{ task.name }}
                </span>
                <span v-if="task.status === 'generating'" class="text-xs font-mono text-blue-600 ml-2">{{
                  Math.round(task.progress) }}%</span>
                <span v-else-if="task.status === 'done'" class="text-xs text-green-600 ml-2">完成</span>
                <span v-else-if="task.status === 'error'" class="text-xs text-red-600 ml-2">失败</span>
                <span v-else class="text-xs text-gray-400 ml-2">等待中</span>
              </div>
              <div v-if="task.status === 'generating'"
                class="mt-1.5 w-full h-1.5 bg-blue-200 rounded-full overflow-hidden">
                <div class="h-full rounded-full bg-blue-500 transition-all duration-300"
                  :style="{ width: task.progress + '%' }"></div>
              </div>
            </div>
          </div>
        </div>
        <button @click="showTaskModal = false"
          class="mt-4 w-full py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-sm font-medium transition-colors">
          关闭
        </button>
      </div>
    </div>

    <Teleport to="body">
      <div v-if="showPitchInput" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/50"
        @click.self="cancelPitchInput">
        <div class="bg-white rounded-xl p-6 w-[400px] shadow-xl">
          <div class="flex items-center justify-between mb-5">
            <div>
              <h2 class="text-lg font-semibold text-gray-800">输入 MIDI 音高</h2>
              <p class="text-xs text-gray-500 mt-0.5">文件: {{ showPitchInput.filename }}</p>
            </div>
            <button @click="cancelPitchInput" class="text-gray-500 hover:text-gray-700 transition-colors p-1">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div class="mb-5">
            <label class="block text-xs text-gray-500 mb-1.5">MIDI 音高 (0-127)</label>
            <input v-model="pitchInputValue" type="number" min="0" max="127"
              class="w-full bg-white border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-700 outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 transition-all"
              @keydown.enter="confirmPitchInput" />
          </div>

          <div class="flex justify-end gap-3">
            <button @click="cancelPitchInput"
              class="px-4 py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-sm font-medium transition-colors">取消</button>
            <button @click="confirmPitchInput"
              class="px-4 py-2 rounded-lg bg-blue-500 hover:bg-blue-600 text-white text-sm font-medium transition-colors">确认</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>