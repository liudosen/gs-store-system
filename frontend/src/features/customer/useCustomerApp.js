import { computed, reactive, ref } from "vue";
import {
  createCustomerAddress,
  createOrder,
  getCustomerMe,
  listCustomerAddresses,
  listOrders,
  listServiceItems,
  loginCustomerBySms,
  sendCustomerSmsCode,
  updateCustomerAddress,
  updateCustomerRegion,
} from "../../shared/api/customer";
import {
  CUSTOMER_APP_REGION_KEY,
  CUSTOMER_PHONE_STORAGE_KEY,
  CUSTOMER_SELECTED_SERVICE_STORAGE_KEY,
  CUSTOMER_TOKEN_STORAGE_KEY,
} from "../../shared/constants/storage";

const token = ref(localStorage.getItem(CUSTOMER_TOKEN_STORAGE_KEY) || "");
const profile = ref(null);
const regions = ref([]);
const addresses = ref([]);
const services = ref([]);
const serviceTimeSlots = ref([]);
const orders = ref([]);
const selectedOrderId = ref(null);
const orderCategory = ref("active");
const orderHasMore = ref(false);
const orderNextOffset = ref(0);
const remainingSeconds = ref(0);
let countdownTimer = null;

const loading = reactive({
  bootstrap: false,
  sendingCode: false,
  loggingIn: false,
  savingRegion: false,
  savingAddress: false,
  creatingOrder: false,
  loadingOrders: false,
  loadingMoreOrders: false,
});

const messages = reactive({
  loginError: "",
  loginSuccess: "",
  appError: "",
  appSuccess: "",
});

const loginForm = reactive({
  phone: localStorage.getItem(CUSTOMER_PHONE_STORAGE_KEY) || "",
  code: "",
});

const addressForm = reactive({
  id: null,
  regionCode: "",
  regionName: "",
  cityName: "",
  districtName: "",
  detailAddress: "",
  contactName: "",
  contactPhone: loginForm.phone,
  isDefault: true,
});

const orderForm = reactive({
  serviceItemId: null,
  addressId: "",
  serviceDate: getNextDate(),
  serviceTimeSlot: "",
  note: "",
});

const isLoggedIn = computed(() => Boolean(token.value));
const selectedRegionCode = computed(
  () => profile.value?.selected_region_code || localStorage.getItem(CUSTOMER_APP_REGION_KEY) || "",
);
const defaultAddress = computed(
  () => addresses.value.find((item) => item.is_default) || addresses.value[0] || null,
);
const selectedService = computed(
  () => services.value.find((item) => item.id === orderForm.serviceItemId) || null,
);
const selectedOrder = computed(
  () => orders.value.find((item) => item.id === selectedOrderId.value) || orders.value[0] || null,
);

export function useCustomerApp() {
  async function bootstrap() {
    if (!token.value || loading.bootstrap) {
      return;
    }

    loading.bootstrap = true;
    messages.appError = "";

    try {
      const [meResponse, addressResponse, serviceResponse, orderResponse] = await Promise.all([
        getCustomerMe(token.value),
        listCustomerAddresses(token.value),
        listServiceItems(selectedRegionCode.value || "", token.value),
        listOrders(token.value, { category: orderCategory.value, offset: 0, limit: 10 }),
      ]);

      assertApiSuccess(meResponse, "获取用户信息失败");
      assertApiSuccess(addressResponse, "获取地址失败");
      assertApiSuccess(serviceResponse, "获取服务失败");
      assertApiSuccess(orderResponse, "获取订单失败");

      profile.value = meResponse.data.data.user;
      regions.value = meResponse.data.data.regions;
      addresses.value = addressResponse.data.data.items;
      services.value = serviceResponse.data.data.items;
      serviceTimeSlots.value = serviceResponse.data.data.time_slots || [];
      orders.value = orderResponse.data.data.items;
      orderHasMore.value = Boolean(orderResponse.data.data.has_more);
      orderNextOffset.value = Number(orderResponse.data.data.next_offset || orders.value.length);
      selectedOrderId.value = selectedOrderId.value || orders.value[0]?.id || null;

      if (profile.value?.selected_region_code) {
        localStorage.setItem(CUSTOMER_APP_REGION_KEY, profile.value.selected_region_code);
      }

      if (!orderForm.addressId && defaultAddress.value) {
        orderForm.addressId = defaultAddress.value.id;
      }

      if (!orderForm.serviceTimeSlot && serviceTimeSlots.value.length > 0) {
        orderForm.serviceTimeSlot = serviceTimeSlots.value[0];
      }
    } catch (error) {
      messages.appError = error instanceof Error ? error.message : "加载失败";
      if (messages.appError.includes("authorization") || messages.appError.includes("session")) {
        logout();
      }
    } finally {
      loading.bootstrap = false;
    }
  }

  async function fetchOrders(category = orderCategory.value) {
    if (!token.value || loading.loadingOrders) {
      return;
    }

    loading.loadingOrders = true;
    orderCategory.value = category;

    try {
      const orderResponse = await listOrders(token.value, { category, offset: 0, limit: 10 });
      assertApiSuccess(orderResponse, "获取订单失败");
      orders.value = orderResponse.data.data.items;
      orderHasMore.value = Boolean(orderResponse.data.data.has_more);
      orderNextOffset.value = Number(orderResponse.data.data.next_offset || orders.value.length);
      selectedOrderId.value = selectedOrderId.value || orders.value[0]?.id || null;
    } catch (error) {
      messages.appError = error instanceof Error ? error.message : "获取订单失败";
    } finally {
      loading.loadingOrders = false;
    }
  }

  async function fetchMoreOrders() {
    if (!token.value || loading.loadingMoreOrders || !orderHasMore.value) {
      return;
    }

    loading.loadingMoreOrders = true;

    try {
      const orderResponse = await listOrders(token.value, {
        category: orderCategory.value,
        offset: orderNextOffset.value,
        limit: 10,
      });
      assertApiSuccess(orderResponse, "获取更多订单失败");
      const nextItems = orderResponse.data.data.items || [];
      orders.value = [...orders.value, ...nextItems];
      orderHasMore.value = Boolean(orderResponse.data.data.has_more);
      orderNextOffset.value = Number(orderResponse.data.data.next_offset || orders.value.length);
    } catch (error) {
      messages.appError = error instanceof Error ? error.message : "获取更多订单失败";
    } finally {
      loading.loadingMoreOrders = false;
    }
  }

  async function sendCode() {
    if (loading.sendingCode || remainingSeconds.value > 0) {
      return;
    }

    loading.sendingCode = true;
    messages.loginError = "";
    messages.loginSuccess = "";

    try {
      const response = await sendCustomerSmsCode({ phone: loginForm.phone });
      assertApiSuccess(response, "验证码发送失败");
      localStorage.setItem(CUSTOMER_PHONE_STORAGE_KEY, loginForm.phone);
      startCountdown(Number(response.data.data.next_send_in_seconds || 60));
      messages.loginSuccess = "验证码已发送，请查收短信。";
    } catch (error) {
      messages.loginError = error instanceof Error ? error.message : "验证码发送失败";
    } finally {
      loading.sendingCode = false;
    }
  }

  async function login() {
    if (loading.loggingIn) {
      return false;
    }

    loading.loggingIn = true;
    messages.loginError = "";
    messages.loginSuccess = "";

    try {
      const response = await loginCustomerBySms({
        phone: loginForm.phone,
        code: loginForm.code,
      });
      assertApiSuccess(response, "登录失败");
      token.value = response.data.data.token;
      localStorage.setItem(CUSTOMER_TOKEN_STORAGE_KEY, token.value);
      localStorage.setItem(CUSTOMER_PHONE_STORAGE_KEY, response.data.data.phone);
      messages.loginSuccess = "登录成功";
      void bootstrap();
      return true;
    } catch (error) {
      messages.loginError = error instanceof Error ? error.message : "登录失败";
      return false;
    } finally {
      loading.loggingIn = false;
    }
  }

  async function chooseRegion(regionCode) {
    if (!regionCode || loading.savingRegion || !token.value) {
      return;
    }

    loading.savingRegion = true;
    messages.appError = "";
    messages.appSuccess = "";

    try {
      const response = await updateCustomerRegion({ region_code: regionCode }, token.value);
      assertApiSuccess(response, "地区更新失败");
      profile.value = response.data.data.user;
      regions.value = response.data.data.regions;
      localStorage.setItem(CUSTOMER_APP_REGION_KEY, regionCode);

      const serviceResponse = await listServiceItems(regionCode, token.value);
      assertApiSuccess(serviceResponse, "获取服务失败");
      services.value = serviceResponse.data.data.items;
      serviceTimeSlots.value = serviceResponse.data.data.time_slots || [];
      if (serviceTimeSlots.value.length > 0) {
        orderForm.serviceTimeSlot = serviceTimeSlots.value[0];
      }
      messages.appSuccess = "服务地区已更新";
    } catch (error) {
      messages.appError = error instanceof Error ? error.message : "地区更新失败";
    } finally {
      loading.savingRegion = false;
    }
  }

  function startAddressCreate() {
    addressForm.id = null;
    addressForm.regionCode = selectedRegionCode.value || "";
    addressForm.regionName = profile.value?.selected_region_name || "";
    addressForm.cityName = "";
    addressForm.districtName = "";
    addressForm.detailAddress = "";
    addressForm.contactName = "";
    addressForm.contactPhone = loginForm.phone || "";
    addressForm.isDefault = addresses.value.length === 0;
    messages.appError = "";
    messages.appSuccess = "";
  }

  function startAddressEdit(address) {
    addressForm.id = address.id;
    addressForm.regionCode = address.region_code;
    addressForm.regionName = address.region_name;
    addressForm.cityName = address.city_name;
    addressForm.districtName = address.district_name;
    addressForm.detailAddress = address.detail_address;
    addressForm.contactName = address.contact_name;
    addressForm.contactPhone = address.contact_phone;
    addressForm.isDefault = address.is_default;
    messages.appError = "";
    messages.appSuccess = "";
  }

  async function saveAddress() {
    if (loading.savingAddress || !token.value) {
      return false;
    }

    loading.savingAddress = true;
    messages.appError = "";

    try {
      const payload = {
        region_code: addressForm.regionCode,
        region_name: addressForm.regionName,
        city_name: addressForm.cityName,
        district_name: addressForm.districtName,
        detail_address: addressForm.detailAddress,
        contact_name: addressForm.contactName,
        contact_phone: addressForm.contactPhone,
        is_default: addressForm.isDefault,
      };

      const response = addressForm.id
        ? await updateCustomerAddress(addressForm.id, payload, token.value)
        : await createCustomerAddress(payload, token.value);
      assertApiSuccess(response, "地址保存失败");

      const refreshResponse = await listCustomerAddresses(token.value);
      assertApiSuccess(refreshResponse, "地址刷新失败");
      addresses.value = refreshResponse.data.data.items;
      if (defaultAddress.value) {
        orderForm.addressId = defaultAddress.value.id;
      }
      messages.appSuccess = "地址已保存";
      return true;
    } catch (error) {
      messages.appError = error instanceof Error ? error.message : "地址保存失败";
      return false;
    } finally {
      loading.savingAddress = false;
    }
  }

  function selectService(serviceId) {
    orderForm.serviceItemId = serviceId;
    messages.appError = "";
    messages.appSuccess = "";
  }

  async function submitOrder() {
    if (!token.value || loading.creatingOrder) {
      return false;
    }

    loading.creatingOrder = true;
    messages.appError = "";
    messages.appSuccess = "";

    try {
      const response = await createOrder(
        {
          service_item_id: orderForm.serviceItemId,
          address_id: Number(orderForm.addressId),
          service_date: orderForm.serviceDate,
          service_time_slot: orderForm.serviceTimeSlot,
          note: orderForm.note,
        },
        token.value,
      );
      assertApiSuccess(response, "下单失败");

      const orderResponse = await listOrders(token.value, { category: orderCategory.value, offset: 0, limit: 10 });
      assertApiSuccess(orderResponse, "订单刷新失败");
      orders.value = orderResponse.data.data.items;
      orderHasMore.value = Boolean(orderResponse.data.data.has_more);
      orderNextOffset.value = Number(orderResponse.data.data.next_offset || orders.value.length);
      selectedOrderId.value = response.data.data.order.id;
      messages.appSuccess = "预约已提交，系统正在进入匹配与派单流程。";
      orderForm.note = "";
      return true;
    } catch (error) {
      messages.appError = error instanceof Error ? error.message : "下单失败";
      return false;
    } finally {
      loading.creatingOrder = false;
    }
  }

  function pickOrder(orderId) {
    selectedOrderId.value = orderId;
  }

  function logout() {
    token.value = "";
    profile.value = null;
    regions.value = [];
    addresses.value = [];
    services.value = [];
    serviceTimeSlots.value = [];
    orders.value = [];
    orderCategory.value = "active";
    orderHasMore.value = false;
    orderNextOffset.value = 0;
    selectedOrderId.value = null;
    messages.loginError = "";
    messages.loginSuccess = "";
    messages.appError = "";
    messages.appSuccess = "";
    localStorage.removeItem(CUSTOMER_TOKEN_STORAGE_KEY);
    localStorage.removeItem(CUSTOMER_PHONE_STORAGE_KEY);
  }

  return {
    addressForm,
    addresses,
    bootstrap,
    chooseRegion,
    defaultAddress,
    fetchOrders,
    fetchMoreOrders,
    isLoggedIn,
    loading,
    login,
    loginForm,
    logout,
    messages,
    orderForm,
    orders,
    orderCategory,
    orderHasMore,
    pickOrder,
    profile,
    regions,
    remainingSeconds,
    saveAddress,
    selectedOrder,
    selectedRegionCode,
    selectedService,
    selectService,
    sendCode,
    services,
    startAddressCreate,
    startAddressEdit,
    submitOrder,
    timeSlots: serviceTimeSlots,
  };
}

export function persistSelectedCustomerService(serviceId) {
  if (!serviceId) {
    sessionStorage.removeItem(CUSTOMER_SELECTED_SERVICE_STORAGE_KEY);
    return;
  }

  sessionStorage.setItem(CUSTOMER_SELECTED_SERVICE_STORAGE_KEY, String(serviceId));
}

function assertApiSuccess(response, fallbackMessage) {
  if (!response.ok || !response.data?.success) {
    throw new Error(response.data?.message || fallbackMessage);
  }
}

function getNextDate() {
  const date = new Date();
  date.setDate(date.getDate() + 1);
  return date.toISOString().slice(0, 10);
}

function startCountdown(seconds) {
  if (countdownTimer) {
    window.clearInterval(countdownTimer);
  }

  remainingSeconds.value = seconds;
  countdownTimer = window.setInterval(() => {
    remainingSeconds.value -= 1;
    if (remainingSeconds.value <= 0) {
      remainingSeconds.value = 0;
      window.clearInterval(countdownTimer);
      countdownTimer = null;
    }
  }, 1000);
}
