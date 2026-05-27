import { getJson, postJson } from "./http";

export function sendCustomerSmsCode(payload) {
  return postJson("/api/customer/auth/send-sms-code", payload);
}

export function loginCustomerBySms(payload) {
  return postJson("/api/customer/auth/login-by-sms", payload);
}

export function getCustomerMe(token) {
  return getJson("/api/customer/me", token);
}

export function updateCustomerRegion(payload, token) {
  return postJson("/api/customer/me/region", payload, token);
}

export function listCustomerAddresses(token) {
  return getJson("/api/customer/addresses", token);
}

export function createCustomerAddress(payload, token) {
  return postJson("/api/customer/addresses", payload, token);
}

export function updateCustomerAddress(addressId, payload, token) {
  return postJson(`/api/customer/addresses/${addressId}`, payload, token);
}

export function listServiceItems(regionCode, token) {
  const query = regionCode ? `?region_code=${encodeURIComponent(regionCode)}` : "";
  return getJson(`/api/customer/service-items${query}`, token);
}

export function listOrders(token, params = {}) {
  const search = new URLSearchParams();
  if (params.category) search.set("category", params.category);
  if (params.offset != null) search.set("offset", String(params.offset));
  if (params.limit != null) search.set("limit", String(params.limit));
  const query = search.toString() ? `?${search.toString()}` : "";
  return getJson(`/api/customer/orders${query}`, token);
}

export function createOrder(payload, token) {
  return postJson("/api/customer/orders", payload, token);
}
