ALTER TABLE goods_skus
    ADD COLUMN is_default TINYINT(1) NOT NULL DEFAULT 0 AFTER stock_quantity;

INSERT INTO goods_skus (spu_id, sku_image, spec_info, sale_price, line_price, stock_quantity, is_default)
SELECT
    g.id,
    NULL,
    '[]',
    g.min_sale_price,
    g.max_line_price,
    g.spu_stock_quantity,
    1
FROM goods g
LEFT JOIN goods_skus gs ON gs.spu_id = g.id
WHERE gs.id IS NULL;
