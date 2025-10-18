import { type Context, h, type Logger } from 'koishi'
import type Config from './config'
import { processOneLastImage } from './utils/imageProcessing'
// inlined watermark asset (data:<mime>;base64,...) provided by rolldown
//@ts-expect-error
import logoDataUri from './assets/one-last-image-logo2.png'
//@ts-expect-error
import pencilTextureUri from './assets/pencil-texture.jpg'

export let logger: Logger

export function apply(ctx: Context, config: Config) {
  ctx
    .command('oli', 'One Last Image 图片处理')
    .option('watermark', '-w [enable:boolean] 是否添加水印（默认使用配置）')
    .action(async ({ session, options }) => {
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
      await session.send(`请发送要处理的图片，等待 ${config.timeout} 秒...`)

      const imageUrl = await session.prompt(
        async (s) => {
          if (s.userId !== session.userId || s.channelId !== session.channelId)
            return null

          let [i] = h.select(s.elements, 'img')
          if (!i && s.quote) {
            const qImg = h
              .select(s.quote.content, 'img')
              .map((item) => item.attrs.src)[0]
            const qFace = h
              .select(s.quote.content, 'mface')
              .map((item) => item.attrs.url)[0]
            const src = qImg || qFace
            if (src) i = { attrs: { src } as any } as any
          }

          if (i) return i.attrs.src
          if (s.elements.length > 0) return 'CANCEL'
          return null
        },
        { timeout: config.timeout * 1000 }
      )

      if (!imageUrl) {
        return '等待图片超时，已取消操作'
      }

      if (imageUrl === 'CANCEL') {
        return '未检测到图片，已取消操作'
      }

      img = { attrs: { src: imageUrl } as any } as any
    }

    const imageUrl = img.attrs.src

    try {
      const imageBuffer = Buffer.from(
        await ctx.http.get(imageUrl, {
          responseType: 'arraybuffer'
        })
      )

      // 水印配置：优先使用命令选项，否则使用配置文件
      const useWatermark = options.watermark !== undefined ? options.watermark : config.watermark

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
        watermark: useWatermark,
        hajimei: config.hajimei
      }

      if (useWatermark && typeof logoDataUri === 'string') {
        const parts = logoDataUri.split(',')
        if (parts.length === 2) {
          cfg.watermark_image = parts[1]
        }
      }

      if (config.shade && typeof pencilTextureUri === 'string') {
        const parts = pencilTextureUri.split(',')
        if (parts.length === 2) {
          cfg.pencil_texture = parts[1]
        }
      }

      const processedBuffer = await processOneLastImage(imageBuffer, cfg)

      await session.send(h.image(processedBuffer, 'image/png'))

      return
    } catch (error) {
      logger.error('处理图片时发生错误', { error })
      return '图片处理失败了喵~'
    }
  })
}

export * from './config'
