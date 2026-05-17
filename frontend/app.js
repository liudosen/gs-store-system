const statusEl = document.querySelector("#status");
const messagesEl = document.querySelector("#messages");
const form = document.querySelector("#message-form");
const input = document.querySelector("#message-input");
const refreshButton = document.querySelector("#refresh");

async function api(path, options) {
  const response = await fetch(`/api${path}`, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });

  if (!response.ok) {
    throw new Error(`Request failed: ${response.status}`);
  }

  return response.json();
}

function renderMessages(messages) {
  messagesEl.replaceChildren(
    ...messages.map((message) => {
      const item = document.createElement("li");
      const label = document.createElement("span");
      const text = document.createElement("p");

      label.textContent = `#${message.id}`;
      text.textContent = message.text;
      item.append(label, text);

      return item;
    }),
  );
}

async function loadMessages() {
  statusEl.textContent = "正在读取后端数据...";
  const [health, messages] = await Promise.all([
    api("/health"),
    api("/messages"),
  ]);
  statusEl.textContent = `后端状态：${health.status}`;
  renderMessages(messages);
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  const text = input.value.trim();

  if (!text) {
    return;
  }

  await api("/messages", {
    method: "POST",
    body: JSON.stringify({ text }),
  });

  input.value = "";
  await loadMessages();
});

refreshButton.addEventListener("click", loadMessages);

loadMessages().catch((error) => {
  statusEl.textContent = error.message;
});
