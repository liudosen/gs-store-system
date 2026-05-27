-- ============================================
--  Mock 测试数据 — 侠伴行订单
--  运行方式: mysql -u root -p mcx-yl < mock_orders.sql
-- ============================================

-- 1. 确保有测试用户 (customer)
INSERT IGNORE INTO customer_users (id, phone, region_code, region_name, created_at, updated_at)
VALUES
  (1, '13800001111', 'sh-pudong', 'Pudong', NOW(), NOW()),
  (2, '13800002222', 'sh-pudong', 'Pudong', NOW(), NOW()),
  (3, '13800003333', 'sh-minhang', 'Minhang', NOW(), NOW());

-- 2. 确保有测试地址
INSERT IGNORE INTO customer_addresses (id, user_id, region_code, region_name, city_name, district_name, detail_address, contact_name, contact_phone, is_default, created_at, updated_at)
VALUES
  (1, 1, 'sh-pudong', 'Pudong', '上海市', '浦东新区', '张江镇张衡路100号', '张阿姨', '13900001111', 1, NOW(), NOW()),
  (2, 2, 'sh-pudong', 'Pudong', '上海市', '浦东新区', '陆家嘴环路200号', '李先生', '13900002222', 1, NOW(), NOW()),
  (3, 3, 'sh-minhang', 'Minhang', '上海市', '闵行区', '七宝镇七莘路300号', '王叔叔', '13900003333', 1, NOW(), NOW());

-- 3. 确保有测试退役军人 (veteran)
INSERT IGNORE INTO veteran_profiles (id, name, id_number, phone, veteran_card_number, region_code, region_name, service_status, is_dispatch_ready, rating_score, completed_order_count, service_tags, created_at, updated_at)
VALUES
  (1, '张班长', '310000199001011234', '13810001111', 'VET20260001', 'sh-pudong', 'Pudong', 'available', 1, 4.80, 5, 'escort-medical,home-companion,home-cleaning,meal-delivery', NOW(), NOW());

-- 4. 确保有服务项目
INSERT IGNORE INTO service_items (id, code, category_name, name, short_description, badge, base_price, duration_minutes)
VALUES
  (1, 'escort-medical', '陪诊', '陪诊就医服务', '协助挂号、排队、取药、就医陪同', '热', '200', 120),
  (2, 'home-companion', '陪伴', '居家陪伴聊天', '陪伴老人聊天、散步、读报', '', '150', 60),
  (3, 'home-cleaning', '家政', '家政清洁服务', '居家日常清洁、收纳整理', '', '180', 120),
  (4, 'meal-delivery', '送餐', '送餐到家服务', '热餐配送上门', '', '120', 30);

-- 5. 插入测试订单（不同状态）
INSERT INTO service_orders (id, user_id, order_no, service_item_id, service_item_name, region_code, region_name, city_name, district_name, address_id, service_address, contact_name, contact_phone, service_date, service_time_slot, note, status, status_label, assigned_veteran_id, assigned_veteran_name, assigned_veteran_phone, dispatch_message, created_at)
VALUES
  -- 进行中订单 (assigned) — 新接的
  (1001, 1, 'MCK20260523001', 1, '陪诊就医服务', 'sh-pudong', 'Pudong', '上海市', '浦东新区', 1, '上海市浦东新区张江镇张衡路100号', '张阿姨', '13900001111', '2026-05-24', '上午 9:00-12:00', '需要轮椅协助', 'assigned', '已接单', 1, '张班长', '13810001111', '已分配服务者，等待服务开始', NOW()),
  
  -- 进行中订单 (in_progress) — 正在服务
  (1002, 2, 'MCK20260523002', 2, '居家陪伴聊天', 'sh-pudong', 'Pudong', '上海市', '浦东新区', 2, '上海市浦东新区陆家嘴环路200号', '李先生', '13900002222', '2026-05-23', '下午 14:00-16:00', '需要会下棋的服务者', 'in_progress', '服务中', 1, '张班长', '13810001111', '服务进行中', NOW()),
  
  -- 已完成订单 (completed) ×3 — 历史
  (1003, 1, 'MCK20260520003', 3, '家政清洁服务', 'sh-pudong', 'Pudong', '上海市', '浦东新区', 1, '上海市浦东新区张江镇张衡路100号', '张阿姨', '13900001111', '2026-05-20', '上午 8:00-11:00', '', 'completed', '已完成', 1, '张班长', '13810001111', '服务已完成', '2026-05-20 10:00:00'),
  (1004, 3, 'MCK20260518004', 4, '送餐到家服务', 'sh-minhang', 'Minhang', '上海市', '闵行区', 3, '上海市闵行区七宝镇七莘路300号', '王叔叔', '13900003333', '2026-05-18', '中午 11:30', '少盐', 'completed', '已完成', 1, '张班长', '13810001111', '服务已完成', '2026-05-18 11:30:00'),
  (1005, 1, 'MCK20260515005', 1, '陪诊就医服务', 'sh-pudong', 'Pudong', '上海市', '浦东新区', 1, '上海市浦东新区张江镇张衡路100号', '张阿姨', '13900001111', '2026-05-15', '上午 9:00-12:00', '需要帮忙取报告', 'completed', '已完成', 1, '张班长', '13810001111', '服务已完成', '2026-05-15 10:00:00'),
  
  -- 待接订单 (matching) — 首页可看到
  (1006, 2, 'MCK20260523006', 1, '陪诊就医服务', 'sh-pudong', 'Pudong', '上海市', '浦东新区', 2, '上海市浦东新区陆家嘴环路200号', '李先生', '13900002222', '2026-05-25', '上午 9:00-12:00', '', 'matching', '待接单', NULL, NULL, NULL, '订单已创建，等待服务者接单', NOW()),
  (1007, 2, 'MCK20260523007', 4, '送餐到家服务', 'sh-pudong', 'Pudong', '上海市', '浦东新区', 2, '上海市浦东新区陆家嘴环路200号', '李先生', '13900002222', '2026-05-24', '中午 11:30', '', 'matching', '待接单', NULL, NULL, NULL, '订单已创建，等待服务者接单', NOW()),
  (1008, 3, 'MCK20260523008', 3, '家政清洁服务', 'sh-minhang', 'Minhang', '上海市', '闵行区', 3, '上海市闵行区七宝镇七莘路300号', '王叔叔', '13900003333', '2026-05-24', '下午 14:00-17:00', '重点打扫厨房', 'matching', '待接单', NULL, NULL, NULL, '订单已创建，等待服务者接单', NOW());

-- 6. 插入状态日志
INSERT INTO order_status_logs (order_id, status, message, created_at) VALUES
  (1001, 'matching', '订单已创建，等待服务者接单', NOW()),
  (1001, 'assigned', '已分配服务者：张班长', NOW()),
  (1002, 'matching', '订单已创建，等待服务者接单', NOW()),
  (1002, 'assigned', '已分配服务者：张班长', NOW()),
  (1002, 'in_progress', '服务开始', NOW()),
  (1003, 'matching', '订单已创建', '2026-05-20 08:00:00'),
  (1003, 'assigned', '已分配服务者：张班长', '2026-05-20 08:05:00'),
  (1003, 'completed', '服务已完成', '2026-05-20 10:00:00'),
  (1004, 'completed', '服务已完成', '2026-05-18 11:30:00'),
  (1005, 'completed', '服务已完成', '2026-05-15 10:00:00'),
  (1006, 'matching', '订单已创建，等待服务者接单', NOW()),
  (1007, 'matching', '订单已创建，等待服务者接单', NOW()),
  (1008, 'matching', '订单已创建，等待服务者接单', NOW());

SELECT 'Mock data inserted!' AS result;
SELECT 'Assigned orders:' AS label, COUNT(*) AS count FROM service_orders WHERE assigned_veteran_id = 1;
SELECT 'Matching orders (available):' AS label, COUNT(*) AS count FROM service_orders WHERE status = 'matching';
