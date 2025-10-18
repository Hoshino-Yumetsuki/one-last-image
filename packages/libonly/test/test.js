/**
 * One Last Image 库测试脚本
 * 测试基本的图像处理功能
 */

import { one_last_image } from '../lib/index.mjs'
import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

// 测试配置
const TEST_CONFIGS = [
  {
    name: 'default',
    description: '默认配置',
    config: undefined
  },
  {
    name: 'fine',
    description: '精细线稿',
    config: {
      quality: 'fine',
      kiss: true
    }
  },
  {
    name: 'coarse',
    description: '粗线稿',
    config: {
      quality: 'coarse',
      kiss: true
    }
  },
  {
    name: 'no-kiss',
    description: '无彩色渐变',
    config: {
      quality: 'normal',
      kiss: false
    }
  },
  {
    name: 'sketch',
    description: '纯线稿模式',
    config: {
      quality: 'sketch',
      kiss: false
    }
  },
  {
    name: 'custom',
    description: '自定义参数',
    config: {
      zoom: 1.5,
      quality: 'normal',
      denoise: true,
      light_cut: 140,
      dark_cut: 110,
      kiss: true,
      light: 10
    }
  }
]

function log(message, type = 'info') {
  const colors = {
    info: '\x1b[36m',
    success: '\x1b[32m',
    error: '\x1b[31m',
    warn: '\x1b[33m'
  }
  const reset = '\x1b[0m'
  console.log(`${colors[type]}${message}${reset}`)
}

async function testOneLastImage() {
  log('='.repeat(60), 'info')
  log('One Last Image 库测试', 'info')
  log('='.repeat(60), 'info')

  // 检查测试图片
  const inputPath = join(__dirname, 'input.jpg')
  if (!existsSync(inputPath)) {
    log('错误: 找不到测试图片 test/input.jpg', 'error')
    log('请在 test 目录下放置一张名为 input.jpg 的测试图片', 'warn')
    process.exit(1)
  }

  // 创建输出目录
  const outputDir = join(__dirname, 'output')
  if (!existsSync(outputDir)) {
    mkdirSync(outputDir, { recursive: true })
  }

  // 读取测试图片
  log('\n读取测试图片...', 'info')
  const imageBuffer = readFileSync(inputPath)
  log(
    `✓ 成功读取图片 (${(imageBuffer.length / 1024).toFixed(2)} KB)`,
    'success'
  )

  // 运行测试
  let successCount = 0
  let failCount = 0

  for (const test of TEST_CONFIGS) {
    log(`\n测试: ${test.description} (${test.name})`, 'info')

    try {
      const startTime = Date.now()
      const result = one_last_image(imageBuffer, test.config)
      const duration = Date.now() - startTime

      const outputPath = join(outputDir, `output-${test.name}.png`)
      writeFileSync(outputPath, result)

      log(`✓ 处理成功`, 'success')
      log(`  - 耗时: ${duration}ms`, 'info')
      log(`  - 输出大小: ${(result.length / 1024).toFixed(2)} KB`, 'info')
      log(`  - 保存至: ${outputPath}`, 'info')

      successCount++
    } catch (error) {
      log(`✗ 处理失败: ${error.message}`, 'error')
      failCount++
    }
  }

  // 测试总结
  log(`\n${'='.repeat(60)}`, 'info')
  log('测试总结', 'info')
  log('='.repeat(60), 'info')
  log(`总测试数: ${TEST_CONFIGS.length}`, 'info')
  log(`成功: ${successCount}`, 'success')
  if (failCount > 0) {
    log(`失败: ${failCount}`, 'error')
  }
  log(`\n输出目录: ${outputDir}`, 'info')

  if (failCount > 0) {
    process.exit(1)
  }
}

// 运行测试
testOneLastImage().catch((error) => {
  log(`\n测试运行失败: ${error.message}`, 'error')
  console.error(error)
  process.exit(1)
})
