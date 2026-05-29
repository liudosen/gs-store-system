use crate::error::AppError;
use sqlx::Row;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct InventoryChange {
    pub sku_id: u64,
    pub spu_id: u64,
    pub quantity: i32,
}

#[derive(Debug, sqlx::FromRow)]
struct OrderInventoryRow {
    sku_id: u64,
    spu_id: u64,
    quantity: i32,
}

pub async fn reserve_inventory(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    changes: &[InventoryChange],
) -> Result<(), AppError> {
    let aggregated = aggregate_changes(changes);

    for ((sku_id, spu_id), quantity) in &aggregated {
        let result = sqlx::query(
            "UPDATE goods_skus gs \
             JOIN goods g ON g.id = gs.spu_id \
             SET gs.stock_quantity = gs.stock_quantity - ? \
             WHERE gs.id = ? AND gs.spu_id = ? AND g.status = 1 AND gs.stock_quantity >= ?",
        )
        .bind(*quantity)
        .bind(*sku_id)
        .bind(*spu_id)
        .bind(*quantity)
        .execute(&mut **tx)
        .await?;

        if result.rows_affected() != 1 {
            return Err(AppError::BadRequest(format!(
                "SKU {} stock insufficient or product offline",
                sku_id
            )));
        }
    }

    refresh_goods_stock(tx, aggregated.keys().map(|(_, spu_id)| *spu_id)).await
}

pub async fn release_inventory_for_order(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    order_id: u64,
) -> Result<(), AppError> {
    let rows = sqlx::query_as::<_, OrderInventoryRow>(
        "SELECT sku_id, spu_id, quantity FROM order_items WHERE order_id = ?",
    )
    .bind(order_id)
    .fetch_all(&mut **tx)
    .await?;

    if rows.is_empty() {
        return Ok(());
    }

    let changes: Vec<InventoryChange> = rows
        .into_iter()
        .map(|row| InventoryChange {
            sku_id: row.sku_id,
            spu_id: row.spu_id,
            quantity: row.quantity,
        })
        .collect();
    let aggregated = aggregate_changes(&changes);

    for ((sku_id, _spu_id), quantity) in &aggregated {
        sqlx::query("UPDATE goods_skus SET stock_quantity = stock_quantity + ? WHERE id = ?")
            .bind(*quantity)
            .bind(*sku_id)
            .execute(&mut **tx)
            .await?;
    }

    refresh_goods_stock(tx, aggregated.keys().map(|(_, spu_id)| *spu_id)).await
}

fn aggregate_changes(changes: &[InventoryChange]) -> BTreeMap<(u64, u64), i32> {
    let mut aggregated = BTreeMap::new();
    for change in changes {
        *aggregated
            .entry((change.sku_id, change.spu_id))
            .or_insert(0) += change.quantity;
    }
    aggregated
}

async fn refresh_goods_stock<I>(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    spu_ids: I,
) -> Result<(), AppError>
where
    I: IntoIterator<Item = u64>,
{
    let unique_spu_ids: BTreeSet<u64> = spu_ids.into_iter().collect();

    for spu_id in unique_spu_ids {
        let row = sqlx::query(
            "SELECT CAST(COALESCE(SUM(stock_quantity), 0) AS SIGNED) AS total_stock \
             FROM goods_skus WHERE spu_id = ?",
        )
        .bind(spu_id)
        .fetch_one(&mut **tx)
        .await?;
        let total_stock: i64 = row.try_get("total_stock")?;
        let total_stock_i32 = i32::try_from(total_stock)
            .map_err(|_| AppError::InternalError("goods stock overflow".to_string()))?;

        sqlx::query("UPDATE goods SET spu_stock_quantity = ?, is_sold_out = ? WHERE id = ?")
            .bind(total_stock_i32)
            .bind(total_stock_i32 == 0)
            .bind(spu_id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}
