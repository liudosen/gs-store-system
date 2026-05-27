# 迷彩侠（MCX）—— 军旅实业合作平台

> 上海迷彩侠实业有限公司 | 以退役军人为核心，连接就业与社会服务价值的平台

---

## 目录

- [项目简介](#项目简介)
- [品牌与产品](#品牌与产品)
  - [迷彩侠（品牌平台）](#迷彩侠品牌平台)
  - [侠到家（家庭服务端）](#侠到家家庭服务端)
  - [侠伴行（退役军人端）](#侠伴行退役军人端)
- [项目架构](#项目架构)
  - [技术栈](#技术栈)
  - [目录结构](#目录结构)
  - [后端分层](#后端分层)
  - [前端分层](#前端分层)
- [入口说明](#入口说明)
  - [后端入口](#后端入口)
  - [前端入口](#前端入口)
- [后端配置读取机制](#后端配置读取机制)
- [本地开发](#本地开发)
- [生产构建](#生产构建)
- [远程部署](#远程部署)
- [API 概览](#api-概览)
- [部署架构](#部署架构)

---

## 项目简介

本项目是一个**前后端耦合的全栈应用**，为上海迷彩侠实业有限公司构建品牌官网及业务平台。平台以退役军人群体为核心，连接就业与社会服务价值——当前 MVP 阶段以养老上门服务为主要落地场景，后续可扩展至更多由退役军人承接的社会服务。

- **后端**：Rust + Axum（Web 框架）
- **前端**：Vue 3 + Vite（移动端优先的 H5 应用）
- **数据库**：MySQL（业务数据）+ Redis（缓存 / 验证码 / 会话）
- **存储**：阿里云 OSS（图片与资质文件）
- **部署**：Nginx 反向代理 → Rust 单体应用（systemd 管理）

---

## 品牌与产品

### 迷彩侠（品牌平台）

**迷彩侠（MCX）** 是上海迷彩侠实业有限公司的品牌平台，定位为"军旅实业合作平台"。核心理念：

- 帮助退役军人就业、组织退役军人服务社会
- 围绕创就业、供应链、康养服务、军民融合协同发展
- 养老到家服务是当前落地场景之一，但不是品牌边界的全部

官网首页（`/`）承担品牌介绍、信任建立、社会价值表达，并提供"侠到家"和"侠伴行"两个子产品的入口。

### 侠到家（家庭服务端）

**侠到家** 是面向**老人及其家庭**的 H5 移动端服务平台。

- 路由前缀：`/xia-dao-jia/*`
- 核心功能：陪诊、陪伴聊天、家政清洁、送餐到家、代买代办等上门服务
- 用户流程：手机号短信登录 → 选择地区 → 浏览服务 → 填写时间/地址 → 提交预约 → 追踪订单
- 设计语言：可信、稳定、清晰、克制——更像一个真实的 Chinese service app

| 路由 | 页面 |
|---|---|
| `/xia-dao-jia/login` | 登录页（短信验证码登录） |
| `/xia-dao-jia/home` | 首页（地区选择、服务预览、订单进度） |
| `/xia-dao-jia/services` | 服务列表（按地区浏览、预约） |
| `/xia-dao-jia/addresses` | 地址管理 |
| `/xia-dao-jia/orders` | 订单列表 |
| `/xia-dao-jia/profile` | 个人中心 |

### 侠伴行（退役军人端）

**侠伴行** 是面向**退役军人服务者**的移动端工作平台。

- 路由：`/veteran-portal`（登录后进入工作台）
- 核心功能：入驻申请 → 实名认证 → 接单/拒单 → 服务执行 → 收入结算
- 底部导航：首页（工作台）、订单、培训、我的

| 路由 | 页面 |
|---|---|
| `/veteran-join` | 退役军人入驻申请页 |
| `/veteran-portal` | 退役军人工作台（登录后） |

---

## 项目架构

### 技术栈

| 层 | 技术 | 说明 |
|---|---|---|
| 后端运行时 | Rust (edition 2021) + Tokio | 异步运行时 |
| Web 框架 | Axum 0.7 | 路由、中间件、WebSocket |
| 数据库 | SQLx 0.8 (MySQL) | 异步连接池、编译期 SQL 检查 |
| 缓存 | Redis (redis-rs 0.27) | 验证码、会话、实时通知 |
| 对象存储 | OpenDAL (Aliyun OSS) | 图片、资质文件上传 |
| 短信 | 阿里云 SMS（自封装） | 验证码下发 |
| 日志 | Tracing + tracing-appender | 控制台 + 按天滚动文件 |
| 前端框架 | Vue 3 (Composition API) | SPA |
| 构建工具 | Vite 5 | 开发服务器 + 生产打包 |
| UI 组件库 | Vant 4 | 移动端组件 |

### 目录结构

```
gs-store-system/
├── src/                          # Rust 后端源码
│   ├── main.rs                   # 程序入口（tokio::main）
│   ├── app.rs                    # 启动引导（配置、DB、路由、日志）
│   ├── routes.rs                 # API 路由定义
│   ├── common/                   # 通用层
│   │   ├── api.rs                # 通用 API 类型（如 Health）
│   │   ├── errors.rs             # 错误类型
│   │   └── phone.rs              # 手机号校验
│   ├── domains/                  # 业务领域层
│   │   ├── auth/                 # 退役军人登录认证
│   │   ├── catalog/              # 服务目录
│   │   ├── customer/             # 用户端（登录、地址、个人信息）
│   │   ├── onboarding/           # 退役军人入驻
│   │   ├── order/                # 订单（创建、匹配、接单、WebSocket 推送）
│   │   └── veteran/              # 退役军人信息
│   └── infra/                    # 基础设施层
│       ├── config.rs             # 环境变量配置读取
│       ├── sms.rs                # 阿里云短信服务封装
│       └── state.rs              # 全局 AppState
├── frontend/                     # Vue 3 前端源码
│   ├── index.html                # HTML 入口
│   ├── vite.config.js            # Vite 配置（含 /api 代理）
│   ├── package.json
│   └── src/
│       ├── main.js               # Vue 应用入口
│       ├── App.vue               # 根组件（路由分发）
│       ├── styles.css            # 全局样式 + CSS 变量
│       ├── app/                  # 应用级路由与元信息
│       │   ├── routes.js         # 路由映射表
│       │   ├── useAppRouting.js  # 前端路由（基于 pushState）
│       │   └── usePageMeta.js    # 页面 title/description 管理
│       ├── pages/                # 页面组件
│       │   ├── HomePage.vue      # 官网首页（/）
│       │   ├── VeteranJoinPage.vue       # 退役军人入驻（/veteran-join）
│       │   ├── VeteranPortalPage.vue     # 退役军人工作台（/veteran-portal）
│       │   ├── CustomerLoginPage.vue     # 侠到家登录（/xia-dao-jia/login）
│       │   ├── CustomerHomePage.vue      # 侠到家首页
│       │   ├── CustomerServicesPage.vue  # 侠到家服务列表
│       │   ├── CustomerOrdersPage.vue    # 侠到家订单
│       │   ├── CustomerAddressesPage.vue # 侠到家地址管理
│       │   └── CustomerProfilePage.vue   # 侠到家个人中心
│       ├── features/             # 业务特性模块（composables）
│       │   ├── customer/         # 用户端逻辑
│       │   ├── onboarding/       # 入驻逻辑
│       │   └── portal/           # 退役军人门户逻辑
│       ├── components/           # 共享 UI 组件
│       │   ├── customer/         # 侠到家组件（AppShell 等）
│       │   └── veteran/          # 侠伴行组件（Workbench 等）
│       ├── shared/               # 共享工具
│       │   ├── api/              # API 请求封装
│       │   └── constants/        # 常量（localStorage key 等）
│       ├── content/              # 官网内容数据
│       └── assets/               # 图片等静态资源
├── deploy/                       # 部署配置
│   └── nginx/
│       └── gs-store-system.conf  # Nginx HTTPS 反向代理配置
├── scripts/
│   └── deploy.py                 # 远程部署脚本（Python）
├── deploy.bat                    # Windows 部署启动脚本
├── kb/                           # 知识库（产品计划、技术方案等）
├── docs/                         # 文档
├── logs/                         # 日志输出目录（滚动日志）
├── .env                          # 本地环境变量（Git 忽略）
├── .env.example                  # 环境变量模板
├── Cargo.toml                    # Rust 依赖配置
├── DESIGN.md                     # 侠到家 UI 设计规范
└── CLAUDE.md                     # AI 辅助开发规则
```

### 后端分层

后端采用**单体分层架构**（MVP 阶段不做微服务拆分）：

```
┌─────────────────────────────────────────┐
│  routes.rs    API 路由注册              │
├─────────────────────────────────────────┤
│  domains/     业务领域逻辑              │
│  ├── auth         退役军人认证          │
│  ├── catalog      服务目录              │
│  ├── customer     用户端业务            │
│  ├── onboarding   退役军人入驻          │
│  ├── order        订单 + WebSocket      │
│  └── veteran      退役军人信息          │
├─────────────────────────────────────────┤
│  common/      通用工具                  │
│  ├── api.rs      通用响应类型           │
│  ├── errors.rs   错误处理               │
│  └── phone.rs    手机号校验             │
├─────────────────────────────────────────┤
│  infra/       基础设施                  │
│  ├── config.rs   环境变量配置           │
│  ├── sms.rs      阿里云短信             │
│  └── state.rs    全局 AppState          │
└─────────────────────────────────────────┘
```

每个 domain 内部通常包含：
- `mod.rs` — 模块声明
- `handler.rs` — HTTP 请求处理函数
- `model.rs` — 数据结构 / 请求响应体
- `repository.rs` — 数据库访问层

### 前端分层

前端使用**基于 pushState 的手动路由**（非 vue-router），按路径匹配页面组件：

```
App.vue  ───  根据 currentPath 动态切换页面
  │
  ├── /                       → HomePage.vue          (迷宫侠官网)
  ├── /veteran-join           → VeteranJoinPage.vue   (入驻)
  ├── /veteran-portal         → VeteranPortalPage.vue (侠伴行)
  └── /xia-dao-jia/*          → Customer*Page.vue     (侠到家)
```

---

## 入口说明

### 后端入口

| 项 | 路径 | 说明 |
|---|---|---|
| 程序入口 | `src/main.rs` | `#[tokio::main]` 异步入口，调用 `app::bootstrap()` |
| 启动引导 | `src/app.rs` → `bootstrap()` | 加载 `.env` → 初始化日志 → 读取配置 → 连接 MySQL/Redis → 建表 → 构建路由 → 绑定端口启动 |
| 路由注册 | `src/routes.rs` → `api_router()` | 所有 `/api/*` 路由在此集中注册 |
| 默认端口 | `9000` | 由 `APP_PORT` 环境变量控制 |

> `bootstrap()` 还启动了日志清理定时任务：每天检查 `logs/` 目录，自动删除 30 天前的旧日志文件。

### 前端入口

| 项 | 路径 | 说明 |
|---|---|---|
| HTML 入口 | `frontend/index.html` | Vite 构建的 HTML 模板，`<div id="app">` |
| JS 入口 | `frontend/src/main.js` | `createApp(App).mount('#app')` |
| 根组件 | `frontend/src/App.vue` | 路由分发、页面切换、meta 管理 |
| 路由配置 | `frontend/src/app/routes.js` | 路径→页面组件映射表 |
| Vite 配置 | `frontend/vite.config.js` | 开发服务器 `127.0.0.1:5173`，`/api` 代理到 `9000` |

---

## 后端配置读取机制

后端配置通过**环境变量**读取，加载流程如下：

```
1. dotenvy::dotenv()
   └── 从项目根目录的 .env 文件加载环境变量到进程

2. AppConfig::from_env()
   └── src/infra/config.rs
       读取以下环境变量：
```

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

> **注意**：远程部署时，systemd 通过 `EnvironmentFile=/opt/gs-store-system/.env` 加载环境变量，与本地开发使用同一份 `.env` 文件结构。`.env` 被 Git 忽略，不提交到仓库。

---

## 本地开发

### 1. 安装前端依赖

```powershell
cd frontend
npm install
```

### 2. 启动前端开发服务器

```powershell
npm run dev
# → http://127.0.0.1:5173
```

### 3. 启动后端

```powershell
# 在项目根目录
cargo run
# → http://127.0.0.1:9000
```

> 本地开发时，Vite 将 `/api/*` 请求代理到 `http://127.0.0.1:9000`，前端可直接使用同源路径。

---

## 生产构建

```powershell
# 构建前端
cd frontend
npm run build
# 产物在 frontend/dist/

# 构建后端
cd ..
cargo build --release
```

生产模式下，Rust 服务器自动从 `frontend/dist/` 提供静态文件，优先使用 `frontend/dist/index.html`，找不到时回退到 `frontend/`。

---

## 远程部署

### 部署脚本

| 文件 | 说明 |
|---|---|
| `scripts/deploy.py` | **核心部署脚本**，支持从 Windows 本机构建并部署到远程 Linux 服务器 |
| `deploy.bat` | Windows 批处理封装，设置 MSVC 环境变量和代理后调用 `deploy.py` |

### 部署流程

```
deploy.bat  →  deploy.py
                   ├── 1. build_frontend()          npm run build
                   ├── 2. build_backend()           Docker / Zig 交叉编译 Linux 二进制
                   ├── 3. make_package()            打包二进制 + frontend/dist → tar.gz
                   ├── 4. connect()                 SSH 连接远程服务器
                   ├── 5. install_remote()          上传解压 → /opt/gs-store-system/releases/<timestamp>/
                   ├── 6. install_systemd()         写 gs-store-system.service → systemctl daemon-reload
                   ├── 7. install_nginx()           写 nginx 配置 → nginx -t
                   ├── 8. restart_and_verify()      systemctl restart → curl /api/health
                   └── 9. verify_public()           HTTPS 公开验证
```

### 部署目标

- **远程服务器**：从 `.env` 中读取 `DEPLOY_SERVER` / `DEPLOY_USER` / `DEPLOY_SSH_PASSWORD`
- **部署路径**：`/opt/gs-store-system/releases/<timestamp>/` → 软链到 `/opt/gs-store-system/current/`
- **systemd 服务**：`/etc/systemd/system/gs-store-system.service`
- **Nginx 配置**：`/etc/nginx/conf.d/gs-store-system.conf`

### 部署命令

```powershell
# Windows 上执行部署（需先安装 Docker Desktop 或 MSVC + Zig）
.\deploy.bat

# 或直接调用 Python 脚本
python scripts\deploy.py

# 跳过某些步骤
python scripts\deploy.py --skip-build --skip-nginx --skip-public-verify
```

### 生产环境请求链路

```
用户 → https://www.mcx59481.cn
         │
         ▼
      Nginx (443 SSL)
         │  proxy_pass http://127.0.0.1:9000
         ▼
      Rust 应用 (systemd: gs-store-system.service)
         │
         ├── 静态文件: frontend/dist/
         ├── API:      /api/*
         ├── MySQL:    mcx-yl
         ├── Redis:    缓存/会话/验证码
         └── OSS:      阿里云对象存储
```

---

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

---

## 部署架构

```
┌──────────────────────────────────────────────────┐
│                  互联网                           │
│         https://www.mcx59481.cn                   │
└──────────────────┬───────────────────────────────┘
                   │
        ┌──────────▼──────────┐
        │   Nginx (:443)      │
        │   TLS 终止           │
        │   proxy_pass :9000  │
        └──────────┬──────────┘
                   │
        ┌──────────▼──────────┐
        │  Rust App (:9000)   │
        │  systemd 管理       │
│  /opt/gs-store-system/current/  │
        └──┬──────┬──────┬───┘
           │      │      │
    ┌──────▼┐ ┌──▼──┐ ┌─▼──────┐
    │ MySQL │ │Redis│ │Ali OSS │
    │mcx-yl │ │     │ │        │
    └───────┘ └─────┘ └────────┘
```

> 更多设计细节参见：
> - `DESIGN.md` — 侠到家 H5 UI 设计规范
> - `kb/product-plan.md` — 产品计划书
> - `kb/technical-solution.md` — 技术方案
> - `kb/official-website-mvp-plan.md` — 官网 MVP 规划
> - `CLAUDE.md` — AI 辅助开发规则
