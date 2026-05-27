<script setup>
import { computed, onBeforeUnmount, ref, watch } from "vue";
import VeteranOrders from "../components/veteran/VeteranOrders.vue";
import VeteranProfile from "../components/veteran/VeteranProfile.vue";
import VeteranTraining from "../components/veteran/VeteranTraining.vue";
import VeteranWorkbench from "../components/veteran/VeteranWorkbench.vue";
import { useVeteranPortalLogin } from "../features/portal/useVeteranPortalLogin";
import { updateVeteranRegion } from "../shared/api/auth";
import { VETERAN_TOKEN_STORAGE_KEY } from "../shared/constants/storage";
import {
  acceptOrder,
  cancelOrder,
  getDailyStats,
  listAssignedOrders,
  listAvailableOrders,
} from "../shared/api/veteran-orders";

const emit = defineEmits(["navigate"]);

const {
  availableProjects,
  bootstrap,
  completedCount,
  credentials,
  isBootstrapping,
  isLoggedIn,
  isLoggingIn,
  isSendingCode,
  login,
  loginError,
  loginMessage,
  logout,
  remainingSeconds,
  selectedProjects,
  sendSmsCode,
  toggleProject,
  trainingModules,
  veteranProfile,
} = useVeteranPortalLogin({ onNeedOnboarding: () => go("/veteran-join") });

const DEFAULT_STATS = {
  today_orders: 0,
  today_completed: 0,
  month_orders: 0,
  rating_score: 0.0,
};

const tabKeys = ["workbench", "orders", "training", "profile"];
const activeTab = ref("workbench");
const tabMap = {
  workbench: VeteranWorkbench,
  orders: VeteranOrders,
  training: VeteranTraining,
  profile: VeteranProfile,
};
const activeTabComponent = computed(() => tabMap[activeTab.value]);
const tabs = [
  { key: "workbench", label: "首页" },
  { key: "orders", label: "订单" },
  { key: "training", label: "培训" },
  { key: "profile", label: "我的" },
];

const orders = ref([]);
const assignedOrders = ref([]);
const veteranStats = ref({ ...DEFAULT_STATS });
const wsConnected = ref(false);
const acceptMessage = ref("");
const acceptError = ref(false);
const showOrderDetail = ref(null);

let ws = null;
let reconnectTimer = null;
let wsRegionCode = "";

function getVeteranToken() {
  return localStorage.getItem(VETERAN_TOKEN_STORAGE_KEY);
}

function clearReconnectTimer() {
  if (reconnectTimer) {
    window.clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
}

function closeOrdersWs() {
  clearReconnectTimer();
  wsRegionCode = "";

  if (!ws) {
    wsConnected.value = false;
    return;
  }

  const currentWs = ws;
  ws = null;
  currentWs.onopen = null;
  currentWs.onclose = null;
  currentWs.onerror = null;
  currentWs.onmessage = null;

  try {
    currentWs.close();
  } catch {
    // ignore close errors during teardown
  }

  wsConnected.value = false;
}

function scheduleReconnect() {
  if (reconnectTimer || !getVeteranToken() || !veteranProfile.value?.region_code) {
    return;
  }

  reconnectTimer = window.setTimeout(() => {
    reconnectTimer = null;
    connectOrdersWs();
  }, 5000);
}

function connectOrdersWs(targetRegionCode = veteranProfile.value?.region_code || "") {
  const token = getVeteranToken();
  if (!token || !targetRegionCode) return;

  const hasActiveSocket =
    ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING);

  if (hasActiveSocket && wsRegionCode === targetRegionCode) {
    return;
  }

  if (hasActiveSocket && wsRegionCode !== targetRegionCode) {
    closeOrdersWs();
  }

  clearReconnectTimer();
  wsRegionCode = targetRegionCode;

  const protocol = location.protocol === "https:" ? "wss:" : "ws:";
  const url = `${protocol}//${location.host}/api/veteran/ws/orders?token=${encodeURIComponent(token)}`;

  ws = new WebSocket(url);
  ws.onopen = () => {
    wsConnected.value = true;
  };
  ws.onmessage = (event) => {
    try {
      const message = JSON.parse(event.data);
      if (message.type === "init") {
        orders.value = Array.isArray(message.orders) ? message.orders : [];
        return;
      }

      if (message.type === "new_order" && message.order) {
        orders.value = [message.order, ...orders.value.filter((order) => order.id !== message.order.id)];
      }
    } catch {
      // ignore malformed websocket messages
    }
  };
  ws.onclose = () => {
    wsConnected.value = false;
    ws = null;
    wsRegionCode = "";
    scheduleReconnect();
  };
  ws.onerror = () => {
    wsConnected.value = false;
  };
}

async function loadAvailableOrders() {
  const token = getVeteranToken();
  if (!token) return;

  try {
    const result = await listAvailableOrders(token);
    if (result.ok && result.data?.success) {
      orders.value = result.data.data.items || [];
    }
  } catch {
    // keep last successful available-order snapshot
  }
}

async function loadAssignedOrders() {
  const token = getVeteranToken();
  if (!token) return;

  try {
    const result = await listAssignedOrders(token);
    if (result.ok && result.data?.success) {
      assignedOrders.value = result.data.data.items || [];
    }
  } catch {
    // keep last successful assigned-order snapshot
  }
}

async function loadStats() {
  const token = getVeteranToken();
  if (!token) return;

  try {
    const result = await getDailyStats(token);
    if (result.ok && result.data?.success) {
      veteranStats.value = result.data.data;
    }
  } catch {
    // keep last successful stats snapshot
  }
}

async function loadPortalData() {
  await Promise.all([
    loadAvailableOrders(),
    loadAssignedOrders(),
    loadStats(),
  ]);
}

async function refreshPortalData() {
  const ok = await bootstrap();
  if (!ok) {
    return false;
  }

  connectOrdersWs();
  await loadPortalData();
  return true;
}

function switchTab(key) {
  if (!tabKeys.includes(key)) return;

  activeTab.value = key;

  if (key === "workbench") {
    void loadAvailableOrders();
    return;
  }

  if (key === "orders") {
    void loadAssignedOrders();
  }
}

async function handleAcceptOrder(orderId) {
  const token = getVeteranToken();
  if (!token) return;

  acceptMessage.value = "";
  acceptError.value = false;

  try {
    const result = await acceptOrder(orderId, token);
    if (result.ok && result.data?.success) {
      acceptMessage.value = "接单成功";
      orders.value = orders.value.filter((order) => order.id !== orderId);
      await refreshPortalData();
    } else {
      acceptMessage.value = result.data?.message || "接单失败，请重试";
      acceptError.value = true;
    }
  } catch {
    acceptMessage.value = "接单失败，请重试";
    acceptError.value = true;
  }

  window.setTimeout(() => {
    acceptMessage.value = "";
    acceptError.value = false;
  }, 3000);
}

async function handleCancelOrder(orderId) {
  const token = getVeteranToken();
  if (!token) return;

  try {
    const result = await cancelOrder(orderId, token);
    if (result.ok && result.data?.success) {
      await refreshPortalData();
    }
  } catch {
    // keep current UI state when cancel fails
  }
}

async function handleUpdateRegion(payload) {
  const token = getVeteranToken();
  const regionCode = payload?.regionCode?.trim();
  const regionName = payload?.regionName?.trim();

  if (!token || !regionCode) {
    return;
  }

  if (
    regionCode === veteranProfile.value?.region_code &&
    (!regionName || regionName === veteranProfile.value?.region_name)
  ) {
    return;
  }

  acceptMessage.value = "";
  acceptError.value = false;

  try {
    const result = await updateVeteranRegion({ region_code: regionCode, region_name: regionName }, token);
    if (result.ok && result.data?.success) {
      closeOrdersWs();
      await refreshPortalData();
      acceptMessage.value = "服务区域已更新";
    } else {
      acceptMessage.value = result.data?.message || "服务区域更新失败，请重试";
      acceptError.value = true;
    }
  } catch {
    acceptMessage.value = "服务区域更新失败，请重试";
    acceptError.value = true;
  }

  window.setTimeout(() => {
    acceptMessage.value = "";
    acceptError.value = false;
  }, 3000);
}

watch(
  () => isLoggedIn.value,
  (loggedIn) => {
    if (!loggedIn) {
      closeOrdersWs();
      orders.value = [];
      assignedOrders.value = [];
      showOrderDetail.value = null;
      veteranStats.value = { ...DEFAULT_STATS };
      return;
    }

    connectOrdersWs();
    void loadPortalData();
  },
  { immediate: true },
);

watch(
  () => veteranProfile.value?.region_code || "",
  (nextRegionCode, previousRegionCode) => {
    if (!isLoggedIn.value || !nextRegionCode || nextRegionCode === previousRegionCode) {
      return;
    }

    orders.value = [];
    closeOrdersWs();
    connectOrdersWs(nextRegionCode);
    void loadAvailableOrders();
  },
);

onBeforeUnmount(() => {
  closeOrdersWs();
});

const regionLabelMap = {
  "sh-pudong": "上海市 / 上海市 / 浦东新区",
  "sh-minhang": "上海市 / 上海市 / 闵行区",
  Pudong: "上海市 / 上海市 / 浦东新区",
  Minhang: "上海市 / 上海市 / 闵行区",
};
const serviceStatusLabelMap = {
  available: "可接单",
  busy: "服务中",
  paused: "暂停服务",
  pending: "待排班",
};
const veteranName = computed(() => veteranProfile.value?.name || "退役军人服务者");
const displayRegion = computed(
  () =>
    regionLabelMap[veteranProfile.value?.region_name] ||
    veteranProfile.value?.region_name ||
    regionLabelMap[veteranProfile.value?.region_code] ||
    "待配置服务区域",
);
const serviceStatusText = computed(
  () => serviceStatusLabelMap[veteranProfile.value?.service_status] || "待准备",
);
const maskedPhone = computed(() => {
  const phone = (veteranProfile.value?.phone || credentials.phone || "").trim();
  if (phone.length !== 11) return phone || "待补充手机号";
  return `${phone.slice(0, 3)} ${phone.slice(3, 7)} ${phone.slice(7)}`;
});
const projectCards = computed(() =>
  availableProjects.map((project) => ({
    ...project,
    selected: selectedProjects.value.includes(project.id),
  })),
);
const moduleCards = computed(() =>
  trainingModules.map((module) => ({
    ...module,
    status: "待开始",
    state: "pending",
  })),
);
const profileSummary = computed(() => [
  { label: "姓名", value: veteranName.value },
  { label: "绑定手机号", value: maskedPhone.value },
  { label: "服务区域", value: displayRegion.value },
  { label: "当前状态", value: serviceStatusText.value },
]);

function go(path) {
  emit("navigate", path);
}
</script>

<template>
  <div class="site-shell portal-shell">
    <main v-if="isBootstrapping" class="portal-login">
      <section class="portal-panel">
        <p class="section-kicker">Xia Ban Xing</p>
        <h1>正在进入侠伴行</h1>
        <p>检测到本地登录状态，正在校验你的服务者身份并恢复工作台。</p>
      </section>
    </main>

    <main v-else-if="!isLoggedIn" class="portal-login">
      <section class="portal-panel">
        <p class="section-kicker">Xia Ban Xing</p>
        <h1>侠伴行</h1>

        <form class="login-form" @submit.prevent="login">
          <label>
            <span>手机号</span>
            <input
              v-model="credentials.phone"
              type="tel"
              inputmode="numeric"
              maxlength="11"
              placeholder="请输入手机号"
              required
            />
          </label>

          <div class="sms-field">
            <label>
              <span>短信验证码</span>
              <input
                v-model="credentials.smsCode"
                type="text"
                inputmode="numeric"
                maxlength="6"
                placeholder="请输入验证码"
                required
              />
            </label>

            <button
              class="button ghost"
              type="button"
              :disabled="isSendingCode || remainingSeconds > 0"
              @click="sendSmsCode"
            >
              {{ isSendingCode ? "发送中..." : remainingSeconds > 0 ? `${remainingSeconds}秒后重试` : "获取验证码" }}
            </button>
          </div>

          <p v-if="loginMessage" class="form-note success">{{ loginMessage }}</p>
          <p v-if="loginError" class="form-note error">{{ loginError }}</p>

          <button class="button primary wide" type="submit" :disabled="isLoggingIn">
            {{ isLoggingIn ? "登录中..." : "登录侠伴行" }}
          </button>
        </form>
      </section>
    </main>

    <main v-else-if="!veteranProfile" class="portal-login">
      <section class="portal-panel">
        <p class="section-kicker">Xia Ban Xing</p>
        <h1>工作台加载失败</h1>
        <p>登录态已存在，但首页信息暂时没有恢复成功。</p>
        <p v-if="loginError" class="form-note error">{{ loginError }}</p>

        <div class="form-actions">
          <button class="button primary" type="button" @click="refreshPortalData">重新加载</button>
          <button class="button ghost" type="button" @click="logout">退出登录</button>
        </div>
      </section>
    </main>

    <main v-else class="vb-app-shell">
      <div v-if="acceptMessage" class="vb-toast" :class="{ error: acceptError }">{{ acceptMessage }}</div>

      <component
        :is="activeTabComponent"
        :veteran-name="veteranName"
        :display-region="displayRegion"
        :service-status-text="serviceStatusText"
        :masked-phone="maskedPhone"
        :project-cards="projectCards"
        :orders="orders"
        :ws-connected="wsConnected"
        :module-cards="moduleCards"
        :completed-count="completedCount"
        :total-modules="trainingModules.length"
        :profile-summary="profileSummary"
        :veteran-card-number="veteranProfile?.veteran_card_number || ''"
        :region-code="veteranProfile?.region_code || ''"
        :region-name="veteranProfile?.region_name || ''"
        :assigned-orders="assignedOrders"
        :veteran-stats="veteranStats"
        @accept-order="handleAcceptOrder"
        @toggle-project="toggleProject"
        @cancel-order="handleCancelOrder"
        @logout="logout"
        @view-detail="(order) => (showOrderDetail = order)"
        @update-region="handleUpdateRegion"
      />

      <nav class="vb-tabbar">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          type="button"
          :class="{ active: activeTab === tab.key }"
          @click="switchTab(tab.key)"
        >
          {{ tab.label }}
        </button>
      </nav>

      <div v-if="showOrderDetail" class="vb-modal-overlay" @click.self="showOrderDetail = null">
        <div class="vb-modal">
          <div class="vb-modal-head">
            <h2>订单详情</h2>
            <button class="vb-modal-close" type="button" @click="showOrderDetail = null">×</button>
          </div>

          <div class="vb-modal-body">
            <div class="vb-detail-item">
              <span>订单号</span>
              <strong>{{ showOrderDetail.order_no }}</strong>
            </div>
            <div class="vb-detail-item">
              <span>服务项目</span>
              <strong>{{ showOrderDetail.service_item_name }}</strong>
            </div>
            <div class="vb-detail-item">
              <span>状态</span>
              <strong>{{ showOrderDetail.status_label }}</strong>
            </div>
            <div class="vb-detail-item">
              <span>预约时间</span>
              <strong>{{ showOrderDetail.service_date }} {{ showOrderDetail.service_time_slot }}</strong>
            </div>
            <div class="vb-detail-item">
              <span>服务地址</span>
              <strong>{{ showOrderDetail.service_address }}</strong>
            </div>

            <div class="vb-detail-divider"></div>
            <p class="vb-kicker">下单人信息</p>

            <div class="vb-detail-item">
              <span>姓名</span>
              <strong>{{ showOrderDetail.contact_name }}</strong>
            </div>
            <div class="vb-detail-item">
              <span>电话</span>
              <strong>{{ showOrderDetail.contact_phone }}</strong>
            </div>
            <div class="vb-detail-item">
              <span>备注</span>
              <strong>{{ showOrderDetail.note || "无" }}</strong>
            </div>

            <div class="vb-detail-divider"></div>
            <div class="vb-detail-item">
              <span>派单信息</span>
              <strong>{{ showOrderDetail.dispatch_message }}</strong>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>
