import { getJson, postJson } from "./http";

export function listAvailableOrders(token) {
  return getJson("/api/veteran/orders/available", token);
}

export function acceptOrder(orderId, token) {
  return postJson(`/api/veteran/orders/${orderId}/accept`, {}, token);
}

export function listAssignedOrders(token) {
  return getJson("/api/veteran/orders/assigned", token);
}

export function getDailyStats(token) {
  return getJson("/api/veteran/stats/daily", token);
}

export function cancelOrder(orderId, token) {
  return postJson(`/api/veteran/orders/${orderId}/cancel`, {}, token);
}