**【本项目不再维护】**
由于本人时间、精力、能力水平的限制，继续维护这个项目不仅难以及时修复问题，而且可能引入更多不确定性。现将此项目归档，您仍然可以 Fork 并自行修改，但将不能再提交 Issue 或 Pull Request，也不会收到任何更新。

**【このプロジェクトはメンテナンス終了】**
私の時間・労力・能力の限界により、このプロジェクトを継続してメンテナンスすることは、問題への迅速な対応が難しくなるだけでなく、新たな不確実性を招く恐れがあります。つきましては、本プロジェクトをアーカイブ（読み取り専用）とします。引き続き Fork してご自身で修正いただくことは可能ですが、Issue や Pull Request の受け付けは終了させていただきます。また、更新を受け取ることもできなくなります。

**【This project is no longer maintained】**
Due to limitations of my time, energy, and skill, continuing to maintain this project would not only make it difficult to address issues promptly, but may also introduce further uncertainty. This project has now been archived. You may still fork it and modify it yourself, but you will no longer be able to submit Issues or Pull Requests, nor will you receive any updates.

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
