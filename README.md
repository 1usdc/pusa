### Serving Your App

# 网页端启动

```bash
just web
just server
# 本地认证
ANOTHERME_BASE_URL=http://host.docker.internal:8881 just web
ANOTHERME_BASE_URL=http://host.docker.internal:8881 just server
# 浏览器访问 http://127.0.0.1:8080/
```

# 桌面端启动

```bash
just desktop
```

# 桌面端打包
```bash
# 凭证：cp .env.signing.example .env.signing（勿提交）

# 本机 macOS（Developer ID 签名 + 公证 → desktop/dist/*.dmg）
just desktop-mac
# NOTARIZE=0 just desktop-mac   # 只签名，不公证

# 本机 Windows（未签名 NSIS 安装器 → desktop/dist/*-setup.exe）
just desktop-windows
# SIGN=1 just desktop-windows   # 可选：打包后 Azure Artifact Signing

# 上传到 GitHub Release（各平台独立发版）
TAG=desktop-v0.2.7 BUILD=1 just release-mac      # macOS 机器
TAG=desktop-v0.2.7 BUILD=1 just release-windows  # Windows 机器
# 或已打包好：TAG=desktop-v0.2.7 just release-windows
```

### 发版流程示例（Windows）

```bash
# 1. 提交代码
git push

# 2. Windows 本机（需 dx CLI）
just desktop-windows

# 3. 上传 GitHub Release
TAG=desktop-v0.2.7 just release-windows
# 或一步：TAG=desktop-v0.2.7 BUILD=1 just release-windows

# macOS 同理：just desktop-mac → TAG=... just release-mac
```

### Windows 可选签名（Azure Artifact Signing）

默认不签名。需要签名时：

1. Azure Portal 创建 **Artifact Signing** 账户，完成 Identity Validation，建 Certificate Profile
2. 本机安装 Client Tools + 登录 Azure：
   ```powershell
   winget install -e --id Microsoft.Azure.ArtifactSigningClientTools
   az login
   ```
3. 配置 `.env.signing`（见 `.env.signing.example`）：
   ```ini
   AZURE_TS_ENDPOINT=https://eus.codesigning.azure.net
   AZURE_TS_ACCOUNT=你的账户名
   AZURE_TS_PROFILE=你的 Profile 名
   ```
4. 签名打包：
   ```powershell
   SIGN=1 just desktop-windows
   # 或对已有安装器：pwsh scripts/windows-sign.ps1
   ```

定价约 **$9.99/月**（Basic，含 5000 次签名）。Endpoint 区域须与账户一致（East US → `https://eus.codesigning.azure.net`）。