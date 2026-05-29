<!--
 Copyright Atemukesu
 SPDX-License-Identifier: GPL-3.0
-->

<script setup>
import { ref, watch, computed } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';

const props = defineProps({
  visible: Boolean,
});

const emit = defineEmits(['close', 'create']);

const name = ref('');
const description = ref('');
const packPath = ref('');
const nameError = ref('');

watch(() => props.visible, (newVal) => {
  if (newVal) {
    name.value = '';
    description.value = '';
    packPath.value = '';
    nameError.value = '';
  }
});

const isValidName = computed(() => {
  if (!name.value) return false;
  return /^[a-zA-Z\-_]+$/.test(name.value);
});

const handleNameInput = () => {
  if (name.value && !isValidName.value) {
    nameError.value = '名称只能包含字母 (a-z, A-Z)、连字符 (-) 和下划线 (_)';
  } else {
    nameError.value = '';
  }
};

const pickPath = async () => {
  try {
    const selected = await open({
      directory: true,
    });
    if (selected) {
      packPath.value = selected;
    }
  } catch (err) {
    console.error('选择目录失败:', err);
  }
};

const handleCreate = () => {
  if (!name.value || !packPath.value || !isValidName.value) return;
  emit('create', {
    name: name.value,
    description: description.value,
    path: packPath.value,
  });
};
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div class="bg-white rounded-xl p-6 w-[500px] shadow-xl">
        <div class="flex items-center justify-between mb-5">
          <div>
            <h2 class="text-lg font-semibold text-gray-800">创建音色包</h2>
            <p class="text-xs text-gray-500 mt-0.5">设置新的 Minecraft 音色包</p>
          </div>
          <button @click="emit('close')" class="text-gray-500 hover:text-gray-700 transition-colors p-1">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>

        <div class="space-y-4">
          <div>
            <label class="block text-xs text-gray-500 mb-1.5">音色包名称</label>
            <input
              v-model="name"
              @input="handleNameInput"
              type="text"
              placeholder="My-Soundpack"
              class="w-full bg-gray-50 border rounded-lg px-3 py-2 text-sm text-gray-700 placeholder-gray-400 outline-none focus:ring-1 transition-all"
              :class="nameError ? 'border-red-300 focus:border-red-400 focus:ring-red-400' : 'border-gray-200 focus:border-blue-400 focus:ring-blue-400'"
            />
            <p v-if="nameError" class="text-xs text-red-500 mt-1.5">{{ nameError }}</p>
            <p v-else class="text-xs text-gray-400 mt-1.5">只能使用字母、连字符 (-) 和下划线 (_)</p>
          </div>

          <div>
            <label class="block text-xs text-gray-500 mb-1.5">描述</label>
            <textarea
              v-model="description"
              rows="2"
              placeholder="一个自定义音色包..."
              class="w-full bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-700 placeholder-gray-400 outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 transition-all resize-none"
            ></textarea>
          </div>

          <div>
            <label class="block text-xs text-gray-500 mb-1.5">保存位置</label>
            <div class="flex gap-2">
              <input
                :value="packPath"
                readonly
                placeholder="点击右侧按钮选择文件夹..."
                class="flex-1 bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-700 placeholder-gray-400 outline-none"
              />
              <button @click="pickPath" class="px-4 py-2 rounded-lg bg-blue-500 hover:bg-blue-600 text-white text-xs font-medium transition-colors whitespace-nowrap">
                选择文件夹
              </button>
            </div>
          </div>

          <p class="text-xs text-gray-400">
            Minecraft 1.20.1 使用 pack format 15。
          </p>
        </div>

        <div class="flex justify-end gap-3 mt-6 pt-4 border-t border-gray-100">
          <button @click="emit('close')" class="px-4 py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-sm font-medium transition-colors">取消</button>
          <button
            @click="handleCreate"
            :disabled="!name || !packPath || !isValidName"
            class="px-4 py-2 rounded-lg bg-blue-500 hover:bg-blue-600 text-white text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            创建音色包
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>