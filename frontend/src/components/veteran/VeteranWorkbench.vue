<script setup>
import { computed } from "vue";

const serviceItemIdToProjectIdMap = {
  1: "escort-medical",
  2: "home-companion",
  3: "home-cleaning",
  4: "meal-delivery",
  5: "community-support",
};

const props = defineProps({
  veteranName: { type: String, required: true },
  displayRegion: { type: String, required: true },
  serviceStatusText: { type: String, required: true },
  maskedPhone: { type: String, required: true },
  projectCards: { type: Array, default: () => [] },
  orders: { type: Array, default: () => [] },
  wsConnected: { type: Boolean, default: false },
  assignedOrders: { type: Array, default: () => [] },
  completedCount: { type: Number, default: 0 },
  totalModules: { type: Number, default: 0 },
  profileSummary: { type: Array, default: () => [] },
  veteranStats: { type: Object, default: () => ({ today_orders: 0, today_completed: 0, month_orders: 0, rating_score: 0.0 }) },
});

const emit = defineEmits(["accept-order", "toggle-project"]);

const filteredOrders = computed(() => {
  const selectedIds = props.projectCards.filter((p) => p.selected).map((p) => p.id);
  if (selectedIds.length === 0) return props.orders;

  return props.orders.filter((order) =>
    selectedIds.includes(serviceItemIdToProjectIdMap[order.service_item_id]),
  );
});

const upcomingOrder = computed(() => {
  if (!props.assignedOrders || props.assignedOrders.length === 0) return null;
  const active = props.assignedOrders.filter(o => o.status !== 'completed');
  if (active.length === 0) return null;
  return active[0];
});

const trainingDone = computed(() => props.completedCount >= props.totalModules && props.totalModules > 0);
</script>

<template>
  <div class="vb-screen">

    <!-- 服务者状态卡 -->
    <section class="vb-status-card">
      <div class="vb-status-top">
        <div class="vb-status-avatar">{{ veteranName.slice(0, 1) }}</div>
        <div class="vb-status-info">
          <strong>{{ veteranName }}</strong>
          <span>{{ displayRegion }}</span>
        </div>
        <span class="vb-status-badge" :data-status="serviceStatusText">{{ serviceStatusText }}</span>
      </div>
      <div class="vb-status-divider"></div>
      <div class="vb-status-stats">
        <div class="vb-status-stat">
          <span>今日</span>
          <strong>{{ veteranStats.today_orders }}</strong>
        </div>
        <div class="vb-status-stat">
          <span>本月</span>
          <strong>{{ veteranStats.month_orders }}</strong>
        </div>
        <div class="vb-status-stat">
          <span>评分</span>
          <strong>{{ veteranStats.rating_score ? veteranStats.rating_score.toFixed(1) : '--' }}</strong>
        </div>
      </div>
    </section>

    <!-- 待服务提醒 -->
    <section v-if="upcomingOrder" class="vb-reminder-card">
      <span class="vb-reminder-dot"></span>
      <div class="vb-reminder-body">
        <p class="vb-reminder-kicker">即将开始的服务</p>
        <strong>{{ upcomingOrder.service_item_name }}&ensp;·&ensp;{{ upcomingOrder.service_date }} {{ upcomingOrder.service_time_slot }}</strong>
      </div>
    </section>

    <!-- 培训未完成横幅 -->
    <div v-if="!trainingDone" class="vb-guide-banner">
      <span>完成培训后即可接单（{{ completedCount }}/{{ totalModules }}）</span>
    </div>

    <!-- 待接订单 -->
    <section class="vb-section-card">
      <div class="vb-section-head">
        <div class="vb-section-head-left">
          <p class="vb-kicker">待接订单</p>
          <h2>{{ displayRegion }}</h2>
        </div>
        <div class="vb-ws-status" :class="{ on: wsConnected }">
          <span class="vb-ws-dot"></span>
          <span>{{ wsConnected ? '在线' : '离线' }}</span>
        </div>
      </div>

      <div class="vb-filter-pills">
        <button
          v-for="project in projectCards"
          :key="project.id"
          class="vb-filter-pill"
          :class="{ active: project.selected }"
          type="button"
          @click="emit('toggle-project', project.id)"
        >
          {{ project.title }}
        </button>
      </div>

      <div v-if="!wsConnected && orders.length === 0" class="vb-empty">
        <div class="vb-empty-illustration">
          <svg width="48" height="48" viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="6" y="12" width="36" height="26" rx="4"/>
            <path d="M6 16 L24 28 L42 16"/>
            <line x1="24" y1="22" x2="24" y2="32" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </div>
        <p class="vb-empty-title">正在连接服务…</p>
        <p class="vb-empty-desc">请稍候，新订单将自动出现</p>
      </div>
      <div v-else-if="filteredOrders.length === 0" class="vb-empty">
        <div class="vb-empty-illustration">
          <svg width="48" height="48" viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="6" y="12" width="36" height="26" rx="4"/>
            <path d="M6 16 L24 28 L42 16"/>
          </svg>
        </div>
        <p class="vb-empty-title">暂无待接订单</p>
        <p class="vb-empty-desc">新订单将实时推送，请保持连线</p>
      </div>
      <div v-else class="vb-order-list">
        <article v-for="order in filteredOrders" :key="order.id" class="vb-order-card">
          <div class="vb-order-head">
            <strong>{{ order.service_item_name }}</strong>
            <span class="vb-order-no">{{ order.order_no }}</span>
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
           <div class="vb-order-foot" :class="{ 'no-note': !order.note }">
             <span v-if="order.note" class="vb-order-note">{{ order.note }}</span>
             <button
               class="button primary small"
               type="button"
               @click="emit('accept-order', order.id)"
             >
               接单
             </button>
           </div>
        </article>
      </div>
    </section>

  </div>
</template>
