<script setup>
import { computed, ref, watch } from "vue";
import { areaList } from "@vant/area-data";
import { Area, Popup } from "vant";

const props = defineProps({
  form: {
    type: Object,
    required: true,
  },
  loading: {
    type: Boolean,
    default: false,
  },
  title: {
    type: String,
    default: "新增地址",
  },
  submitLabel: {
    type: String,
    default: "保存地址",
  },
  submitBusyLabel: {
    type: String,
    default: "保存中...",
  },
  successMessage: {
    type: String,
    default: "",
  },
  errorMessage: {
    type: String,
    default: "",
  },
  showCancel: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(["submit", "cancel"]);

const showAreaPicker = ref(false);
const pendingAreaCode = ref("310115");

const selectedAreaLabel = computed(() => {
  if (props.form.regionName) {
    return props.form.regionName;
  }

  return formatAreaLabel(pendingAreaCode.value) || "请选择省市区";
});

watch(
  () => props.form.regionCode,
  (nextCode) => {
    if (nextCode) {
      pendingAreaCode.value = nextCode;
    }
  },
  { immediate: true },
);

function openAreaPicker() {
  pendingAreaCode.value = props.form.regionCode || pendingAreaCode.value || "310115";
  showAreaPicker.value = true;
}

function handleAreaConfirm({ selectedValues, selectedOptions }) {
  const [provinceCode, cityCode, countyCode] = selectedValues;
  const regionCode = countyCode || cityCode || provinceCode;
  if (!regionCode) {
    return;
  }

  props.form.regionCode = regionCode;
  props.form.regionName = selectedOptions.filter(Boolean).map((item) => item.text).join(" / ");
  props.form.cityName =
    selectedOptions.find((item) => item?.value === cityCode)?.text ||
    selectedOptions.find((item) => item?.value === regionCode)?.text ||
    "";
  props.form.districtName =
    selectedOptions.find((item) => item?.value === countyCode)?.text ||
    props.form.cityName;
  pendingAreaCode.value = regionCode;
  showAreaPicker.value = false;
}

function formatAreaLabel(areaCode) {
  if (!areaCode) {
    return "";
  }

  const county = areaList.county?.[areaCode];
  if (county) {
    const cityCode = `${areaCode.slice(0, 4)}00`;
    const provinceCode = `${areaCode.slice(0, 2)}0000`;
    return [areaList.province?.[provinceCode], areaList.city?.[cityCode], county]
      .filter(Boolean)
      .join(" / ");
  }

  const city = areaList.city?.[areaCode];
  if (city) {
    const provinceCode = `${areaCode.slice(0, 2)}0000`;
    return [areaList.province?.[provinceCode], city].filter(Boolean).join(" / ");
  }

  return areaList.province?.[areaCode] || "";
}
</script>

<template>
  <div class="xjd-detail-stack">
    <div class="xjd-section-head">
      <div>
        <p class="xjd-section-kicker">地址表单</p>
        <h3>{{ title }}</h3>
      </div>
    </div>

    <form class="xjd-form" @submit.prevent="emit('submit')">
      <label class="xjd-field">
        <span>省市区</span>
        <button class="xjd-link-row" type="button" @click="openAreaPicker">
          <div>
            <strong>{{ selectedAreaLabel }}</strong>
          </div>
          <span>选择</span>
        </button>
      </label>

      <label class="xjd-field">
        <span>联系人</span>
        <input v-model="form.contactName" type="text" placeholder="请输入联系人" required />
      </label>

      <label class="xjd-field">
        <span>联系电话</span>
        <input v-model="form.contactPhone" type="tel" maxlength="11" placeholder="请输入联系电话" required />
      </label>

      <label class="xjd-field">
        <span>详细地址</span>
        <textarea v-model="form.detailAddress" rows="4" placeholder="请输入详细地址" required />
      </label>

      <label class="xjd-check-row">
        <input v-model="form.isDefault" type="checkbox" />
        <span>设为默认地址</span>
      </label>

      <p v-if="successMessage" class="form-note success">{{ successMessage }}</p>
      <p v-if="errorMessage" class="form-note error">{{ errorMessage }}</p>

      <div class="xjd-login-actions">
        <button class="button primary wide" type="submit" :disabled="loading">
          {{ loading ? submitBusyLabel : submitLabel }}
        </button>
        <button v-if="showCancel" class="button ghost wide" type="button" @click="emit('cancel')">
          取消
        </button>
      </div>
    </form>
  </div>

  <Popup
    v-model:show="showAreaPicker"
    position="bottom"
    round
    class="vb-area-popup"
    safe-area-inset-bottom
  >
    <section class="vb-area-panel">
      <div class="vb-area-summary">
        <span>当前选择</span>
        <strong>{{ formatAreaLabel(pendingAreaCode) || "请选择省市区" }}</strong>
      </div>

      <Area
        v-model="pendingAreaCode"
        title="选择地址地区"
        :area-list="areaList"
        :columns-num="3"
        :columns-placeholder="['请选择省份', '请选择城市', '请选择区县']"
        confirm-button-text="确认"
        cancel-button-text="取消"
        @cancel="showAreaPicker = false"
        @confirm="handleAreaConfirm"
      />
    </section>
  </Popup>
</template>
