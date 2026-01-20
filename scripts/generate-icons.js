#!/usr/bin/env node
/**
 * 图标生成脚本
 * 使用 sharp 库将 SVG 转换为各种尺寸的 PNG 和 ICNS/ICO
 * 
 * 安装依赖: npm install sharp png2icons --save-dev
 * 运行: node scripts/generate-icons.js
 */

const fs = require('fs');
const path = require('path');

async function generateIcons() {
    let sharp;
    try {
        sharp = require('sharp');
    } catch (e) {
        console.error('请先安装 sharp: npm install sharp --save-dev');
        process.exit(1);
    }

    const iconsDir = path.join(__dirname, '..', 'src-tauri', 'icons');
    const svgPath = path.join(iconsDir, 'app-icon.svg');

    if (!fs.existsSync(svgPath)) {
        console.error('SVG 图标不存在:', svgPath);
        process.exit(1);
    }

    console.log('正在生成图标...');

    // PNG 尺寸列表
    const sizes = [32, 128, 256, 512];

    for (const size of sizes) {
        const outputPath = path.join(iconsDir, size === 256 ? '128x128@2x.png' : `${size}x${size}.png`);
        await sharp(svgPath)
            .resize(size, size)
            .png()
            .toFile(outputPath);
        console.log(`已生成: ${path.basename(outputPath)}`);
    }

    // 生成 icon.png (1024x1024)
    await sharp(svgPath)
        .resize(1024, 1024)
        .png()
        .toFile(path.join(iconsDir, 'icon.png'));
    console.log('已生成: icon.png');

    console.log('\n图标生成完成！');
    console.log('注意: .icns (macOS) 和 .ico (Windows) 需要使用 tauri icon 命令生成:');
    console.log('  npx @tauri-apps/cli icon src-tauri/icons/icon.png');
}

generateIcons().catch(console.error);
