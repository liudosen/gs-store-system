import { postJson } from "./http";

export function submitVeteranJoin(payload) {
  return postJson("/api/onboarding/veteran-join", payload);
}
