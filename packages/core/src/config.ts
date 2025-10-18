import { Schema } from 'koishi'

export interface Config {
  timeout: number
  zoom: number
  cover: boolean
  quality:
    | 'fine'
    | 'normal'
    | 'coarse'
    | 'superCoarse'
    | 'extraCoarse'
    | 'emboss'
    | 'sketch'
  denoise: boolean
  lightCut: number
  darkCut: number
  shade: boolean
  shadeLimit: number
  shadeLight: number
  toneCount: number
  light: number
  kiss: boolean
  watermark: boolean
  hajimei: boolean
}

export const Config: Schema<Config> = Schema.intersect([
  // 基础设置
  Schema.object({
    timeout: Schema.number()
      .default(30)
      .description('图片接受超时时间（秒）')
      .min(5)
      .max(120)
      .step(1),

    zoom: Schema.number()
      .default(1)
      .description('缩放比例')
      .min(0.5)
      .max(4)
      .step(0.1),

    cover: Schema.boolean().default(false).description('是否裁剪为正方形')
  }).description('基础设置'),

  // 质量设置（模式）
  Schema.object({
    quality: Schema.union([
      Schema.const('fine').description('精细'),
      Schema.const('normal').description('一般'),
      Schema.const('coarse').description('稍粗'),
      Schema.const('superCoarse').description('超粗'),
      Schema.const('extraCoarse').description('极粗'),
      Schema.const('emboss').description('浮雕'),
      Schema.const('sketch').description('线稿')
    ])
      .default('normal')
      .description('线稿质量 / 模式（7种模式）'),

    denoise: Schema.boolean().default(true).description('是否启用降噪')
  }).description('质量设置'),

  // 线迹设置
  Schema.object({
    lightCut: Schema.number()
      .default(128)
      .description('线迹轻重 - 浅色截断值')
      .min(0)
      .max(255)
      .step(1),

    darkCut: Schema.number()
      .default(118)
      .description('线迹轻重 - 深色截断值')
      .min(0)
      .max(255)
      .step(1)
  }).description('线迹设置'),

  // 调子设置
  Schema.object({
    shade: Schema.boolean().default(true).description('是否启用阴影效果'),

    shadeLimit: Schema.number()
      .default(108)
      .description('调子阈值')
      .min(0)
      .max(255)
      .step(1),

    shadeLight: Schema.number()
      .default(80)
      .description('调子轻重')
      .min(0)
      .max(255)
      .step(1),

    toneCount: Schema.number()
      .default(3)
      .description('调子数量（层数）')
      .min(1)
      .max(10)
      .step(1),

    light: Schema.number()
      .default(0)
      .description('额外亮度调整（百分比）')
      .min(-100)
      .max(100)
      .step(1)
  }).description('调子设置'),

  // Kiss 效果
  Schema.object({
    kiss: Schema.boolean()
      .default(true)
      .description('是否启用 Kiss 彩色渐变效果')
  }).description('Kiss 效果'),

  // 水印设置
  Schema.object({
    watermark: Schema.boolean().default(true).description('是否添加水印'),

    hajimei: Schema.boolean().default(false).description('是否使用初回样式水印')
  }).description('水印设置')
])

export const name = 'one-last-image'

export default Config
