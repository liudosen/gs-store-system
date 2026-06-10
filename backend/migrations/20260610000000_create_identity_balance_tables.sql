CREATE TABLE IF NOT EXISTS identity_balance_accounts (
    id          BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    identity_no VARCHAR(32) NOT NULL UNIQUE COMMENT '认证号（健康卡权益号/身份证号）',
    balance     BIGINT NOT NULL DEFAULT 0 COMMENT '当前余额，单位分',
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_identity_balance_accounts_identity_no (identity_no)
);

CREATE TABLE IF NOT EXISTS identity_balance_transactions (
    id                BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    identity_no       VARCHAR(32) NOT NULL COMMENT '认证号（健康卡权益号/身份证号）',
    source_openid     VARCHAR(64) NULL COMMENT '触发本次变动的微信 openid',
    amount            BIGINT NOT NULL COMMENT '变动金额，单位分',
    balance_after     BIGINT NOT NULL COMMENT '变动后余额，单位分',
    `type`            TINYINT NOT NULL COMMENT '1=充值 2=余额消费',
    external_order_no VARCHAR(128) NULL COMMENT '外部订单号',
    status            TINYINT NOT NULL DEFAULT 0 COMMENT '0=失败 1=成功',
    remark            VARCHAR(500) NULL COMMENT '备注/失败原因',
    request_hash      VARCHAR(64) NULL COMMENT '充值请求幂等哈希',
    created_at        DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_identity_balance_transactions_identity_no (identity_no),
    INDEX idx_identity_balance_transactions_source_openid (source_openid),
    INDEX idx_identity_balance_transactions_status (status),
    INDEX idx_identity_balance_transactions_request_hash (request_hash)
);

INSERT INTO identity_balance_accounts (identity_no, balance, created_at, updated_at)
SELECT old_balances.identity_no,
       SUM(old_balances.balance) AS balance,
       MIN(old_balances.created_at) AS created_at,
       NOW() AS updated_at
FROM (
    SELECT UPPER(TRIM(w.id_card_number)) AS identity_no,
           ba.balance AS balance,
           ba.created_at AS created_at
    FROM balance_accounts ba
    INNER JOIN wechat_users w ON w.openid = ba.openid
    WHERE TRIM(COALESCE(w.id_card_number, '')) <> ''

    UNION ALL

    SELECT UPPER(TRIM(w.id_card_number)) AS identity_no,
           latest_tx.balance_after AS balance,
           latest_tx.created_at AS created_at
    FROM (
        SELECT bt.openid, bt.balance_after, bt.created_at
        FROM balance_transactions bt
        INNER JOIN (
            SELECT openid, MAX(id) AS max_id
            FROM balance_transactions
            GROUP BY openid
        ) last_tx ON last_tx.openid = bt.openid AND last_tx.max_id = bt.id
    ) latest_tx
    INNER JOIN wechat_users w ON w.openid = latest_tx.openid
    LEFT JOIN balance_accounts ba ON ba.openid = latest_tx.openid
    WHERE ba.openid IS NULL
      AND TRIM(COALESCE(w.id_card_number, '')) <> ''
) old_balances
GROUP BY old_balances.identity_no
ON DUPLICATE KEY UPDATE
    balance = VALUES(balance),
    updated_at = NOW();

INSERT INTO identity_balance_transactions (
    identity_no,
    source_openid,
    amount,
    balance_after,
    `type`,
    external_order_no,
    status,
    remark,
    request_hash,
    created_at
)
SELECT UPPER(TRIM(w.id_card_number)) AS identity_no,
       bt.openid AS source_openid,
       bt.amount,
       bt.balance_after,
       bt.`type`,
       bt.external_order_no,
       bt.status,
       bt.remark,
       bt.request_hash,
       bt.created_at
FROM balance_transactions bt
INNER JOIN wechat_users w ON w.openid = bt.openid
WHERE TRIM(COALESCE(w.id_card_number, '')) <> ''
  AND NOT EXISTS (
      SELECT 1
      FROM identity_balance_transactions ibt
      WHERE ibt.identity_no = UPPER(TRIM(w.id_card_number))
        AND COALESCE(ibt.source_openid, '') = bt.openid
        AND COALESCE(ibt.request_hash, '') = COALESCE(bt.request_hash, '')
        AND COALESCE(ibt.external_order_no, '') = COALESCE(bt.external_order_no, '')
        AND ibt.amount = bt.amount
        AND ibt.balance_after = bt.balance_after
        AND ibt.`type` = bt.`type`
        AND ibt.status = bt.status
        AND ibt.created_at = bt.created_at
  );
