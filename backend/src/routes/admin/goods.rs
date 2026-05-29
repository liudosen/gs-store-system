mod helpers;

use crate::error::AppError;
use crate::models::{
    build_admin_goods_detail, AdminGoodsDetail, CreateGoodsRequest, GoodsRow, GoodsSkuRow,
    UpdateGoodsRequest,
};
use crate::routes::admin::auth::authorize_admin;
use crate::routes::admin::permissions::GOODS_VIEW;
use crate::routes::ApiResponse;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ListGoodsQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub category_id: Option<String>,
    pub keyword: Option<String>,
    pub status: Option<i32>,
}

#[derive(serde::Serialize)]
pub struct PagedGoods {
    pub list: Vec<AdminGoodsDetail>,
    pub total: i64,
    pub page: u64,
    pub page_size: u64,
}

async fn fetch_skus(state: &AppState, spu_id: u64) -> Result<Vec<GoodsSkuRow>, AppError> {
    let skus = sqlx::query_as::<_, GoodsSkuRow>(
        "SELECT id, spu_id, sku_image, spec_info, sale_price, line_price, stock_quantity, is_default \
         FROM goods_skus WHERE spu_id = ? ORDER BY id",
    )
    .bind(spu_id)
    .fetch_all(&state.db)
    .await?;
    Ok(skus)
}

pub async fn list_goods(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(q): Query<ListGoodsQuery>,
) -> Result<Json<ApiResponse<PagedGoods>>, AppError> {
    authorize_admin(&state, &headers, &[GOODS_VIEW]).await?;

    let plan = helpers::build_goods_list_plan(&q);
    let total: i64 =
        helpers::bind_goods_count_query(sqlx::query_scalar(&plan.count_sql), &plan.filters)
            .fetch_one(&state.db)
            .await?;

    let rows: Vec<GoodsRow> = helpers::bind_goods_list_query(
        sqlx::query_as::<_, GoodsRow>(&plan.list_sql),
        &plan.filters,
    )
    .bind(plan.page_size)
    .bind(plan.offset)
    .fetch_all(&state.db)
    .await?;

    let mut list = Vec::with_capacity(rows.len());
    for row in &rows {
        let skus = fetch_skus(&state, row.id).await?;
        list.push(build_admin_goods_detail(row, skus));
    }

    Ok(Json(ApiResponse::success(PagedGoods {
        list,
        total,
        page: plan.page,
        page_size: plan.page_size,
    })))
}

pub async fn get_goods(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
) -> Result<Json<ApiResponse<AdminGoodsDetail>>, AppError> {
    authorize_admin(&state, &headers, &[GOODS_VIEW]).await?;

    let row = sqlx::query_as::<_, GoodsRow>(
        "SELECT id, store_id, saas_id, title, primary_image, images, desc_images, spec_list, \
         min_sale_price, max_line_price, spu_tag_list, is_sold_out, spu_stock_quantity, sold_num, \
         category_id, status FROM goods WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("goods not found".to_string()))?;

    let skus = fetch_skus(&state, id).await?;
    Ok(Json(ApiResponse::success(build_admin_goods_detail(
        &row, skus,
    ))))
}

pub async fn create_goods(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateGoodsRequest>,
) -> Result<Json<ApiResponse<AdminGoodsDetail>>, AppError> {
    authorize_admin(&state, &headers, &[GOODS_VIEW]).await?;

    let plan = helpers::prepare_create_goods_plan(&body)?;
    let mut tx = state.db.begin().await?;

    sqlx::query("INSERT INTO goods (store_id, saas_id, title, primary_image, images, desc_images, spec_list, min_sale_price, max_line_price, spu_tag_list, is_sold_out, spu_stock_quantity, sold_num, category_id, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?)")
    .bind(&plan.store_id)
    .bind(&plan.saas_id)
    .bind(&plan.title)
    .bind(&plan.primary_image)
    .bind(&plan.images_json)
    .bind(&plan.desc_json)
    .bind(&plan.spec_json)
    .bind(plan.min_sale)
    .bind(plan.max_line)
    .bind(&plan.tag_json)
    .bind(plan.is_sold_out)
    .bind(plan.total_stock)
    .bind(&plan.category_id)
    .bind(plan.status)
    .execute(&mut *tx)
    .await?;

    let spu_id: u64 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
        .fetch_one(&mut *tx)
        .await?;

    helpers::insert_goods_skus(&mut tx, spu_id, &plan.persist_skus).await?;
    tx.commit().await?;

    let row = sqlx::query_as::<_, GoodsRow>("SELECT id, store_id, saas_id, title, primary_image, images, desc_images, spec_list, min_sale_price, max_line_price, spu_tag_list, is_sold_out, spu_stock_quantity, sold_num, category_id, status FROM goods WHERE id = ?")
    .bind(spu_id)
    .fetch_one(&state.db)
    .await?;

    let skus = fetch_skus(&state, spu_id).await?;
    Ok(Json(ApiResponse::success(build_admin_goods_detail(
        &row, skus,
    ))))
}

pub async fn update_goods(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
    Json(body): Json<UpdateGoodsRequest>,
) -> Result<Json<ApiResponse<AdminGoodsDetail>>, AppError> {
    authorize_admin(&state, &headers, &[GOODS_VIEW]).await?;

    let existing = sqlx::query_as::<_, GoodsRow>("SELECT id, store_id, saas_id, title, primary_image, images, desc_images, spec_list, min_sale_price, max_line_price, spu_tag_list, is_sold_out, spu_stock_quantity, sold_num, category_id, status FROM goods WHERE id = ?")
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("goods not found".to_string()))?;

    let plan = helpers::prepare_update_goods_plan(&existing, &body)?;
    let mut tx = state.db.begin().await?;

    sqlx::query("UPDATE goods SET store_id=?, saas_id=?, title=?, primary_image=?, images=?, desc_images=?, spec_list=?, min_sale_price=?, max_line_price=?, spu_tag_list=?, is_sold_out=?, spu_stock_quantity=?, category_id=?, status=? WHERE id=?")
    .bind(&plan.store_id)
    .bind(&plan.saas_id)
    .bind(&plan.title)
    .bind(&plan.primary_image)
    .bind(&plan.images_json)
    .bind(&plan.desc_json)
    .bind(&plan.spec_json)
    .bind(plan.min_sale)
    .bind(plan.max_line)
    .bind(&plan.tag_json)
    .bind(plan.is_sold_out)
    .bind(plan.total_stock)
    .bind(&plan.category_id)
    .bind(plan.status)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    if let Some(ref skus) = plan.persist_skus {
        sqlx::query("DELETE FROM goods_skus WHERE spu_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        helpers::insert_goods_skus(&mut tx, id, skus).await?;
    }

    tx.commit().await?;

    let row = sqlx::query_as::<_, GoodsRow>("SELECT id, store_id, saas_id, title, primary_image, images, desc_images, spec_list, min_sale_price, max_line_price, spu_tag_list, is_sold_out, spu_stock_quantity, sold_num, category_id, status FROM goods WHERE id = ?")
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    let skus = fetch_skus(&state, id).await?;
    Ok(Json(ApiResponse::success(build_admin_goods_detail(
        &row, skus,
    ))))
}

pub async fn delete_goods(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    authorize_admin(&state, &headers, &[GOODS_VIEW]).await?;

    let result = sqlx::query("DELETE FROM goods WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("goods not found".to_string()));
    }

    Ok(Json(ApiResponse::success(())))
}
