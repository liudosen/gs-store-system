# 管理后台逐页联调任务清单

执行顺序固定为：UI 设计 -> Arco 组件选型 -> 后端接口联调 -> 空/错/加载态验收。

## 全局任务

| 状态 | 页面/模块 | UI 设计 | Arco 组件 | 后端联调 | 验收点 |
|---|---|---|---|---|---|
| 进行中 | 后台框架 | 去掉顶部重复“管理后台”标题，改为轻量上下文栏；左侧导航强化层级 | Layout、Menu、Dropdown、Tag、Button | `/auth/codes` 保持登录态 | 刷新不丢登录，导航不遮挡内容 |
| 待办 | 权限控制 | 按权限码隐藏不可用入口和危险操作 | Menu、Tooltip、Button | `/auth/codes` | 无权限不显示敏感按钮 |
| 待办 | 统一数据状态 | 每页提供加载、空数据、错误重试、保存中状态 | Spin、Empty、Result、Skeleton | 所有 `/api/admin/*` | 接口为空也能判断是“无数据”不是“没联调” |
| 待办 | 统一表单体验 | 抽屉/弹窗分组、必填校验、危险操作二次确认 | Modal、Drawer、Form、Popconfirm | CRUD 接口 | 表单失败可定位字段 |

## 页面任务

| 优先级 | 页面 | UI 设计任务 | 组件库选型 | 后端接口联调 | 当前问题/验收 |
|---|---|---|---|---|---|
| P0 | 登录页 | 保持现有高质感登录页，补充错误状态和快捷调试信息 | Form、Input、Button、Alert | `POST /auth/login`、`GET /auth/codes` | 登录成功后刷新保持登录态 |
| P0 | 首页概览 | 指标卡片、最新订单、时间筛选、趋势占位 | Card、Statistic、Table、DatePicker | `GET /api/admin/dashboard` | 有数据展示金额/订单/用户；无数据展示空态 |
| P0 | 分类管理 | 简洁列表 + 新建编辑弹窗；突出排序、状态、商品数 | Card、Table、Tag、Modal、Form | `GET/POST /api/admin/categories`、`PUT/DELETE /api/admin/categories/{id}` | 分类列表不能只看到空白，删除在有关联商品时提示后端错误 |
| P0 | 商品管理 | 商品缩略图、价格、分类、上下架状态；支持关键词本地筛选 | Table、Avatar、Image、Tag、Modal、InputNumber | `GET/POST /api/admin/products`、`GET/PUT/DELETE /api/admin/products/{id}` | 图片字段、价格分转换正确 |
| P0 | 权益商品 | SPU 卡片化表格、库存风险、价格与上下架；编辑表单分区 | Table、Tag、Drawer/Modal、Form、InputNumber | `GET/POST /api/admin/goods`、`GET/PUT/DELETE /api/admin/goods/{id}` | SKU/库存保存与列表刷新正常 |
| P0 | 订单管理 | 筛选条、状态标签、详情抽屉、状态变更菜单 | Table、Select、Drawer、Descriptions、Dropdown | `GET /api/admin/orders`、`GET /api/admin/orders/{id}`、`PUT /api/admin/orders/{id}/status` | 订单分页、详情、改状态均有反馈 |
| P1 | 用户管理 | 头像+实名信息、敏感字段遮罩、余额抽屉 | Table、Avatar、Tag、Drawer、Descriptions | `GET/POST /api/admin/wechat/users`、`GET/PUT/DELETE /api/admin/wechat/users/{id}`、余额/流水接口 | 身份证/支付密码默认不裸露 |
| P1 | 订阅与充值 | 订阅记录表 + 自动充值动作区 + 结果摘要 | Table、Statistic、Modal、Alert | `GET /api/admin/subscription/records`、`POST /api/admin/subscription/auto-recharge` | 自动充值必须二次确认并展示结果 |
| P1 | 权限管理 | 管理员列表、权限分组勾选、保存反馈 | Table、Checkbox、Modal、Tabs | `GET /api/admin/admin-users`、`GET /api/admin/permissions`、`PUT /api/admin/admin-users/{id}/permissions` | 权限目录为空时显示说明 |
| P2 | 日志/运维 | 最近日志查看、类型筛选、自动刷新 | Select、Button、Textarea/Code block | `GET /api/admin/logs/recent` | 可用于排查联调失败原因 |
| P2 | 上传 | 商品/权益商品图片从 URL 输入升级为 OSS 直传 | Upload、Image、Progress | `GET /api/admin/upload/signature` | 上传成功回填 URL，失败明确提示 |

## 本轮执行记录

- 使用 Arco Design Vue 作为字节跳动组件库基础，不引入额外 UI 框架。
- 后台框架先去掉顶部重复大标题，保留当前页面上下文和刷新/用户操作。
- 所有页面继续使用现有 Rust `/api/admin/*` 接口，优先修复“看不到数据时无反馈”的体验。
