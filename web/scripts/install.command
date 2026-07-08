#!/bin/bash
# HeraMind 自动安装脚本

DMG_PATH="$HOME/Downloads/HeraMind_0.1.0_aarch64.dmg"

echo "🔧 HeraMind 安装程序"
echo "===================="

# 检查 DMG 是否存在
if [ ! -f "$DMG_PATH" ]; then
    echo "❌ 未找到安装包，请将 DMG 文件放到下载文件夹"
    echo "   文件名应为: HeraMind_0.1.0_aarch64.dmg"
    read -p "按回车键退出..."
    exit 1
fi

echo "✓ 找到安装包"

# 移除隔离属性
echo "🔓 正在处理权限..."
xattr -cr "$DMG_PATH" 2>/dev/null

# 挂载 DMG
echo "📦 正在打开安装包..."
open "$DMG_PATH"

# 等待用户拖拽应用
echo ""
echo "请执行以下操作："
echo "1. 在打开的窗口中，将 HeraMind.app 拖拽到\"应用程序\"文件夹"
echo "2. 等待复制完成后，按回车键继续..."
read

# 安装完成，处理应用
APP_PATH="/Applications/HeraMind.app"
if [ -d "$APP_PATH" ]; then
    echo "✓ 应用已安装"
    
    # 移除应用的隔离属性
    echo "🔓 正在设置应用权限..."
    xattr -cr "$APP_PATH" 2>/dev/null
    
    echo ""
    echo "✅ 安装完成！"
    echo "   现在可以在启动台中找到 HeraMind"
    echo "   如果首次打开提示损坏，请在\"系统设置 → 隐私与安全性\"中点击\"仍要打开\""
else
    echo "⚠️  未检测到应用，请确认是否已拖拽到应用程序文件夹"
fi

echo ""
read -p "按回车键关闭..."
