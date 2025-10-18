/**
 * One Last Image 独立图像处理库
 * 可以独立于 Koishi 使用
 */

import { one_last_image as one_last_image_rslib } from './wasm/bindings'

/**
 * 将图片转换为线稿效果
 * @param imageBuffer 图片的 Buffer 或 Uint8Array
 * @param config 可选配置对象
 * @returns 处理后的图片 Buffer
 */
export function one_last_image(
  imageBuffer: Buffer | Uint8Array,
  config?: Record<string, any>
): Buffer {
  try {
    const input =
      imageBuffer instanceof Buffer ? new Uint8Array(imageBuffer) : imageBuffer
    const configJson =
      config && Object.keys(config).length > 0
        ? JSON.stringify(config)
        : undefined
    const output = one_last_image_rslib(input, configJson)
    return Buffer.from(output)
  } catch (err) {
    console.warn(
      'processImage: Processing failed, returning original buffer',
      err
    )
    return Buffer.isBuffer(imageBuffer) ? imageBuffer : Buffer.from(imageBuffer)
  }
}
