# CLAUDE.md

## 项目定位

- 项目全称：**迷彩侠（MCX）**—— 上海迷彩侠实业有限公司的军旅实业合作平台
- 后端：Rust + Axum，前端：Vue 3 + Vite（移动端优先 H5）
- 数据库：MySQL（`mcx-yl`）+ Redis（缓存/验证码/会话）
- 存储：阿里云 OSS（图片与资质文件）
- 部署：Nginx 反向代理 → Rust 单体应用（systemd 管理）
- 域名：`mcx59481.cn` / `www.mcx59481.cn`


## 🔴 开发工作流（每次任务必经，不可跳过）

**以下规则对所有开发任务强制生效，任务开始前必须先匹配 skill，完成后必须 review：**

### 任务开始时（必须执行）
- 需求分析 / 功能规划 → 立即加载 `product-manager`
- UI / 交互 / 视觉设计 → 立即加载 `frontend-ui-engineering`（或 `ui-ux-pro-max`）
- 前端代码变更（Vue/JS/CSS/HTML）→ 立即加载 `frontend-developer`
- 后端代码变更（Rust/SQL/配置）→ 立即加载 `backend-developer`
- 多个 skill 可并行加载，不要串行等待

### 任务完成后（必须执行）
- 加载 `qa-engineer` 对本次所有变更做全面检查
- 发现问题 → 立即修复 → 再次检查
- 只有 review 通过（零问题）才算任务完成
- **禁止**在未完成 review 的情况下宣布任务完成

### 违规自查
- 如果发现自己在写前端代码但没加载 `frontend-developer`，停下来，先加载 skill
- 如果发现自己在写后端代码但没加载 `backend-developer`，停下来，先加载 skill
- 如果准备说"完成"但还没跑 `qa-engineer` review，这不是真正完成

## 品牌与产品体系

### 迷彩侠（品牌平台）
- 定位：以退役军人为核心的军旅实业合作平台，帮助退役军人就业、组织退役军人服务社会
- 养老到家服务是当前 MVP 落地场景，不是品牌边界的全部
- 官网首页（`/`）承担品牌介绍、信任建立，提供"侠到家"和"侠伴行"两个子产品的入口

### 侠到家（家庭服务端）
- 面向**老人及其家庭**的 H5 移动端服务平台
- 路由前缀：`/xia-dao-jia/*`
- 核心功能：陪诊、陪伴聊天、家政清洁、送餐到家、代买代办
- 用户流程：短信登录 → 选择地区 → 浏览服务 → 预约 → 追踪订单
- 页面路由：
  - `/xia-dao-jia/login` — 登录页
  - `/xia-dao-jia/home` — 首页
  - `/xia-dao-jia/services` — 服务列表
  - `/xia-dao-jia/addresses` — 地址管理
  - `/xia-dao-jia/orders` — 订单列表
  - `/xia-dao-jia/profile` — 个人中心

### 侠伴行（退役军人端）
- 面向**退役军人服务者**的移动端工作平台
- 路由：`/veteran-portal`（登录后进入工作台）
- 核心功能：入驻申请 → 实名认证 → 接单/拒单 → 服务执行 → 收入结算
- 底部导航：首页（工作台）、订单、培训、我的
- 页面路由：
  - `/veteran-join` — 入驻申请页
  - `/veteran-portal` — 工作台（登录后）

## 后端架构

### 分层结构
```
src/
├── main.rs              # #[tokio::main] 异步入口
├── app.rs               # 启动引导（配置、DB、路由、日志）
├── routes.rs            # /api/* 路由注册
├── common/              # 通用层
│   ├── api.rs           # 通用响应类型（Health 等）
│   ├── errors.rs        # 错误类型
│   └── phone.rs         # 手机号校验
├── domains/             # 业务领域层（单体分层，不做微服务）
│   ├── auth/            # 退役军人登录认证
│   ├── catalog/         # 服务目录
│   ├── customer/        # 用户端（登录、地址、个人信息）
│   ├── onboarding/      # 退役军人入驻
│   ├── order/           # 订单 + WebSocket 推送
│   └── veteran/         # 退役军人信息
└── infra/               # 基础设施层
    ├── config.rs        # 环境变量配置读取
    ├── sms.rs           # 阿里云短信服务
    └── state.rs         # 全局 AppState（DB + Redis + SMS + Broadcaster）
```

每个 domain 内部结构：`mod.rs` / `handler.rs` / `dto.rs` / `entity.rs` / `repository.rs` / `service.rs`

### 依赖栈
- `sqlx` — MySQL
- `redis` — Redis
- `opendal` + `services-oss` — 阿里云 OSS
- `anyhow` — 错误处理
- `tracing` + `tracing-appender` — 日志（控制台 + 按天滚动文件）

## 前端架构

### 路由系统
- 基于 `pushState` 的手动路由（非 vue-router），`App.vue` 根据 `currentPath` 动态切换页面
- 路由映射表在 `frontend/src/app/routes.js`

### 分层结构
```
frontend/src/
├── main.js              # Vue 应用入口
├── App.vue              # 根组件（路由分发）
├── styles.css           # 全局样式 + CSS 变量
├── app/                 # 路由与元信息
│   ├── routes.js        # 路径→页面映射
│   ├── useAppRouting.js # pushState 路由
│   └── usePageMeta.js   # title/description
├── pages/               # 页面组件（按路由对应）
│   ├── HomePage.vue     # /
│   ├── VeteranJoinPage.vue       # /veteran-join
│   ├── VeteranPortalPage.vue     # /veteran-portal
│   ├── CustomerLoginPage.vue     # /xia-dao-jia/login
│   ├── CustomerHomePage.vue      # /xia-dao-jia/home
│   ├── CustomerServicesPage.vue  # /xia-dao-jia/services
│   ├── CustomerOrdersPage.vue    # /xia-dao-jia/orders
│   ├── CustomerAddressesPage.vue # /xia-dao-jia/addresses
│   └── CustomerProfilePage.vue   # /xia-dao-jia/profile
├── features/            # 业务 composables
│   ├── customer/        # useCustomerApp
│   ├── onboarding/      # useVeteranJoinForm
│   └── portal/          # useVeteranPortalLogin
├── components/          # 共享 UI
│   ├── customer/        # CustomerAppShell
│   └── veteran/         # VeteranWorkbench/Orders/Training/Profile
├── shared/              # 工具
│   ├── api/             # auth.js / customer.js / http.js / onboarding.js / veteran-orders.js
│   └── constants/       # localStorage key 常量
├── content/             # 官网内容数据
└── assets/              # 图片等静态资源
```

### 关键常量（localStorage key）
- `mcx:portal-phone` — 门户手机号
- `mcx:customer-token` — 侠到家登录 token
- `mcx:veteran-token` — 侠伴行登录 token
- `mcx:customer-region` — 用户所选地区

## 后端配置读取机制

加载流程：
1. `dotenvy::dotenv()` 从项目根目录 `.env` 加载环境变量到进程
2. `AppConfig::from_env()` 在 `src/infra/config.rs` 中读取

环境变量清单：

| 环境变量 | 必填 | 说明 | 默认值 |
|---|---|---|---|
| `DATABASE_URL` | ✅ | MySQL 连接字符串 | — |
| `REDIS_URL` | ✅ | Redis 连接字符串 | — |
| `SERVER_HOST` | 否 | 监听 IP | `127.0.0.1` |
| `APP_PORT` | 否 | 监听端口 | `9000` |
| `ALIYUN_SMS_ENDPOINT` | 否 | 短信 API 地址 | `https://dysmsapi.aliyuncs.com/` |
| `ALIYUN_SMS_ACCESS_KEY_ID` | ✅ | 短信 AccessKey（也可用 `ALIYUN_ACCESS_KEY_ID`） | — |
| `ALIYUN_SMS_ACCESS_KEY_SECRET` | ✅ | 短信 AccessKey Secret（也可用 `ALIYUN_ACCESS_KEY_SECRET`） | — |
| `ALIYUN_SMS_SIGN_NAME` | ✅ | 短信签名 | — |
| `ALIYUN_SMS_TEMPLATE_CODE` | ✅ | 短信模板 Code | — |

## 本地开发规则

- **不要重新创建 `.env`**。复用现有的根目录 `.env`。
- `.env` 被 Git 忽略，不要提交。
- 后端默认端口 `9000`。
- 前端 `127.0.0.1:5173`，Vite 将 `/api/*` 代理到 `http://127.0.0.1:9000`。
- 前端应使用同源 `/api` 调用，不要硬编码完整后端 URL。
- 本地开发时后端静态文件查找优先级：`frontend/dist` → `frontend`。

## 数据库约定

- MySQL 数据库名：`mcx-yl`。
- 因为数据库名含连字符，SQL 中必须用反引号引用：`` `mcx-yl` ``。
- MySQL 和 Redis 连接信息从 `.env` 读取。

## API 概览

### 通用
| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/health` | 健康检查 |

### 用户端（侠到家）
| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/customer/auth/send-sms-code` | 发送短信验证码 |
| POST | `/api/customer/auth/login-by-sms` | 短信验证码登录 |
| GET | `/api/customer/me` | 获取个人信息 |
| POST | `/api/customer/me/region` | 更新地区 |
| GET | `/api/customer/addresses` | 地址列表 |
| POST | `/api/customer/addresses` | 新增地址 |
| POST | `/api/customer/addresses/:id` | 更新地址 |
| GET | `/api/service-items` | 服务目录 |
| GET | `/api/orders` | 订单列表 |
| POST | `/api/orders` | 创建订单 |
| GET | `/api/orders/:id` | 订单详情 |

### 退役军人端（侠伴行）
| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/auth/send-sms-code` | 发送短信验证码 |
| POST | `/api/auth/register-by-sms` | 短信验证码注册 |
| POST | `/api/onboarding/veteran-join` | 退役军人入驻 |
| GET | `/api/veteran/me` | 获取退役军人信息 |
| GET | `/api/veteran/orders/available` | 可接订单列表 |
| POST | `/api/veteran/orders/:id/accept` | 接受订单 |
| GET | `/api/veteran/orders/assigned` | 已分配订单 |
| GET | `/api/veteran/orders/:id/detail` | 订单详情 |
| GET | `/api/veteran/ws/orders` | WebSocket（订单实时推送） |

## 远程部署

### 部署脚本
- `scripts/deploy.py` — 核心部署脚本（Python），从 Windows 本机构建并部署到 Linux
- `deploy.bat` — Windows 批处理封装

### 部署流程
```
deploy.bat → deploy.py
  1. build_frontend()      npm run build
  2. build_backend()       Docker / Zig 交叉编译 Linux 二进制
  3. make_package()        打包二进制 + frontend/dist → tar.gz
  4. connect()             SSH 连接远程服务器
  5. install_remote()      上传解压 → /opt/gs-store-system/releases/<timestamp>/
  6. install_systemd()     写 gs-store-system.service
  7. install_nginx()       写 nginx 配置
  8. restart_and_verify()  systemctl restart → curl /api/health
  9. verify_public()       HTTPS 公开验证
```

### 部署目标
- 远程服务器：从 `.env` 读取 `DEPLOY_SERVER` / `DEPLOY_USER` / `DEPLOY_SSH_PASSWORD`
- 部署路径：`/opt/gs-store-system/releases/<timestamp>/`，软链到 `/opt/gs-store-system/current/`
- systemd 服务：`/etc/systemd/system/gs-store-system.service`
- Nginx 配置：`/etc/nginx/conf.d/gs-store-system.conf`（模板在 `deploy/nginx/gs-store-system.conf`）

### 生产环境链路
```
用户 → https://www.mcx59481.cn
         ↓
      Nginx (443 SSL, proxy_pass 127.0.0.1:9000)
         ↓
      Rust App (systemd: gs-store-system.service)
         ├── 静态文件: frontend/dist/
         ├── API: /api/*
         ├── MySQL: mcx-yl
         ├── Redis
         └── Aliyun OSS
```

## Windows 机器说明

- Rust 通过 `rustup` 安装，MSVC 构建工具已就绪，`cargo check` / `cargo run` 可直接使用。
- 下载工具链或包时如有网络问题，使用本地代理 `127.0.0.1:7890`。

## 安全注意事项

- 不要将 `.env` 中的密钥打印或复制到 Git 跟踪文件中。
- 不要盲目覆盖远程 Nginx 配置，先检查已有域名块。
- 修改域名路由时保持 HTTPS 证书路径和 TLS 配置不变。

## 相关文档

- `DESIGN.md` — 侠到家 H5 UI 设计规范
- `README.md` — 面向人类的项目介绍
- `kb/product-plan.md` — 产品计划书
- `kb/technical-solution.md` — 技术方案
- `kb/official-website-mvp-plan.md` — 官网 MVP 规划
