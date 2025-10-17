import { logger } from '../index'
import { one_last_image_with_config } from '../wasm/bindings'

/**
 * 处理用户上传的图片，应用 one-last-image 线稿效果
 * 支持传入配置对象
 */
export async function processOneLastImage(
  imageBuffer: Buffer,
  config: Record<string, any> = {}
): Promise<Buffer> {
  try {
    const configJson = JSON.stringify(config)
    const out = one_last_image_with_config(
      new Uint8Array(imageBuffer),
      configJson
    )
    return Buffer.from(out)
  } catch (err) {
    logger?.warn?.(
      'processOneLastImage: decode failed, return original buffer',
      {
        err
      }
    )
    return imageBuffer
  }
}
