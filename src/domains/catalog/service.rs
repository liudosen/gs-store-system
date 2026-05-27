use crate::{
    common::api::ApiError,
    domains::catalog::{
        dto::{ServiceCatalogData, ServiceItemData},
        entity::ServiceItemRow,
        repository,
    },
    infra::state::AppState,
};

pub async fn list_service_items(
    state: &AppState,
    region_code: Option<String>,
) -> Result<ServiceCatalogData, ApiError> {
    let items = repository::list_service_items(&state.db, region_code.as_deref()).await?;
    let time_slots = repository::list_service_time_slots(&state.db).await?;

    Ok(ServiceCatalogData {
        items: items
            .into_iter()
            .filter(|item| item.visible_in_customer == 1)
            .map(map_service_item)
            .collect(),
        time_slots,
    })
}

pub fn map_service_item(item: ServiceItemRow) -> ServiceItemData {
    ServiceItemData {
        id: item.id,
        code: item.code,
        category_name: item.category_name,
        name: item.name,
        short_description: item.short_description,
        badge: item.badge,
        base_price: item.base_price,
        duration_minutes: item.duration_minutes,
    }
}
