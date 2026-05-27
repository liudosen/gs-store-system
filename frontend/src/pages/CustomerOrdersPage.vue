<script setup>
import { computed, onMounted, ref } from "vue";
import CustomerAppShell from "../components/customer/CustomerAppShell.vue";
import { useCustomerApp } from "../features/customer/useCustomerApp";

const props = defineProps({
  currentPath: {
    type: String,
    default: "",
  },
});

const emit = defineEmits(["navigate"]);
const { bootstrap, fetchOrders, loading, messages, orders, pickOrder, selectedOrder } = useCustomerApp();

const activeOrders = computed(() =>
  orders.value.filter((order) => order.status !== "completed"),
);
const historyOrders = computed(() =>
  orders.value.filter((order) => order.status === "completed"),
);
const showOrderDetail = ref(false);

onMounted(() => {
  bootstrap();
});

function openOrderDetail(orderId) {
  pickOrder(orderId);
  showOrderDetail.value = true;
}
</script>

<template>
  <CustomerAppShell
    :current-path="props.currentPath"
    title="订单"
    kicker="侠到家"
    @navigate="emit('navigate', $event)"
  >
    <section v-if="loading.bootstrap || loading.loadingOrders" class="xjd-section-card">
      <div class="xjd-empty-card">
        <strong>加载中…</strong>
      </div>
    </section>

    <section v-else-if="messages.appError" class="xjd-section-card">
      <div class="xjd-empty-card">
        <strong style="color: var(--xjd-accent)">订单加载失败</strong>
        <p style="color: var(--xjd-accent)">{{ messages.appError }}</p>
        <button class="button primary" type="button" @click="fetchOrders">重试</button>
      </div>
    </section>

    <template v-else>
      <section class="xjd-section-card">
        <div class="xjd-section-head">
          <div>
            <p class="xjd-section-kicker">进行中</p>
            <h3>当前服务订单</h3>
          </div>
        </div>

        <div v-if="activeOrders.length" class="xjd-order-feed">
          <button
            v-for="order in activeOrders"
            :key="order.id"
            class="xjd-order-feed-card"
            type="button"
            @click="openOrderDetail(order.id)"
          >
            <div class="xjd-order-feed-head">
              <strong>{{ order.service_item_name }}</strong>
              <span class="xjd-status-pill">{{ order.status_label }}</span>
            </div>
            <p>{{ order.service_date }} {{ order.service_time_slot }}</p>
            <p>{{ order.service_address }}</p>
          </button>
        </div>

        <div v-else class="xjd-empty-card">
          <strong>暂无进行中的订单</strong>
          <p>新的预约成功后会出现在这里。</p>
        </div>
      </section>

      <section class="xjd-section-card">
        <div class="xjd-section-head">
          <div>
            <p class="xjd-section-kicker">历史订单</p>
            <h3>已完成服务记录</h3>
          </div>
          <button class="button ghost" type="button" @click="emit('navigate', '/xia-dao-jia/services')">再次预约</button>
        </div>

        <div v-if="historyOrders.length" class="xjd-order-feed">
          <button
            v-for="order in historyOrders"
            :key="order.id"
            class="xjd-order-feed-card"
            type="button"
            @click="openOrderDetail(order.id)"
          >
            <div class="xjd-order-feed-head">
              <strong>{{ order.service_item_name }}</strong>
              <span class="xjd-status-pill">{{ order.status_label }}</span>
            </div>
            <p>{{ order.service_date }} {{ order.service_time_slot }}</p>
            <p>{{ order.service_address }}</p>
          </button>
        </div>

        <div v-else class="xjd-empty-card">
          <strong>暂无历史订单</strong>
          <p>完成服务后会沉淀在这里，方便回看。</p>
        </div>
      </section>

    </template>

    <div v-if="showOrderDetail && selectedOrder" class="vb-modal-overlay" @click.self="showOrderDetail = false">
      <div class="vb-modal">
        <div class="vb-modal-head">
          <h2>{{ selectedOrder.service_item_name }}</h2>
          <button class="vb-modal-close" type="button" @click="showOrderDetail = false">×</button>
        </div>

        <div class="vb-modal-body">
          <article class="xjd-detail-card">
            <div class="xjd-detail-head">
              <span class="xjd-status-pill">{{ selectedOrder.status_label }}</span>
              <strong>订单号：{{ selectedOrder.order_no }}</strong>
            </div>
            <p>{{ selectedOrder.dispatch_message }}</p>
          </article>

          <article class="xjd-detail-card">
            <h3>订单信息</h3>
            <p>服务时间：{{ selectedOrder.service_date }} {{ selectedOrder.service_time_slot }}</p>
            <p>联系人：{{ selectedOrder.contact_name }} {{ selectedOrder.contact_phone }}</p>
            <p>服务地址：{{ selectedOrder.service_address }}</p>
            <p>备注：{{ selectedOrder.note || "-" }}</p>
          </article>

          <article class="xjd-detail-card">
            <h3>上门服务退役军人</h3>
            <p>姓名：{{ selectedOrder.assigned_veteran_name || "待分配" }}</p>
            <p>联系电话：{{ selectedOrder.assigned_veteran_phone || "-" }}</p>
          </article>
        </div>
      </div>
    </div>
  </CustomerAppShell>
</template>
