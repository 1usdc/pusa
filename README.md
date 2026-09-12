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

Windows 前置：Rust 用 `x86_64-pc-windows-msvc`，需要 MSVC C++ 生成工具（`link.exe`）：

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

```bash
# 版本号：just desktop-mac 默认把 desktop/Cargo.toml patch +1；BUMP=0 则不改
# 发版 tag = desktop-v{version}（也可用 TAG= 覆盖）
# 可选签名凭证：cp .env.signing.example .env.signing（勿提交）

# 本机 macOS（Developer ID 签名 + 公证 → desktop/dist/*.dmg）
just desktop-mac
# 覆盖旧版本（不升版号）
BUMP=0 just desktop-mac
# NOTARIZE=0 just desktop-mac   # 只签名，不公证
# 把本机已打好的 DMG 挂到 GitHub Release
just desktop-mac-release

# 本机 Windows（未签名 NSIS 安装器 → desktop/dist/Pusa_*-setup.exe）
# 安装器会把 vendor 里的 pusa_core.dll 装到程序目录（与 exe 同级）
just desktop-windows
# SIGN=1 just desktop-windows   # 可选：打包后 Azure Artifact Signing

# 上传到 GitHub Release（tag 自动取自 desktop/Cargo.toml）
BUILD=1 just release-mac       # macOS
BUILD=1 just release-windows   # Windows
# 或已打包好：just release-windows
```

### 发版流程示例（Windows）

```bash
# 1. 改版本并提交
#    desktop/Cargo.toml → version = "0.2.7"
git push

# 2. 打包并上传（生成 tag desktop-v0.2.7）
BUILD=1 just release-windows
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
