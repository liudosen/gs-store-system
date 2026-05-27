<script setup>
import { computed, onMounted, ref } from "vue";
import CustomerAddressFormPanel from "../components/customer/CustomerAddressFormPanel.vue";
import CustomerAppShell from "../components/customer/CustomerAppShell.vue";
import {
  persistSelectedCustomerService,
  useCustomerApp,
} from "../features/customer/useCustomerApp";
import { CUSTOMER_SELECTED_SERVICE_STORAGE_KEY } from "../shared/constants/storage";

const emit = defineEmits(["navigate"]);

const {
  addresses,
  addressForm,
  bootstrap,
  fetchMoreOrders,
  fetchOrders,
  loading,
  loginForm,
  logout,
  messages,
  orderForm,
  orders,
  orderCategory,
  orderHasMore,
  profile,
  saveAddress,
  selectedOrder,
  selectedService,
  selectService,
  startAddressCreate,
  startAddressEdit,
  submitOrder,
  timeSlots,
  pickOrder,
  services,
} = useCustomerApp();

const activeTab = ref("services");
const showBookingSheet = ref(false);
const showInlineAddressForm = ref(false);
const showAddressPicker = ref(false);
const showOrderDetail = ref(false);
const showAddressFormInProfile = ref(false);
const orderView = ref("active");

const hasAddressOptions = computed(() => addresses.value.length > 0);
const selectedAddress = computed(
  () => addresses.value.find((address) => address.id === Number(orderForm.addressId)) || null,
);
const displayRegion = computed(() => profile.value?.selected_region_name || "未选择");

const portalTabs = [
  { key: "services", label: "服务", path: "/xiadaojia" },
  { key: "orders", label: "订单", path: "/xiadaojia" },
  { key: "profile", label: "我的", path: "/xiadaojia" },
];

const activeTitle = computed(() => {
  if (activeTab.value === "orders") return "订单";
  if (activeTab.value === "profile") return "我的";
  return "服务";
});

onMounted(async () => {
  await bootstrap();
  await fetchOrders(orderView.value);
  const pendingServiceId = Number(sessionStorage.getItem(CUSTOMER_SELECTED_SERVICE_STORAGE_KEY) || 0);
  if (pendingServiceId) {
    handleOpenBooking(pendingServiceId);
    sessionStorage.removeItem(CUSTOMER_SELECTED_SERVICE_STORAGE_KEY);
  }
});

function changeTab(tabKey) {
  activeTab.value = tabKey;
  if (tabKey === "orders") {
    void fetchOrders(orderView.value);
  }
}

function handleOpenBooking(serviceId) {
  activeTab.value = "services";
  selectService(serviceId);
  persistSelectedCustomerService(serviceId);
  if (!addresses.value.length) {
    startAddressCreate();
    showInlineAddressForm.value = true;
  } else {
    showInlineAddressForm.value = false;
  }
  showBookingSheet.value = true;
}

function openAddressPicker() {
  if (!hasAddressOptions.value) {
    startAddressCreate();
    showInlineAddressForm.value = true;
    return;
  }

  showAddressPicker.value = true;
}

function chooseAddress(addressId) {
  orderForm.addressId = addressId;
  showAddressPicker.value = false;
}

async function submitAndGoOrders() {
  const success = await submitOrder();
  if (success) {
    showBookingSheet.value = false;
    activeTab.value = "orders";
    orderView.value = "active";
    await fetchOrders("active");
  }
}

async function submitInlineAddress() {
  const success = await saveAddress();
  if (success) {
    showInlineAddressForm.value = false;
    if (addresses.value.length) {
      orderForm.addressId = addresses.value[0].id;
    }
  }
}

function openOrderDetail(orderId) {
  pickOrder(orderId);
  showOrderDetail.value = true;
}

async function changeOrderView(view) {
  orderView.value = view;
  await fetchOrders(view);
}

async function handleOrderListScroll(event) {
  const element = event.target;
  if (!orderHasMore.value || loading.loadingMoreOrders) {
    return;
  }

  const nearBottom = element.scrollTop + element.clientHeight >= element.scrollHeight - 80;
  if (nearBottom) {
    await fetchMoreOrders();
  }
}

function createNewAddress() {
  startAddressCreate();
  showAddressFormInProfile.value = true;
}

function editAddress(address) {
  startAddressEdit(address);
  showAddressFormInProfile.value = true;
}

async function submitAddressFromProfile() {
  const success = await saveAddress();
  if (success) {
    showAddressFormInProfile.value = false;
  }
}

function handleLogout() {
  logout();
  emit("navigate", "/xia-dao-jia/login");
}
</script>

<template>
  <CustomerAppShell
    current-path="/xiadaojia"
    :title="activeTitle"
    kicker="侠到家"
    :side-value="''"
    :tabs="portalTabs"
    :active-tab="activeTab"
    :use-internal-tabs="true"
    @navigate="emit('navigate', $event)"
    @tab-change="changeTab"
  >
    <template v-if="activeTab === 'services'">
      <section class="xjd-section-card">
        <div class="xjd-section-head">
          <div>
            <p class="xjd-section-kicker">服务列表</p>
            <h3>选择需要预约的到家服务</h3>
          </div>
        </div>

        <div v-if="services.length > 0" class="xjd-service-list">
          <button
            v-for="service in services"
            :key="service.id"
            class="xjd-service-row"
            type="button"
            @click="handleOpenBooking(service.id)"
          >
            <div class="xjd-service-row-main">
              <span class="xjd-service-row-badge">{{ service.badge }}</span>
              <strong>{{ service.name }}</strong>
              <p>{{ service.short_description }}</p>
            </div>
            <div class="xjd-service-row-side">
              <b>¥{{ service.base_price }}</b>
              <small>{{ service.duration_minutes }} 分钟</small>
            </div>
          </button>
        </div>

        <div v-else class="xjd-empty-card">
          <strong>暂无可预约服务</strong>
          <p>请稍后再试或联系管理员补充服务目录。</p>
        </div>
      </section>
    </template>

    <template v-else-if="activeTab === 'orders'">
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
              <p class="xjd-section-kicker">订单分类</p>
              <h3>查看不同状态的订单</h3>
            </div>
          </div>

          <div class="vb-order-tabs">
            <button class="vb-order-tab" :class="{ active: orderView === 'active' }" type="button" @click="changeOrderView('active')">
              当前服务
            </button>
            <button class="vb-order-tab" :class="{ active: orderView === 'matching' }" type="button" @click="changeOrderView('matching')">
              待接单
            </button>
            <button class="vb-order-tab" :class="{ active: orderView === 'history' }" type="button" @click="changeOrderView('history')">
              历史
            </button>
          </div>

          <div v-if="orders.length" class="xjd-order-feed xjd-order-scroll" @scroll.passive="handleOrderListScroll">
            <button
              v-for="order in orders"
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

            <div v-if="loading.loadingMoreOrders" class="xjd-empty-inline">
              <p>正在加载更多订单…</p>
            </div>
            <div v-else-if="orders.length > 0 && !orderHasMore" class="xjd-empty-inline">
              <p>没有更多订单了</p>
            </div>
          </div>

          <div v-else class="xjd-empty-card">
            <strong>当前分类暂无订单</strong>
            <p>切换到其他分类继续查看，或返回服务页重新预约。</p>
          </div>
        </section>
      </template>
    </template>

    <template v-else>
      <section class="xjd-section-card xjd-profile-card">
        <div class="xjd-profile-avatar">侠</div>
        <div class="xjd-profile-copy">
          <div class="xjd-profile-inline">
            <span>账号</span>
            <strong>{{ profile?.phone || "未登录" }}</strong>
          </div>
        </div>
      </section>

      <section class="xjd-section-card">
        <div class="xjd-section-head">
          <div>
            <p class="xjd-section-kicker">个人信息</p>
            <h3>当前账号资料</h3>
          </div>
        </div>

        <div class="xjd-detail-stack">
          <article class="xjd-detail-card">
            <h3>手机号</h3>
            <p>{{ profile?.phone || "-" }}</p>
          </article>
        </div>
      </section>

      <section class="xjd-section-card">
        <div class="xjd-section-head">
          <div>
            <p class="xjd-section-kicker">地址管理</p>
            <h3>维护上门服务地址</h3>
          </div>
          <button class="button ghost" type="button" @click="createNewAddress">新增地址</button>
        </div>

        <div v-if="addresses.length" class="xjd-address-stack">
          <button
            v-for="address in addresses"
            :key="address.id"
            class="xjd-address-sheet"
            type="button"
            @click="editAddress(address)"
          >
            <div class="xjd-address-sheet-head">
              <strong>{{ address.contact_name }}</strong>
              <span v-if="address.is_default" class="xjd-status-pill">默认地址</span>
            </div>
            <p>{{ address.contact_phone }}</p>
            <p>{{ address.city_name }}{{ address.district_name }}{{ address.detail_address }}</p>
          </button>
        </div>

        <div v-else class="xjd-empty-card">
          <strong>暂无地址</strong>
          <p>新增一个常用地址，预约时可以直接选择。</p>
        </div>
      </section>

      <section v-if="showAddressFormInProfile" class="xjd-section-card">
        <CustomerAddressFormPanel
          :form="addressForm"
          :loading="loading.savingAddress"
          :title="addressForm.id ? '编辑地址' : '新增地址'"
          :success-message="messages.appSuccess"
          :error-message="messages.appError"
          :show-cancel="true"
          @submit="submitAddressFromProfile"
          @cancel="showAddressFormInProfile = false"
        />
      </section>

      <section class="xjd-section-card">
        <button class="button ghost" type="button" @click="handleLogout">退出登录</button>
      </section>
    </template>

    <div v-if="showBookingSheet" class="vb-modal-overlay" @click.self="showBookingSheet = false">
      <div class="vb-modal">
        <div class="vb-modal-head">
          <h2>{{ selectedService ? selectedService.name : "预约服务" }}</h2>
          <button class="vb-modal-close" type="button" @click="showBookingSheet = false">×</button>
        </div>

        <div class="vb-modal-body">
          <div v-if="selectedService" class="xjd-selection-summary">
            <strong>{{ selectedService.name }}</strong>
            <div class="xjd-selection-meta">
              <span>{{ selectedService.duration_minutes }} 分钟</span>
              <span>¥{{ selectedService.base_price }}</span>
            </div>
          </div>

          <CustomerAddressFormPanel
            v-if="showInlineAddressForm"
            :form="addressForm"
            :loading="loading.savingAddress"
            title="新增服务地址"
            submit-label="保存并继续下单"
            submit-busy-label="保存中..."
            :success-message="messages.appSuccess"
            :error-message="messages.appError"
            :show-cancel="addresses.length > 0"
            @submit="submitInlineAddress"
            @cancel="showInlineAddressForm = false"
          />

          <form v-else class="xjd-form" @submit.prevent="submitAndGoOrders">
            <label class="xjd-field">
              <span>服务日期</span>
              <input v-model="orderForm.serviceDate" type="date" required />
            </label>

            <label class="xjd-field">
              <span>服务时段</span>
              <select v-model="orderForm.serviceTimeSlot" required>
                <option v-for="slot in timeSlots" :key="slot" :value="slot">{{ slot }}</option>
              </select>
            </label>

            <label class="xjd-field">
              <span>服务地址</span>
              <button class="xjd-link-row" type="button" @click="openAddressPicker">
                <div>
                  <strong v-if="selectedAddress">
                    {{ selectedAddress.contact_name }}｜{{ selectedAddress.city_name }}{{ selectedAddress.district_name }}{{ selectedAddress.detail_address }}
                  </strong>
                  <strong v-else>请选择地址</strong>
                </div>
                <span>选择</span>
              </button>
            </label>

            <button class="button ghost wide" type="button" @click="goToAddressManager">
              {{ hasAddressOptions ? "管理地址 / 新增地址" : "新增地址" }}
            </button>

            <label class="xjd-field">
              <span>备注</span>
              <textarea v-model="orderForm.note" rows="4" placeholder="请输入备注" />
            </label>

            <p v-if="!hasAddressOptions" class="form-note error">请先添加地址后再下单</p>
            <p v-if="messages.appSuccess" class="form-note success">{{ messages.appSuccess }}</p>
            <p v-if="messages.appError" class="form-note error">{{ messages.appError }}</p>

            <button class="button primary wide" type="submit" :disabled="loading.creatingOrder || !selectedService || !hasAddressOptions">
              {{ loading.creatingOrder ? "提交中..." : "确认下单" }}
            </button>
          </form>
        </div>
      </div>
    </div>

    <div v-if="showAddressPicker" class="vb-modal-overlay" @click.self="showAddressPicker = false">
      <div class="vb-modal">
        <div class="vb-modal-head">
          <h2>选择服务地址</h2>
          <button class="vb-modal-close" type="button" @click="showAddressPicker = false">×</button>
        </div>

        <div class="vb-modal-body">
          <div v-if="addresses.length" class="xjd-address-stack">
            <button
              v-for="address in addresses"
              :key="address.id"
              class="xjd-address-sheet"
              type="button"
              @click="chooseAddress(address.id)"
            >
              <div class="xjd-address-sheet-head">
                <strong>{{ address.contact_name }}</strong>
                <span v-if="address.is_default" class="xjd-status-pill">默认地址</span>
              </div>
              <p>{{ address.contact_phone }}</p>
              <p>{{ address.city_name }}{{ address.district_name }}{{ address.detail_address }}</p>
            </button>
          </div>

          <button class="button ghost wide" type="button" @click="goToAddressManager">
            新增地址
          </button>
        </div>
      </div>
    </div>

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
