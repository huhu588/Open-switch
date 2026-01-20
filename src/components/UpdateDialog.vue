<script setup lang="ts">
import { ref } from 'vue';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

// 对话框状态
const isOpen = ref(false);
const isChecking = ref(false);
const isDownloading = ref(false);
const downloadProgress = ref(0);
const updateInfo = ref<Update | null>(null);
const error = ref<string>('');

// 检查更新
async function checkForUpdate() {
  isChecking.value = true;
  error.value = '';
  
  try {
    const update = await check();
    if (update) {
      updateInfo.value = update;
      isOpen.value = true;
    } else {
      // 没有可用更新
      error.value = '当前已是最新版本';
    }
  } catch (e) {
    error.value = `检查更新失败: ${e}`;
    console.error('检查更新失败:', e);
  } finally {
    isChecking.value = false;
  }
}

// 下载并安装更新
async function downloadAndInstall() {
  if (!updateInfo.value) return;
  
  isDownloading.value = true;
  downloadProgress.value = 0;
  
  try {
    let downloaded = 0;
    let contentLength = 0;
    
    await updateInfo.value.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength || 0;
          console.log(`开始下载: ${contentLength} 字节`);
          break;
        case 'Progress':
          downloaded += event.data.chunkLength;
          if (contentLength > 0) {
            downloadProgress.value = Math.round((downloaded / contentLength) * 100);
          }
          break;
        case 'Finished':
          downloadProgress.value = 100;
          console.log('下载完成');
          break;
      }
    });
    
    // 更新安装完成，重启应用
    await relaunch();
  } catch (e) {
    error.value = `安装更新失败: ${e}`;
    console.error('安装更新失败:', e);
    isDownloading.value = false;
  }
}

// 关闭对话框
function closeDialog() {
  if (!isDownloading.value) {
    isOpen.value = false;
    updateInfo.value = null;
  }
}

// 暴露方法供外部调用
defineExpose({
  checkForUpdate
});
</script>

<template>
  <!-- 更新对话框 -->
  <Teleport to="body">
    <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center">
      <!-- 遮罩层 -->
      <div 
        class="absolute inset-0 bg-black/60 backdrop-blur-sm"
        @click="closeDialog"
      ></div>
      
      <!-- 对话框内容 -->
      <div class="relative bg-slate-800 rounded-2xl shadow-2xl border border-slate-700/50 w-[480px] max-w-[90vw] overflow-hidden">
        <!-- 头部 -->
        <div class="px-6 py-4 border-b border-slate-700/50 flex items-center justify-between">
          <h3 class="text-lg font-semibold text-slate-100 flex items-center gap-2">
            <svg class="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            发现新版本
          </h3>
          <button 
            v-if="!isDownloading"
            @click="closeDialog"
            class="p-1 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-700/50 transition-colors"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        
        <!-- 内容 -->
        <div class="px-6 py-5">
          <div v-if="updateInfo" class="space-y-4">
            <!-- 版本信息 -->
            <div class="flex items-center gap-3">
              <span class="px-3 py-1 bg-emerald-500/20 text-emerald-400 rounded-full text-sm font-medium">
                v{{ updateInfo.version }}
              </span>
              <span v-if="updateInfo.date" class="text-slate-400 text-sm">
                {{ new Date(updateInfo.date).toLocaleDateString('zh-CN') }}
              </span>
            </div>
            
            <!-- 更新说明 -->
            <div v-if="updateInfo.body" class="bg-slate-900/50 rounded-xl p-4 max-h-48 overflow-y-auto">
              <h4 class="text-sm font-medium text-slate-300 mb-2">更新内容</h4>
              <div class="text-sm text-slate-400 whitespace-pre-wrap">{{ updateInfo.body }}</div>
            </div>
            
            <!-- 下载进度 -->
            <div v-if="isDownloading" class="space-y-2">
              <div class="flex items-center justify-between text-sm">
                <span class="text-slate-300">正在下载更新...</span>
                <span class="text-emerald-400 font-medium">{{ downloadProgress }}%</span>
              </div>
              <div class="h-2 bg-slate-700 rounded-full overflow-hidden">
                <div 
                  class="h-full bg-gradient-to-r from-emerald-500 to-teal-400 transition-all duration-300"
                  :style="{ width: `${downloadProgress}%` }"
                ></div>
              </div>
            </div>
          </div>
          
          <!-- 错误提示 -->
          <div v-if="error" class="mt-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg">
            <p class="text-red-400 text-sm">{{ error }}</p>
          </div>
        </div>
        
        <!-- 底部按钮 -->
        <div class="px-6 py-4 border-t border-slate-700/50 flex justify-end gap-3">
          <button
            v-if="!isDownloading"
            @click="closeDialog"
            class="px-4 py-2 text-sm font-medium text-slate-300 bg-slate-700/50 hover:bg-slate-700 rounded-lg transition-colors"
          >
            稍后更新
          </button>
          <button
            @click="downloadAndInstall"
            :disabled="isDownloading"
            class="px-4 py-2 text-sm font-medium text-white bg-gradient-to-r from-emerald-500 to-teal-500 hover:from-emerald-600 hover:to-teal-600 rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            <svg v-if="isDownloading" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {{ isDownloading ? '正在安装...' : '立即更新' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
