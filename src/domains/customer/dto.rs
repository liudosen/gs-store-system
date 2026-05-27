use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SendCustomerSmsCodeRequest {
    pub phone: String,
}

#[derive(Deserialize)]
pub struct CustomerLoginBySmsRequest {
    pub phone: String,
    pub code: String,
}

#[derive(Deserialize)]
pub struct UpdateRegionRequest {
    pub region_code: String,
}

#[derive(Deserialize)]
pub struct UpsertAddressRequest {
    pub region_code: String,
    pub region_name: String,
    pub city_name: String,
    pub district_name: String,
    pub detail_address: String,
    pub contact_name: String,
    pub contact_phone: String,
    pub is_default: bool,
}

#[derive(Serialize)]
pub struct CustomerSmsCodeData {
    pub expires_in_seconds: u64,
    pub next_send_in_seconds: u64,
}

#[derive(Serialize)]
pub struct CustomerLoginData {
    pub user_id: i64,
    pub phone: String,
    pub token: String,
}

#[derive(Serialize)]
pub struct CustomerMeData {
    pub user: CustomerProfileData,
    pub regions: Vec<RegionData>,
}

#[derive(Serialize)]
pub struct CustomerProfileData {
    pub id: i64,
    pub phone: String,
    pub selected_region_code: Option<String>,
    pub selected_region_name: Option<String>,
}

#[derive(Serialize)]
pub struct AddressListData {
    pub items: Vec<AddressData>,
}

#[derive(Serialize)]
pub struct AddressData {
    pub id: i64,
    pub region_code: String,
    pub region_name: String,
    pub city_name: String,
    pub district_name: String,
    pub detail_address: String,
    pub contact_name: String,
    pub contact_phone: String,
    pub is_default: bool,
}

#[derive(Serialize)]
pub struct UpsertAddressData {
    pub address: AddressData,
}

#[derive(Clone, Serialize)]
pub struct RegionData {
    pub code: String,
    pub name: String,
    pub city_name: String,
    pub district_name: String,
}
