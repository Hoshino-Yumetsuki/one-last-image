# koishi-plugin-one-last-image

[![npm](https://img.shields.io/npm/v/koishi-plugin-one-last-image?style=flat-square)](https://www.npmjs.com/package/koishi-plugin-one-last-image)
[![License](https://img.shields.io/github/license/Hoshino-Yumetsuki/koishi-plugin-one-last-image?style=flat-square)](https://github.com/Hoshino-Yumetsuki/koishi-plugin-one-last-image/blob/main/LICENSE)

One Last Image 线稿效果生成器 - 将图片转换为精美的线稿风格

灵感来源于 [itorr/one-last-image](https://github.com/itorr/one-last-image)，使用 Rust + WebAssembly 重写核心图像处理逻辑，提供更高的性能和更好的跨平台兼容性。

## 特性

- **多种线稿质量模式**：精细、一般、稍粗、超粗、极粗、浮雕、线稿
- **Kiss 彩色渐变效果**：为线稿添加梦幻般的彩色渐变
- **阴影效果**：可配置的阴影层次和强度
- **水印支持**：内置水印，支持初回样式
- **高性能**：基于 Rust + WebAssembly 实现
- **灵活配置**：丰富的参数调整选项
- **双模式使用**：既可作为 Koishi 插件，也可作为独立库使用

## 安装

### 作为独立库使用

```bash
# 使用 npm
npm install koishi-plugin-one-last-image

# 使用 yarn
yarn add koishi-plugin-one-last-image

# 使用 pnpm
pnpm add koishi-plugin-one-last-image
```

## 使用方法

### Koishi 插件模式

在 Koishi 中启用插件后，使用以下命令：

```
oli [图片]           # 处理图片
oli -w false [图片]  # 不添加水印
oli -w true [图片]   # 强制添加水印
```

**支持的图片输入方式：**
- 直接发送图片
- 引用消息中的图片
- 命令后等待发送图片（可配置超时时间）

### 独立库模式

```typescript
import { one_last_image } from 'one-last-image'
import fs from 'fs'

// 基础使用（使用默认配置）
const imageBuffer = fs.readFileSync('input.jpg')
const result = one_last_image(imageBuffer)
fs.writeFileSync('output.png', result)

// 自定义配置
const result = one_last_image(imageBuffer, {
  zoom: 1.5,           // 缩放比例
  quality: 'fine',     // 线稿质量
  kiss: true,          // 启用彩色渐变
  watermark: false,    // 不添加水印
  light_cut: 128,      // 浅色截断值
  dark_cut: 118        // 深色截断值
})
```

## 配置选项（独立库模式）

| 参数              | 类型    | 默认值   | 说明                                                                                   |
| ----------------- | ------- | -------- | -------------------------------------------------------------------------------------- |
| `zoom`            | number  | 1        | 缩放比例（0.5-4）                                                                      |
| `cover`           | boolean | false    | 是否裁剪为正方形                                                                       |
| `quality`         | string  | 'normal' | 线稿质量：'fine', 'normal', 'coarse', 'superCoarse', 'extraCoarse', 'emboss', 'sketch' |
| `denoise`         | boolean | true     | 是否启用降噪                                                                           |
| `light_cut`       | number  | 128      | 浅色截断值（0-255）                                                                    |
| `dark_cut`        | number  | 118      | 深色截断值（0-255）                                                                    |
| `shade`           | boolean | true     | 是否启用阴影效果                                                                       |
| `shade_limit`     | number  | 108      | 调子阈值（0-255）                                                                      |
| `shade_light`     | number  | 80       | 调子轻重（0-255）                                                                      |
| `light`           | number  | 0        | 额外亮度调整（-100 到 100）                                                            |
| `kiss`            | boolean | true     | 是否启用彩色渐变效果                                                                   |
| `watermark`       | boolean | false    | 是否添加水印（需要提供 watermark_image）                                               |
| `watermark_image` | string  | -        | 水印图片的 base64 编码（不含 data URI 前缀）                                           |
| `hajimei`         | boolean | false    | 是否使用初回样式水印                                                                   |
| `pencil_texture`  | string  | -        | 铅笔纹理图片的 base64 编码（用于阴影效果）                                             |

## 效果预览

### 线稿质量对比

- **fine（精细）**：细腻的线条，适合细节丰富的图片
- **normal（一般）**：平衡的效果，适合大多数场景
- **coarse（稍粗）**：较粗的线条，风格化效果
- **superCoarse（超粗）**：超粗线条，强烈的艺术感
- **extraCoarse（极粗）**：极粗线条，极简风格
- **emboss（浮雕）**：浮雕效果
- **sketch（线稿）**：纯线稿模式

### Kiss 彩色渐变

启用 Kiss 效果后，线稿会叠加彩色渐变，从暖色调过渡到冷色调。

## 开发

### 环境要求

- Node.js >= 22
- Rust >= 1.70
- wasm-pack

### 构建步骤

```bash
# 克隆仓库
git clone https://github.com/Hoshino-Yumetsuki/koishi-plugin-one-last-image.git
cd koishi-plugin-one-last-image

# 安装依赖
yarn install

# 构建 Rust 部分
cd packages/rslib
yarn build

# 构建 TypeScript 部分
cd ../core
yarn build
```

## 许可证

本项目采用 [MPL-2.0](LICENSE) 许可证。

## 致谢

- 原始项目：[itorr/one-last-image](https://github.com/itorr/one-last-image)

## 问题反馈

如果您在使用过程中遇到问题，欢迎在 [GitHub Issues](https://github.com/Hoshino-Yumetsuki/koishi-plugin-one-last-image/issues) 提出。