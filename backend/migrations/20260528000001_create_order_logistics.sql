CREATE TABLE IF NOT EXISTS order_logistics (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    order_id BIGINT UNSIGNED NOT NULL,
    order_no VARCHAR(64) NOT NULL,
    carrier VARCHAR(100) NOT NULL DEFAULT '',
    tracking_no VARCHAR(128) NOT NULL DEFAULT '',
    delivery_name VARCHAR(100) NOT NULL DEFAULT '',
    delivery_phone VARCHAR(32) NOT NULL DEFAULT '',
    remark VARCHAR(500) NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_order_logistics_order_id (order_id),
    KEY idx_order_logistics_order_no (order_no),
    KEY idx_order_logistics_tracking_no (tracking_no),
    KEY idx_order_logistics_delivery_phone (delivery_phone),
    CONSTRAINT fk_order_logistics_order FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE
);
