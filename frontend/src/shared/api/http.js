export async function readApiPayload(response) {
  const text = await response.text();
  if (!text) {
    return { message: "" };
  }

  try {
    return JSON.parse(text);
  } catch {
    return { message: text };
  }
}

function buildHeaders(token) {
  const headers = {
    "Content-Type": "application/json",
  };

  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  return headers;
}

export async function getJson(url, token) {
  const response = await fetch(url, {
    method: "GET",
    headers: buildHeaders(token),
  });
  const data = await readApiPayload(response);

  return {
    ok: response.ok,
    status: response.status,
    data,
  };
}

export async function postJson(url, payload, token) {
  const response = await fetch(url, {
    method: "POST",
    headers: buildHeaders(token),
    body: JSON.stringify(payload),
  });
  const data = await readApiPayload(response);

  return {
    ok: response.ok,
    status: response.status,
    data,
  };
}
