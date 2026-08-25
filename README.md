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
# 本机 macOS（签名公证，产物在 desktop/dist/）
just desktop-mac
# NOTARIZE=0 just desktop-mac   # 只签名，不公证

# CI 双平台（推 tag，不在本机打包）
TAG=desktop-v0.2.7 just desktop-ship
# 网页端左上角可以重跑失败工作流
# https://github.com/Another-Me-Labs/Another-Claw-Rs/actions
```

```text
1. 开 Azure 订阅
直接去 azure.microsoft.com，没账号注册一个，新用户有 $200 免费额度
国内手机+海外信用卡（VISA/Master）即可；或者用 Azure 中国（21Vianet）但 Trusted Signing 在中国版可能不可用，建议用国际版
2. 开 Trusted Signing (Artifact Signing) 资源
Azure Portal → 搜索 "Trusted Signing Accounts" → Create：

Resource Group：新建一个，如 rg-pusa-signing
Region：选离你近的，比如 East US (eastus)、West Europe (westeurope)。注意 endpoint URL 跟 region 绑定：
East US → https://eus.codesigning.azure.net/
West Central US → https://wcus.codesigning.azure.net/
West Europe → https://weu.codesigning.azure.net/
其他见 region 列表
Pricing Tier：选 Basic（$9.99/月 + 5000 次签名）
3. 提交 Identity Validation（关键步骤）
进到刚建的 Trusted Signing Account → "Identity validation" → Create：

Public Trust type（最常用，签名后 Windows 信任）
选 Individual / Sole Trader / Private Organization / Public Organization 之一
个人选 Individual：传身份证 + 一张账单/银行流水（地址证明）
公司选 Organization：传营业执照 + 一份地址证明
审核时间：1-3 个工作日，通过后 Subject Name（即 publisher）就被锁定了
4. 建 Certificate Profile
Identity Validation 通过后：

Trusted Signing Account → "Certificate profiles" → Create
Profile name 自取（比如 pusa-default）
关联刚才那个已通过的 Identity Validation
5. 建 Microsoft Entra App Registration（让 GitHub Actions 能登 Azure）
Azure Portal → Microsoft Entra ID → App registrations → New registration：

Name: github-actions-pusa
注册后记下 Application (client) ID 和 Directory (tenant) ID
接着配 Federated Credential（让 GitHub OIDC token 能换 Azure token）：

进 App → Certificates & secrets → Federated credentials → Add：
Scenario: GitHub Actions deploying Azure resources
Organization: 你的 GitHub 用户名/组织
Repository: AnotherClaw（或仓库名）
Entity type: Branch，name 写 main（如果你想让 PR 也能签，加一条 entity type = Pull request）
Subject identifier 会自动生成成 repo:你的用户名/AnotherClaw:ref:refs/heads/main
最后给这个 App 授权访问 Trusted Signing：

回到 Trusted Signing Account → Access control (IAM) → Add role assignment
Role: Trusted Signing Certificate Profile Signer
Assign access to: User, group, or service principal → 选刚建的 App Registration
6. 把信息填到 GitHub repo
Secrets（Settings → Secrets and variables → Actions → Secrets）：

Name	Value
AZURE_CLIENT_ID
App Registration 的 Application (client) ID
AZURE_TENANT_ID
Directory (tenant) ID
AZURE_SUBSCRIPTION_ID
Azure 订阅 ID（Portal → Subscriptions）
Variables（同页 → Variables tab；不是机密、明文可见也无所谓）：

Name	Value
AZURE_TS_ENDPOINT
比如 https://eus.codesigning.azure.net/（带尾斜杠）
AZURE_TS_ACCOUNT
Trusted Signing Account 的名字
AZURE_TS_PROFILE
Certificate Profile 的名字（如 pusa-default）
7. 触发 workflow 验证
3 个 secrets + 3 个 vars 都配齐后，再推一次 tag（或手动 workflow_dispatch）。检查 CI 输出：

Detect Azure Trusted Signing config 这一步打印 has_azure=true ✓
Sign MSI with Azure Artifact Signing 不再 skip，跑完无错 ✓
Verify MSI Authenticode signature 输出 Status: Valid ✓
签完下载 MSI，本地 PowerShell 跑 Get-AuthenticodeSignature your.msi 应该看到 SignerCertificate 是你的 Identity Validation 通过的那个 Subject Name。
```