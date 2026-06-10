ALTER TABLE wechat_users
MODIFY COLUMN id_card_number VARCHAR(32) DEFAULT '' COMMENT '认证号（健康卡权益号/身份证号）';
