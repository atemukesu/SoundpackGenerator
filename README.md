# Soundpack Generator

基于 Tauri + Vue 3 的 Minecraft 音色包生成工具。从 SF2 音色库生成适用于 Extended Note Block 模组的资源包。

## 功能

- 从 SF2 文件选取乐器生成音色包
- 支持 128 种 GM 乐器 + 鼓组
- 波形预览与编辑（选区裁剪、增益调整）
- 批量编辑（多样本同时调整增益/裁剪区域）
- 导出为 ZIP 压缩包
- 支持自定义音频导入

## 技术栈

- **前端**: Vue 3 + Tailwind CSS + WaveSurfer.js
- **后端**: Rust + Tauri 2
- **依赖**: FluidSynth（音频合成）、FFmpeg（音频转码）

## 开发

```bash
pnpm install
pnpm tauri dev
```

## 构建

```bash
pnpm tauri build
```

## 许可证

GPL-3.0
