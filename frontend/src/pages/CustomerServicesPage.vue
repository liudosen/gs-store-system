<script setup>
import { computed, onMounted, ref } from "vue";
import CustomerAddressFormPanel from "../components/customer/CustomerAddressFormPanel.vue";
import CustomerAppShell from "../components/customer/CustomerAppShell.vue";
import {
  persistSelectedCustomerService,
  useCustomerApp,
} from "../features/customer/useCustomerApp";
import { CUSTOMER_SELECTED_SERVICE_STORAGE_KEY } from "../shared/constants/storage";

const props = defineProps({
  currentPath: {
    type: String,
    default: "",
  },
});

const emit = defineEmits(["navigate"]);
const {
  addresses,
  addressForm,
  bootstrap,
  loading,
  messages,
  orderForm,
  saveAddress,
  selectedService,
  selectService,
  startAddressCreate,
  services,
  submitOrder,
  timeSlots,
} = useCustomerApp();

const showBookingSheet = ref(false);
const showInlineAddressForm = ref(false);
const showAddressPicker = ref(false);
const filteredServices = computed(() =>
  services.value.filter((service) => service.code !== "community-support"),
);
const hasAddressOptions = computed(() => addresses.value.length > 0);
const selectedAddress = computed(
  () => addresses.value.find((address) => address.id === Number(orderForm.addressId)) || null,
);

onMounted(async () => {
  await bootstrap();
  const pendingServiceId = Number(sessionStorage.getItem(CUSTOMER_SELECTED_SERVICE_STORAGE_KEY) || 0);
  if (pendingServiceId) {
    handleOpenBooking(pendingServiceId);
    sessionStorage.removeItem(CUSTOMER_SELECTED_SERVICE_STORAGE_KEY);
  }
});

function handleOpenBooking(serviceId) {
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

async function submitAndGoOrders() {
  const success = await submitOrder();
  if (success) {
    showBookingSheet.value = false;
    emit("navigate", "/xia-dao-jia/orders");
  }
}

function goToAddressManager() {
  startAddressCreate();
  showInlineAddressForm.value = true;
}

function openAddressPicker() {
  if (!hasAddressOptions.value) {
    goToAddressManager();
    return;
  }

  showAddressPicker.value = true;
}

function chooseAddress(addressId) {
  orderForm.addressId = addressId;
  showAddressPicker.value = false;
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
</script>

<template>
  <CustomerAppShell
    :current-path="props.currentPath"
    title="服务"
    kicker="侠到家"
    @navigate="emit('navigate', $event)"
  >
    <section class="xjd-section-card">
      <div class="xjd-section-head">
        <div>
          <p class="xjd-section-kicker">服务列表</p>
          <h3>选择需要预约的到家服务</h3>
        </div>
      </div>

      <div v-if="filteredServices.length > 0" class="xjd-service-list">
        <button
          v-for="service in filteredServices"
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
  </CustomerAppShell>
</template>
