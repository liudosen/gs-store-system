<script setup>
import { onMounted, ref } from "vue";
import CustomerAddressFormPanel from "../components/customer/CustomerAddressFormPanel.vue";
import CustomerAppShell from "../components/customer/CustomerAppShell.vue";
import { useCustomerApp } from "../features/customer/useCustomerApp";

const props = defineProps({
  currentPath: {
    type: String,
    default: "",
  },
});

const emit = defineEmits(["navigate"]);
const {
  addressForm,
  addresses,
  bootstrap,
  loading,
  messages,
  regions,
  saveAddress,
  startAddressCreate,
  startAddressEdit,
} = useCustomerApp();

const showForm = ref(false);
onMounted(async () => {
  await bootstrap();
  if (!addresses.value.length) {
    startAddressCreate();
    showForm.value = true;
  }
});

function createNew() {
  startAddressCreate();
  showForm.value = true;
}

function editAddress(address) {
  startAddressEdit(address);
  showForm.value = true;
}

async function submitAddress() {
  const success = await saveAddress();
  if (success) {
    showForm.value = false;
  }
}
</script>

<template>
  <CustomerAppShell
    :current-path="props.currentPath"
    active-path="/xia-dao-jia/profile"
    title="地址"
    kicker="侠到家"
    @navigate="emit('navigate', $event)"
  >
    <section class="xjd-section-card">
      <div class="xjd-section-head">
        <div>
          <p class="xjd-section-kicker">地址列表</p>
          <h3>全部地址</h3>
        </div>
        <button class="button ghost" type="button" @click="createNew">新增地址</button>
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
      </div>
    </section>

    <section v-if="showForm" class="xjd-section-card">
      <CustomerAddressFormPanel
        :form="addressForm"
        :loading="loading.savingAddress"
        :title="addressForm.id ? '编辑地址' : '新增地址'"
        :success-message="messages.appSuccess"
        :error-message="messages.appError"
        :show-cancel="true"
        @submit="submitAddress"
        @cancel="showForm = false"
      />
    </section>
  </CustomerAppShell>
</template>
