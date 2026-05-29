# 国膳甄选管理后台开发计划

## 目标

基于 `backend/ADMIN_PRODUCT_DESIGN.md` 的产品设计，逐步完成国膳甄选管理后台。后台前端代码统一放在 `backend/` 目录，后端接口使用 `gs-store-system` 中已有 `/auth/*` 与 `/api/admin/*` 接口。

## Skill 使用约定

| 阶段 | 使用 Skill | 目的 | 主要产出 |
|---|---|---|---|
| 产品拆解 | `product-manager` | 将后端接口和业务目标转成页面、用户任务、优先级 | 页面清单、功能范围、验收标准 |
| UI/UX 设计 | `ui-ux-pro-max` | 定义后台信息架构、视觉规范、交互规则、可访问性要求 | 页面布局、组件规范、状态设计 |
| 前端开发 | `frontend-developer` | 使用 Vue 3、Arco Design、Pinia、Vue Router 实现页面 | 可运行前端页面、接口调用、状态管理 |
| 后端联调 | `backend-developer` | 校验接口契约、鉴权、错误处理、数据库/Redis 环境 | 联调记录、接口修正建议、启动配置 |
| QA 验证 | `qa-engineer` | 验证核心流程、异常状态、权限控制、回归测试点 | 测试清单、问题列表、验收结果 |

## 开发流程

每个页面模块按以下流程推进：

1. **接口确认**
   - 使用 `backend-developer` 检查对应 Rust handler、请求参数、响应字段、权限码。
   - 输出页面所需 API 清单和字段映射。

2. **产品范围确认**
   - 使用 `product-manager` 明确该页面的用户任务、主流程、边界场景和非目标。
   - 输出页面功能点和验收标准。

3. **UI/UX 设计**
   - 使用 `ui-ux-pro-max` 设计页面结构、表格字段、筛选区、表单、弹窗、空状态、错误状态。
   - 保证表单 label、按钮状态、确认弹窗、移动端可用性和对比度。

4. **前端实现**
   - 使用 `frontend-developer` 编写 Vue 页面、组件、API service、Pinia 状态和路由守卫。
   - 所有页面代码放在 `backend/src/`。

5. **联调验证**
   - 使用 `backend-developer` 启动 `gs-store-system`，通过 Vite 代理联调。
   - 验证成功、失败、无权限、空数据、加载中等状态。

6. **QA 回归**
   - 使用 `qa-engineer` 整理测试点并执行构建、关键接口和页面操作验证。

## 技术约定

| 类型 | 约定 |
|---|---|
| 前端框架 | Vue 3 + Vite |
| UI 组件 | Arco Design Vue |
| 状态管理 | Pinia |
| 路由 | Vue Router |
| 请求库 | Axios |
| API 代理 | Vite dev server 代理 `/auth`、`/api` 到 `127.0.0.1:8081` |
| 后端服务 | `gs-store-system` Rust API |
| 本地后端启动 | `cd gs-store-system && cargo run` |
| 本地前端启动 | `cd backend && npm run dev` |
| 默认前端地址 | `http://127.0.0.1:8080/login` |

## 本地后端配置

本地开发使用远程 MySQL 和 Redis。`gs-store-system/.env` 应使用 `config.rs` 实际读取的拆分变量：

```env
DATABASE_HOST=47.103.220.84
DATABASE_PORT=3306
DATABASE_NAME=welfare_store
DATABASE_USER=root
DATABASE_PASSWORD=******

REDIS_HOST=47.103.220.84
REDIS_PORT=6379
REDIS_DB=0
REDIS_USERNAME=default
REDIS_PASSWORD=******

SERVER_PORT=8081
SKIP_MIGRATIONS=true
```

说明：

- `config.rs` 当前不读取 `DATABASE_URL` 和 `REDIS_URL`。
- 密码写在拆分变量中不需要手动 URL 编码，代码会自动编码。
- `SKIP_MIGRATIONS=true` 仅用于本地联调远程库，避免远程库已应用 migration 与本地文件 checksum 不一致导致启动失败。

## Cargo Feature 约定

为避免本地管理后台开发被 OCR 重依赖阻塞：

| 场景 | 命令 | 说明 |
|---|---|---|
| 本地开发 | `cargo run` | 默认不编译 `ddddocr/ort-sys`，JK warmup 可失败但服务继续启动 |
| 生产构建 | `cargo build --release --features jk-ocr` | 启用 JK OCR 能力 |
| 生产运行 | 使用带 `jk-ocr` feature 的 release 二进制 | 保持 JK Pay 验证码识别能力 |

后续如果调整部署脚本，应优先修改 `deploy.ps1`，并确保后端 release 构建包含 `--features jk-ocr`。

## 里程碑计划

### M0：基础工程与登录

使用 Skill：`ui-ux-pro-max`、`frontend-developer`、`backend-developer`

范围：

- 登录页
- Vite API 代理
- Token 保存与请求拦截
- `/auth/login`、`/auth/codes` 联调
- 后台基础布局和中文文案修复

验收：

- 访问 `/login` 无乱码。
- 使用管理员账号可登录。
- 登录后可进入首页布局。
- 刷新页面能保持登录态。

### M1：首页概览

使用 Skill：`product-manager`、`ui-ux-pro-max`、`frontend-developer`、`backend-developer`

范围：

- 经营指标卡
- 最新订单表格
- 时间筛选
- 收入金额格式化
- 加载、错误、空数据状态

接口：

- `GET /api/admin/dashboard`

验收：

- 首页可展示订单、收入、用户、商品等核心指标。
- 接口异常时显示可理解的错误提示。

### M2：商品与分类

使用 Skill：`product-manager`、`ui-ux-pro-max`、`frontend-developer`、`backend-developer`、`qa-engineer`

范围：

- 分类列表、新建、编辑、删除
- 商品列表、新建、编辑、删除
- OSS 上传签名接入
- 状态、排序、图片预览

接口：

- `GET/POST /api/admin/categories`
- `PUT/DELETE /api/admin/categories/{id}`
- `GET/POST /api/admin/products`
- `GET/PUT/DELETE /api/admin/products/{id}`
- `GET /api/admin/upload/signature`

验收：

- 商品和分类 CRUD 可完成。
- 删除类操作有二次确认。
- 上传失败有明确反馈。

### M3：权益商品

使用 Skill：`product-manager`、`ui-ux-pro-max`、`frontend-developer`、`backend-developer`、`qa-engineer`

范围：

- 权益商品列表
- 权益商品详情/编辑
- 规格组和 SKU 表格
- 库存、价格、状态管理

接口：

- `GET/POST /api/admin/goods`
- `GET/PUT/DELETE /api/admin/goods/{id}`

验收：

- SKU 引用规格值前端可校验。
- 低库存有文字标签提示。
- 保存失败可定位到具体字段或业务原因。

### M4：订单管理

使用 Skill：`product-manager`、`ui-ux-pro-max`、`frontend-developer`、`backend-developer`、`qa-engineer`

范围：

- 订单列表
- 状态筛选
- 订单详情
- 订单状态更新
- 金额、用户、商品摘要展示

接口：

- `GET /api/admin/orders`
- `GET /api/admin/orders/{id}`
- `PUT /api/admin/orders/{id}/status`

验收：

- 可按状态、订单号、用户筛选。
- 状态变更有确认和结果反馈。

### M5：用户、余额与订阅

使用 Skill：`product-manager`、`ui-ux-pro-max`、`frontend-developer`、`backend-developer`、`qa-engineer`

范围：

- 用户列表、详情、编辑
- 身份证重复校验
- 支付密码查看权限控制
- 用户余额和流水
- 自动充值
- 订阅记录

接口：

- `GET/POST /api/admin/wechat/users`
- `GET/PUT/DELETE /api/admin/wechat/users/{id}`
- `PUT /api/admin/wechat/users/by-openid/{openid}`
- `POST /api/admin/wechat/users/check-id-card`
- `GET /api/admin/wechat/users/{openid}/payment-password`
- `GET /api/admin/wechat/users/{openid}/balance`
- `GET /api/admin/wechat/users/{openid}/balance/transactions`
- `POST /api/admin/subscription/auto-recharge`
- `GET /api/admin/subscription/records`

验收：

- 敏感字段默认隐藏。
- 无权限时隐藏对应按钮。
- 自动充值有二次确认。

### M6：系统管理

使用 Skill：`product-manager`、`ui-ux-pro-max`、`frontend-developer`、`backend-developer`、`qa-engineer`

范围：

- 管理员列表
- 权限目录
- 角色与权限编辑
- 日志中心

接口：

- `GET /api/admin/permissions`
- `GET /api/admin/admin-users`
- `PUT /api/admin/admin-users/{id}/permissions`
- `GET /api/admin/logs/recent`

验收：

- 权限按分组展示。
- 当前账号关键权限变更有保护提示。
- 日志中心支持刷新、日志类型、行数控制。

## 通用验收标准

- 所有中文文案正常显示，无乱码和问号占位。
- 所有表单字段有可见 label。
- 所有请求有 loading、success、error 状态。
- 删除、充值、状态变更、查看敏感信息有确认机制。
- 页面遵循权限码控制菜单和按钮显隐。
- `npm run build` 通过。
- 关键接口通过 Vite 代理完成联调。
