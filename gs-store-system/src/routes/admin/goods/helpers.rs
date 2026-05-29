use crate::error::AppError;
use crate::models::{CreateGoodsRequest, GoodsRow, SkuRequest, Spec, UpdateGoodsRequest};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

type GoodsCountQuery<'q> =
    sqlx::query::QueryScalar<'q, sqlx::MySql, i64, sqlx::mysql::MySqlArguments>;
type GoodsListQuery<'q, T> = sqlx::query::QueryAs<'q, sqlx::MySql, T, sqlx::mysql::MySqlArguments>;

#[derive(Debug, Clone)]
pub(super) struct GoodsListFilters {
    pub category_id: Option<String>,
    pub keyword: Option<String>,
    pub status: Option<i32>,
}

#[derive(Debug, Clone)]
pub(super) struct GoodsListPlan {
    pub page: u64,
    pub page_size: u64,
    pub offset: u64,
    pub count_sql: String,
    pub list_sql: String,
    pub filters: GoodsListFilters,
}

#[derive(Debug, Clone)]
pub(super) struct PersistSku {
    pub sku_image: Option<String>,
    pub spec_info: Vec<crate::models::SkuSpecInfo>,
    pub sale_price: i64,
    pub line_price: i64,
    pub stock_quantity: i32,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub(super) struct CreateGoodsPlan {
    pub store_id: String,
    pub saas_id: String,
    pub title: String,
    pub primary_image: String,
    pub images_json: String,
    pub desc_json: String,
    pub spec_json: String,
    pub tag_json: String,
    pub category_id: Option<String>,
    pub status: bool,
    pub min_sale: i64,
    pub max_line: i64,
    pub total_stock: i32,
    pub is_sold_out: bool,
    pub persist_skus: Vec<PersistSku>,
}

#[derive(Debug, Clone)]
pub(super) struct UpdateGoodsPlan {
    pub store_id: String,
    pub saas_id: String,
    pub title: String,
    pub primary_image: String,
    pub images_json: String,
    pub desc_json: String,
    pub spec_json: String,
    pub tag_json: String,
    pub category_id: Option<String>,
    pub status: bool,
    pub min_sale: i64,
    pub max_line: i64,
    pub total_stock: i32,
    pub is_sold_out: bool,
    pub persist_skus: Option<Vec<PersistSku>>,
}

pub(super) fn build_goods_list_plan(query: &super::ListGoodsQuery) -> GoodsListPlan {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let mut conditions = vec!["1=1"];
    if query.category_id.is_some() {
        conditions.push("category_id = ?");
    }
    if query.keyword.is_some() {
        conditions.push("title LIKE ?");
    }
    if query.status.is_some() {
        conditions.push("status = ?");
    }

    let where_clause = conditions.join(" AND ");
    let count_sql = format!("SELECT COUNT(*) FROM goods WHERE {}", where_clause);
    let list_sql = format!(
        "SELECT id, store_id, saas_id, title, primary_image, images, desc_images, spec_list, min_sale_price, max_line_price, spu_tag_list, is_sold_out, spu_stock_quantity, sold_num, category_id, status FROM goods WHERE {} ORDER BY id DESC LIMIT ? OFFSET ?",
        where_clause
    );

    GoodsListPlan {
        page,
        page_size,
        offset,
        count_sql,
        list_sql,
        filters: GoodsListFilters {
            category_id: query.category_id.clone(),
            keyword: query.keyword.clone(),
            status: query.status,
        },
    }
}

pub(super) fn bind_goods_count_query<'q>(
    mut query: GoodsCountQuery<'q>,
    filters: &GoodsListFilters,
) -> GoodsCountQuery<'q> {
    if let Some(cid) = filters.category_id.as_ref() {
        query = query.bind(cid.clone());
    }
    if let Some(keyword) = filters.keyword.as_ref() {
        query = query.bind(format!("%{}%", keyword));
    }
    if let Some(status) = filters.status {
        query = query.bind(status);
    }
    query
}

pub(super) fn bind_goods_list_query<'q, T>(
    mut query: GoodsListQuery<'q, T>,
    filters: &GoodsListFilters,
) -> GoodsListQuery<'q, T> {
    if let Some(cid) = filters.category_id.as_ref() {
        query = query.bind(cid.clone());
    }
    if let Some(keyword) = filters.keyword.as_ref() {
        query = query.bind(format!("%{}%", keyword));
    }
    if let Some(status) = filters.status {
        query = query.bind(status);
    }
    query
}

pub(super) fn compute_stock_stats(skus: &[PersistSku]) -> (i64, i64, i32, bool) {
    if skus.is_empty() {
        return (0, 0, 0, true);
    }

    let min_sale = skus.iter().map(|s| s.sale_price).min().unwrap_or(0);
    let max_line = skus.iter().map(|s| s.line_price).max().unwrap_or(0);
    let total_stock: i32 = skus.iter().map(|s| s.stock_quantity).sum();
    let is_sold_out = total_stock == 0;
    (min_sale, max_line, total_stock, is_sold_out)
}

pub(super) fn to_persist_skus(skus: &[SkuRequest]) -> Vec<PersistSku> {
    skus.iter()
        .map(|sku| PersistSku {
            sku_image: sku.sku_image.clone(),
            spec_info: sku.spec_info.clone(),
            sale_price: sku.sale_price,
            line_price: sku.line_price,
            stock_quantity: sku.stock_quantity,
            is_default: false,
        })
        .collect()
}

pub(super) fn validate_skus_against_specs(
    spec_list: &[Spec],
    skus: &[SkuRequest],
) -> Result<(), AppError> {
    if spec_list.is_empty() {
        return Ok(());
    }

    if skus.is_empty() {
        return Err(AppError::BadRequest(
            "specs are configured, please generate SKU combinations first".to_string(),
        ));
    }

    let mut allowed_values: HashMap<String, HashSet<String>> = HashMap::new();
    for spec in spec_list {
        if spec.spec_id.trim().is_empty() {
            return Err(AppError::BadRequest("spec ID cannot be empty".to_string()));
        }

        let entry = allowed_values.entry(spec.spec_id.clone()).or_default();
        for value in &spec.spec_value_list {
            if value.spec_value_id.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "spec value ID cannot be empty".to_string(),
                ));
            }
            entry.insert(value.spec_value_id.clone());
        }
    }

    let mut combo_keys: HashSet<String> = HashSet::new();
    for sku in skus {
        if sku.spec_info.is_empty() {
            return Err(AppError::BadRequest(
                "SKU spec info cannot be empty when generating combinations".to_string(),
            ));
        }

        let mut seen_specs: HashSet<String> = HashSet::new();
        let mut parts = Vec::with_capacity(sku.spec_info.len());
        for spec_info in &sku.spec_info {
            let spec_id = spec_info.spec_id.clone();
            let value_id = spec_info.spec_value_id.clone();

            let values = allowed_values.get(&spec_id).ok_or_else(|| {
                AppError::BadRequest(format!("SKU references undefined spec: {}", spec_id))
            })?;
            if !values.contains(&value_id) {
                return Err(AppError::BadRequest(format!(
                    "SKU references undefined spec value: {} / {}",
                    spec_id, value_id
                )));
            }
            if !seen_specs.insert(spec_id.clone()) {
                return Err(AppError::BadRequest(format!(
                    "SKU spec is duplicated: {}",
                    spec_id
                )));
            }
            parts.push(format!("{}:{}", spec_id, value_id));
        }

        if seen_specs.len() != spec_list.len() {
            return Err(AppError::BadRequest(
                "each SKU must include a complete spec combination".to_string(),
            ));
        }

        parts.sort();
        let combo_key = parts.join("|");
        if !combo_keys.insert(combo_key) {
            return Err(AppError::BadRequest(
                "SKU combination is duplicated, please check the specs".to_string(),
            ));
        }
    }

    Ok(())
}

pub(super) fn prepare_create_goods_plan(
    body: &CreateGoodsRequest,
) -> Result<CreateGoodsPlan, AppError> {
    if body.title.is_empty() {
        return Err(AppError::BadRequest(
            "product name cannot be empty".to_string(),
        ));
    }
    if body.primary_image.is_empty() {
        return Err(AppError::BadRequest(
            "primary image cannot be empty".to_string(),
        ));
    }
    if body.skus.is_empty() {
        return Err(AppError::BadRequest("skus cannot be empty".to_string()));
    }

    validate_skus_against_specs(&body.spec_list, &body.skus)?;
    let persist_skus = to_persist_skus(&body.skus);
    let (min_sale, max_line, total_stock, is_sold_out) = compute_stock_stats(&persist_skus);

    Ok(CreateGoodsPlan {
        store_id: body.store_id.clone().unwrap_or_default(),
        saas_id: body.saas_id.clone().unwrap_or_default(),
        title: body.title.clone(),
        primary_image: body.primary_image.clone(),
        images_json: serialize_json_or_empty(&body.images),
        desc_json: serialize_json_or_empty(&body.desc_images),
        spec_json: serialize_json_or_empty(&body.spec_list),
        tag_json: serialize_json_or_empty(&body.spu_tag_list),
        category_id: body.category_id.clone(),
        status: body.status.unwrap_or(true),
        min_sale,
        max_line,
        total_stock,
        is_sold_out,
        persist_skus,
    })
}

pub(super) fn prepare_update_goods_plan(
    existing: &GoodsRow,
    body: &UpdateGoodsRequest,
) -> Result<UpdateGoodsPlan, AppError> {
    let store_id = body
        .store_id
        .clone()
        .unwrap_or_else(|| existing.store_id.clone());
    let saas_id = body
        .saas_id
        .clone()
        .unwrap_or_else(|| existing.saas_id.clone());
    let title = body.title.clone().unwrap_or_else(|| existing.title.clone());
    let primary_image = body
        .primary_image
        .clone()
        .unwrap_or_else(|| existing.primary_image.clone());
    let images_json = json_or_existing(body.images.as_ref(), &existing.images);
    let desc_json = json_or_existing(body.desc_images.as_ref(), &existing.desc_images);
    let provided_spec_list = body.spec_list.clone();
    let spec_json = match provided_spec_list.as_ref() {
        Some(v) => serialize_json_or_empty(v),
        None => existing.spec_list.clone(),
    };
    let tag_json = json_or_existing(body.spu_tag_list.as_ref(), &existing.spu_tag_list);
    let category_id = body.category_id.clone().or(existing.category_id.clone());
    let status = body.status.unwrap_or(existing.status);

    let persist_skus = if let Some(skus) = body.skus.as_ref() {
        if skus.is_empty() {
            return Err(AppError::BadRequest("skus cannot be empty".to_string()));
        }

        let effective_spec_list: Vec<Spec> = provided_spec_list
            .clone()
            .unwrap_or_else(|| serde_json::from_str(&existing.spec_list).unwrap_or_default());
        validate_skus_against_specs(&effective_spec_list, skus)?;
        Some(to_persist_skus(skus))
    } else {
        None
    };

    let (min_sale, max_line, total_stock, is_sold_out) = if let Some(ref skus) = persist_skus {
        compute_stock_stats(skus)
    } else {
        (
            existing.min_sale_price,
            existing.max_line_price,
            existing.spu_stock_quantity,
            existing.is_sold_out,
        )
    };

    Ok(UpdateGoodsPlan {
        store_id,
        saas_id,
        title,
        primary_image,
        images_json,
        desc_json,
        spec_json,
        tag_json,
        category_id,
        status,
        min_sale,
        max_line,
        total_stock,
        is_sold_out,
        persist_skus,
    })
}

pub(super) async fn insert_goods_skus(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    spu_id: u64,
    skus: &[PersistSku],
) -> Result<(), AppError> {
    for sku in skus {
        let spec_json = serialize_json_or_empty(&sku.spec_info);
        sqlx::query("INSERT INTO goods_skus (spu_id, sku_image, spec_info, sale_price, line_price, stock_quantity, is_default) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(spu_id)
            .bind(&sku.sku_image)
            .bind(&spec_json)
            .bind(sku.sale_price)
            .bind(sku.line_price)
            .bind(sku.stock_quantity)
            .bind(sku.is_default)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

fn serialize_json_or_empty<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "[]".to_string())
}

fn json_or_existing<T: Serialize>(value: Option<&T>, existing: &str) -> String {
    value
        .map(serialize_json_or_empty)
        .unwrap_or_else(|| existing.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SkuSpecInfo, SpecValue};

    #[test]
    fn build_goods_list_plan_applies_filters_and_bounds() {
        let plan = build_goods_list_plan(&super::super::ListGoodsQuery {
            page: Some(0),
            page_size: Some(250),
            category_id: Some("cat-1".to_string()),
            keyword: Some("shoe".to_string()),
            status: Some(1),
        });

        assert_eq!(plan.page, 1);
        assert_eq!(plan.page_size, 100);
        assert_eq!(plan.offset, 0);
        assert_eq!(
            plan.count_sql,
            "SELECT COUNT(*) FROM goods WHERE 1=1 AND category_id = ? AND title LIKE ? AND status = ?"
        );
        assert!(plan.list_sql.contains("ORDER BY id DESC LIMIT ? OFFSET ?"));
        assert_eq!(plan.filters.category_id.as_deref(), Some("cat-1"));
        assert_eq!(plan.filters.keyword.as_deref(), Some("shoe"));
        assert_eq!(plan.filters.status, Some(1));
    }

    #[test]
    fn compute_stock_stats_handles_empty_and_non_empty_skus() {
        let empty: Vec<PersistSku> = Vec::new();
        assert_eq!(compute_stock_stats(&empty), (0, 0, 0, true));

        let skus = vec![
            PersistSku {
                sku_image: None,
                spec_info: vec![],
                sale_price: 120,
                line_price: 200,
                stock_quantity: 3,
                is_default: false,
            },
            PersistSku {
                sku_image: None,
                spec_info: vec![],
                sale_price: 90,
                line_price: 250,
                stock_quantity: 7,
                is_default: false,
            },
        ];

        assert_eq!(compute_stock_stats(&skus), (90, 250, 10, false));
    }

    #[test]
    fn validate_skus_against_specs_rejects_duplicate_combinations() {
        let spec_list = vec![Spec {
            spec_id: "color".to_string(),
            title: "Color".to_string(),
            spec_value_list: vec![SpecValue {
                spec_value_id: "red".to_string(),
                spec_value: "Red".to_string(),
                image: None,
            }],
        }];
        let sku = SkuRequest {
            sku_image: None,
            spec_info: vec![SkuSpecInfo {
                spec_id: "color".to_string(),
                spec_value_id: "red".to_string(),
                spec_value: None,
            }],
            sale_price: 100,
            line_price: 120,
            stock_quantity: 1,
        };

        let result = validate_skus_against_specs(&spec_list, &[sku.clone(), sku]);
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }
}
