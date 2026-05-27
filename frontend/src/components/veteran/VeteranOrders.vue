<script setup>
import { computed, ref } from "vue";

const props = defineProps({
  assignedOrders: { type: Array, default: () => [] },
});

const emit = defineEmits(["view-detail", "cancel-order"]);

const activeTab = ref("active");

const tabs = [
  { key: "active", label: "进行中" },
  { key: "history", label: "历史" },
];

const activeOrders = computed(() =>
  props.assignedOrders.filter((order) => order.status !== "completed"),
);

const historyOrders = computed(() =>
  props.assignedOrders.filter((order) => order.status === "completed"),
);

const displayOrders = computed(() =>
  activeTab.value === "active" ? activeOrders.value : historyOrders.value,
);

const activeCount = computed(() => activeOrders.value.length);
const historyCount = computed(() => historyOrders.value.length);

const showCancelConfirm = ref(null);

function confirmCancel(orderId) {
  showCancelConfirm.value = orderId;
}

function doCancel(orderId) {
  showCancelConfirm.value = null;
  emit("cancel-order", orderId);
}
</script>

<template>
  <div class="vb-screen">
    <div class="vb-order-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        class="vb-order-tab"
        :class="{ active: activeTab === tab.key }"
        type="button"
        @click="activeTab = tab.key"
      >
        {{ tab.label }}
        <span class="vb-order-tab-count">{{ tab.key === "active" ? activeCount : historyCount }}</span>
      </button>
    </div>

    <section class="vb-section-card">
      <div v-if="displayOrders.length === 0" class="vb-empty">
        <p v-if="activeTab === 'active'" class="vb-empty-title">暂无进行中的订单</p>
        <p v-else class="vb-empty-title">暂无历史订单</p>
      </div>
      <div v-else class="vb-order-list">
        <article
          v-for="order in displayOrders"
          :key="order.id"
          class="vb-order-card"
          :class="{ 'is-active': order.status !== 'completed' }"
          @click="emit('view-detail', order)"
        >
          <div class="vb-order-head">
            <strong>{{ order.service_item_name }}</strong>
            <span class="vb-tag" :data-status="order.status">{{ order.status_label }}</span>
          </div>
          <div class="vb-order-body">
            <p class="vb-order-schedule">
              <span>{{ order.service_date }}</span>
              <span class="vb-order-divider">·</span>
              <span>{{ order.service_time_slot }}</span>
            </p>
            <p class="vb-order-address">{{ order.service_address }}</p>
            <p class="vb-order-contact">{{ order.contact_name }} · {{ order.contact_phone }}</p>
          </div>
          <div class="vb-order-foot" :class="{ 'no-note': !order.dispatch_message }">
            <span v-if="order.dispatch_message" class="vb-order-note">{{ order.dispatch_message }}</span>
            <button
              v-if="order.status !== 'completed'"
              class="vb-cancel-btn"
              type="button"
              @click.stop="confirmCancel(order.id)"
            >
              取消订单
            </button>
          </div>
        </article>
      </div>
    </section>

    <div v-if="showCancelConfirm" class="vb-modal-overlay" @click.self="showCancelConfirm = null">
      <div class="vb-modal" style="max-width: 340px;">
        <div class="vb-modal-head">
          <h2>确认取消</h2>
          <button class="vb-modal-close" type="button" @click="showCancelConfirm = null">×</button>
        </div>
        <div class="vb-modal-body">
          <p style="margin:0; font-size:.88rem; color:var(--text-muted); line-height:1.5;">
            取消后该订单将重新回到待接单列表，其他服务者可以继续接单。确认取消吗？
          </p>
          <div style="display:flex; gap:10px; margin-top:12px;">
            <button class="button ghost" type="button" @click="showCancelConfirm = null" style="flex:1;">再想想</button>
            <button class="button primary" type="button" @click="doCancel(showCancelConfirm)" style="flex:1; background:var(--accent);">确认取消</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
