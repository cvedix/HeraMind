# GitHub Actions 自动签名配置指南

## 方案选择

### 方案 A: Apple ID 签名（免费，推荐用于开源项目）

**优点**: 免费，无需开发者账号
**缺点**: 用户首次打开需要右键选择"打开"

#### 配置步骤

1. **生成应用专用密码**
   - 访问 https://appleid.apple.com
   - 登录后选择「安全」→「应用专用密码」→「生成」
   - 标签填写: `HeraMind GitHub Actions`
   - 保存生成的密码

2. **添加 GitHub Secrets**
   
   访问: https://github.com/camthink-ai/HeraMind/settings/secrets/actions
   
   添加以下 Secrets:
   
   | Secret 名称 | 值 | 必填 |
   |------------|-----|------|
   | `APPLE_ID` | 你的 Apple ID 邮箱 | 是 |
   | `APPLE_PASSWORD` | 刚生成的应用专用密码 | 是 |
   | `APPLE_TEAM_ID` | 团队 ID（可选） | 否 |

3. **获取 Team ID（可选）**
   ```bash
   # 在 macOS 终端运行
   security find-identity -v -p codesigning
   # 输出中的 10 位字符即为 Team ID
   ```

---

### 方案 B: Developer ID 证书签名（$99/年，最佳用户体验）

**优点**: 用户双击即可安装，无警告
**缺点**: 需要苹果开发者账号

#### 配置步骤

1. **创建开发者证书**
   - 加入 Apple Developer Program: https://developer.apple.com/programs/
   - 访问: https://developer.apple.com/account/resources/certificates/list
   - 点击「+」创建「Developer ID Application」证书
   - 下载并安装到钥匙串

2. **导出证书**
   
   在 macOS 上运行项目中的脚本:
   ```bash
   .github/scripts/export-certificate.sh
   ```
   
   这会生成证书的 base64 编码并复制到剪贴板。

3. **添加 GitHub Secrets**
   
   访问: https://github.com/camthink-ai/HeraMind/settings/secrets/actions
   
   | Secret 名称 | 值 | 必填 |
   |------------|-----|------|
   | `APPLE_CERTIFICATE` | 证书的 base64 内容 | 是 |
   | `APPLE_CERTIFICATE_PASSWORD` | 导出时生成的密码 | 是 |
   | `APPLE_SIGNING_IDENTITY` | 证书身份（如 `Developer ID Application: Your Name (TEAMID)`） | 是 |

---

## 触发自动构建

### 方式 1: 推送标签（推荐）

```bash
# 创建新版本标签
git tag v0.3.0

# 推送标签触发构建
git push origin v0.3.0
```

### 方式 2: 手动触发

1. 访问: https://github.com/camthink-ai/HeraMind/actions
2. 选择「Build and Release」workflow
3. 点击「Run workflow」
4. 输入版本号（如 `v0.3.0`）

---

## 构建产物

构建成功后，DMG 文件会自动上传到 GitHub Release:

https://github.com/camthink-ai/HeraMind/releases

---

## 用户安装说明

### 未签名 / Apple ID 签名版本

如果用户看到「无法验证开发者」警告:

**方法 1**: 右键 `HeraMind.app` → 选择「打开」

**方法 2**: 终端运行
```bash
xattr -d com.apple.quarantine /Applications/HeraMind.app
open /Applications/HeraMind.app
```

### Developer ID 签名版本

直接双击 DMG 并拖拽到 Applications 文件夹即可。
