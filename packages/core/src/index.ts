import { type Context, h, Logger } from 'koishi'
import type Config from './config'
import { createLogger, setLoggerLevel } from './utils/logger'
import { processOneLastImage } from './utils/imageProcessing'
// inlined watermark asset (data:<mime>;base64,...) provided by rolldown
//@ts-expect-error
import logoDataUri from './assets/one-last-image-logo2.png'

export let logger: Logger

export function apply(ctx: Context, config: Config) {
  logger = createLogger(ctx)
  setupLogger(config)

  ctx.command('oli', 'One Last Image 图片处理').action(async ({ session }) => {
    // 获取用户消息中的图片（优先元素中的 img, 若没有则尝试引用内容或 mface）
    let [img] = h.select(session.elements, 'img')

    if (!img && session.quote) {
      const qImg = h
        .select(session.quote.content, 'img')
        .map((item) => item.attrs.src)[0]
      const qFace = h
        .select(session.quote.content, 'mface')
        .map((item) => item.attrs.url)[0]
      const src = qImg || qFace
      if (src) {
        img = { attrs: { src } as any } as any
      }
    }

    if (!img) {
      return '请发送要处理的图片~'
    }

    const imageUrl = img.attrs.src

    try {
      // 获取图片数据
      const imageBuffer = Buffer.from(
        await ctx.http.get(imageUrl, {
          responseType: 'arraybuffer'
        })
      )

      // 构建传入图像处理的配置（将 camelCase 转为 snake_case 以匹配 Rust 端）
      const cfg: Record<string, any> = {
        zoom: config.zoom,
        cover: config.cover,
        quality: config.quality,
        denoise: config.denoise,
        light_cut: config.lightCut,
        dark_cut: config.darkCut,
        shade: config.shade,
        shade_limit: config.shadeLimit,
        shade_light: config.shadeLight,
        kiss: config.kiss,
        watermark: config.watermark,
        hajimei: config.hajimei
      }

      // 如果启用了 watermark，提取 data URI 的 base64 部分并传入 wasm
      if (config.watermark && typeof logoDataUri === 'string') {
        const parts = logoDataUri.split(',')
        if (parts.length === 2) {
          cfg.watermark_image = parts[1]
        }
      }

      // 处理图片 - 使用 one-last-image 线稿效果，并传递配置
      const processedBuffer = await processOneLastImage(imageBuffer, cfg)

      // 发送处理后的图片
      await session.send(h.image(processedBuffer, 'image/png'))

      return
    } catch (error) {
      logger.error('处理图片时发生错误', { error })
      return '图片处理失败了喵~'
    }
  })
}

function setupLogger(config: Config) {
  if (config.isLog) {
    setLoggerLevel(Logger.DEBUG)
  }
}

export * from './config'
