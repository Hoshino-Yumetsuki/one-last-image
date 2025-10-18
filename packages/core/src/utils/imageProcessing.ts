import { logger } from '../index'
import { one_last_image } from '../wasm/bindings'

export async function processOneLastImage(
  imageBuffer: Buffer,
  config?: Record<string, any>
): Promise<Buffer> {
  try {
    const configJson =
      config && Object.keys(config).length > 0
        ? JSON.stringify(config)
        : undefined

    const out = one_last_image(new Uint8Array(imageBuffer), configJson)

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
