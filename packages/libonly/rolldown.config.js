import { defineConfig } from 'rolldown'
import pkg from './package.json' with { type: 'json' }
import { dts } from 'rolldown-plugin-dts'
import { readFileSync } from 'node:fs'

const external = new RegExp(
  `^(node:|${[...Object.getOwnPropertyNames(pkg.devDependencies ? pkg.devDependencies : []), ...Object.getOwnPropertyNames(pkg.dependencies ? pkg.dependencies : [])].join('|')})`
)

function inlineWasm() {
  return {
    name: 'inline-wasm',
    load(id) {
      if (id.endsWith('.wasm')) {
        const wasmBuffer = readFileSync(id)
        const base64 = wasmBuffer.toString('base64')
        return `export default Buffer.from('${base64}', 'base64')`
      }

      // inline images (watermark / pencil texture)
      if (id.endsWith('.png') || id.endsWith('.jpg') || id.endsWith('.jpeg')) {
        const imgBuffer = readFileSync(id)
        const base64 = imgBuffer.toString('base64')
        const mime = id.endsWith('.png') ? 'image/png' : 'image/jpeg'
        return `export default "data:${mime};base64,${base64}"`
      }
    }
  }
}

export default defineConfig([
  // Koishi Plugin - ES Module
  {
    input: './src/index.ts',
    output: [{ file: 'lib/index.mjs', format: 'es', minify: true }],
    external: external,
    plugins: [inlineWasm()]
  },
  // Koishi Plugin - CommonJS
  {
    input: './src/index.ts',
    output: [{ file: 'lib/index.cjs', format: 'cjs', minify: true }],
    external: external,
    plugins: [inlineWasm()]
  },
  {
    input: './src/index.ts',
    output: [{ dir: 'lib', format: 'es', minify: true }],
    external: external,
    plugins: [dts({ emitDtsOnly: true })]
  }
])
