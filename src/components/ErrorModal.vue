<script setup>
const props = defineProps({
  visible: Boolean,
  title: { type: String, default: '错误' },
  details: { type: String, default: '' },
});

const emit = defineEmits(['close']);

const copyError = async () => {
  try {
    await navigator.clipboard.writeText(props.details);
  } catch {
    const ta = document.createElement('textarea');
    ta.value = props.details;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    document.execCommand('copy');
    document.body.removeChild(ta);
  }
  window.$showToast?.('已复制到剪贴板', 'success');
};
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div class="bg-white rounded-xl p-6 w-[620px] shadow-xl max-h-[80vh] flex flex-col">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-2">
            <div class="w-8 h-8 rounded-lg bg-red-100 flex items-center justify-center flex-shrink-0">
              <svg class="w-4 h-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01M10.29 3.86l-8.6 14.86A2 2 0 003.4 21h17.2a2 2 0 001.71-3.14l-8.6-14.86a2 2 0 00-3.42 0z"/>
              </svg>
            </div>
            <div>
              <h2 class="text-base font-semibold text-gray-800">{{ title }}</h2>
              <p class="text-xs text-gray-500 mt-0.5">命令执行出错，详细信息如下</p>
            </div>
          </div>
          <button @click="emit('close')" class="text-gray-500 hover:text-gray-700 transition-colors p-1">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>

        <div class="flex-1 overflow-y-auto min-h-0">
          <pre class="bg-gray-900 text-gray-100 rounded-lg p-4 text-xs font-mono leading-relaxed whitespace-pre-wrap break-all overflow-x-auto max-h-[50vh]">{{ details }}</pre>
        </div>

        <div class="flex justify-end gap-3 mt-5 pt-4 border-t border-gray-100">
          <button @click="copyError" class="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 text-sm font-medium transition-colors">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
            </svg>
            复制错误详情
          </button>
          <button @click="emit('close')" class="px-4 py-2 rounded-lg bg-red-500 hover:bg-red-600 text-white text-sm font-medium transition-colors">关闭</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
