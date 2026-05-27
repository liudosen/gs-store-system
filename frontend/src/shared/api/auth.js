import { getJson, postJson } from "./http";

export function sendVeteranSmsCode(payload) {
  return postJson("/api/veteran/auth/send-sms-code", payload);
}

export function loginVeteranBySms(payload) {
  return postJson("/api/veteran/auth/register-by-sms", payload);
}

export function getVeteranMe(token) {
  return getJson("/api/veteran/me", token);
}

export function updateVeteranRegion(payload, token) {
  return postJson("/api/veteran/me/region", payload, token);
}
