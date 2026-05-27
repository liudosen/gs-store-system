<script setup>
import { computed, ref, watch } from "vue";
import { areaList } from "@vant/area-data";
import { Area, Popup } from "vant";

const LEGACY_TO_STANDARD_AREA_CODE = {
  "sh-pudong": "310115",
  "sh-minhang": "310112",
};

const STANDARD_TO_LEGACY_REGION_CODE = {
  "310115": "sh-pudong",
  "310112": "sh-minhang",
};

const LEGACY_REGION_NAME_LABEL = {
  Pudong: "上海市 / 上海市 / 浦东新区",
  Minhang: "上海市 / 上海市 / 闵行区",
};

const props = defineProps({
  profileSummary: { type: Array, required: true },
  veteranCardNumber: { type: String, default: "待录入" },
  regionCode: { type: String, default: "" },
  regionName: { type: String, default: "" },
});

const emit = defineEmits(["logout", "update-region"]);

const showRegionPicker = ref(false);
const pendingAreaCode = ref(resolveStandardAreaCode(props.regionCode));

const currentAreaCode = computed(() => resolveStandardAreaCode(props.regionCode));
const pendingRegionLabel = computed(() => {
  const regionName = props.regionName?.trim();
  const currentRegionLabel = regionName
    ? LEGACY_REGION_NAME_LABEL[regionName] || regionName
    : formatAreaLabel(currentAreaCode.value) || "待配置服务区域";

  return formatAreaLabel(pendingAreaCode.value) || currentRegionLabel;
});

watch(
  () => props.regionCode,
  (nextRegionCode) => {
    pendingAreaCode.value = resolveStandardAreaCode(nextRegionCode);
  },
  { immediate: true },
);

function openRegionPicker() {
  pendingAreaCode.value = currentAreaCode.value;
  showRegionPicker.value = true;
}

function handleAreaConfirm({ selectedValues, selectedOptions }) {
  const [provinceCode, cityCode, countyCode] = selectedValues;
  const regionCode = countyCode || cityCode || provinceCode;
  if (!regionCode) {
    return;
  }

  const normalizedCode = STANDARD_TO_LEGACY_REGION_CODE[regionCode] || regionCode;
  const regionName = selectedOptions
    .filter(Boolean)
    .map((option) => option.text)
    .join(" / ");

  emit("update-region", {
    regionCode: normalizedCode,
    regionName,
    areaCode: regionCode,
  });
  showRegionPicker.value = false;
}

function resolveStandardAreaCode(regionCode) {
  if (!regionCode) {
    return "310115";
  }

  return LEGACY_TO_STANDARD_AREA_CODE[regionCode] || regionCode;
}

function formatAreaLabel(areaCode) {
  if (!areaCode) {
    return "";
  }

  const county = areaList.county?.[areaCode];
  if (county) {
    const cityCode = `${areaCode.slice(0, 4)}00`;
    const provinceCode = `${areaCode.slice(0, 2)}0000`;
    const provinceName = areaList.province?.[provinceCode] || "";
    const cityName = areaList.city?.[cityCode] || "";
    return [provinceName, cityName, county].filter(Boolean).join(" / ");
  }

  const city = areaList.city?.[areaCode];
  if (city) {
    const provinceCode = `${areaCode.slice(0, 2)}0000`;
    const provinceName = areaList.province?.[provinceCode] || "";
    return [provinceName, city].filter(Boolean).join(" / ");
  }

  return areaList.province?.[areaCode] || "";
}
</script>

<template>
  <div class="vb-screen">
    <section class="vb-section-card">
      <div class="vb-section-head">
        <div>
          <p class="vb-kicker">我的资料</p>
          <h2>身份与联系信息</h2>
        </div>
      </div>

      <ul class="vb-profile-list">
        <li v-for="item in profileSummary" :key="item.label">
          <span>{{ item.label }}</span>
          <strong>{{ item.value }}</strong>
        </li>
        <li>
          <span>优待证号</span>
          <strong>{{ veteranCardNumber || "待录入" }}</strong>
        </li>
      </ul>
    </section>

    <section class="vb-section-card">
      <div class="vb-region-picker">
        <button class="button primary wide" type="button" @click="openRegionPicker">
          修改服务区域
        </button>
      </div>
    </section>

    <Popup
      v-model:show="showRegionPicker"
      position="bottom"
      round
      class="vb-area-popup"
      safe-area-inset-bottom
    >
      <section class="vb-area-panel">
        <div class="vb-area-summary">
          <span>切换后将使用</span>
          <strong>{{ pendingRegionLabel }}</strong>
        </div>

        <Area
          v-model="pendingAreaCode"
          title="选择服务区域"
          :area-list="areaList"
          :columns-num="3"
          :columns-placeholder="['请选择省份', '请选择城市', '请选择区县']"
          confirm-button-text="确认"
          cancel-button-text="取消"
          @cancel="showRegionPicker = false"
          @confirm="handleAreaConfirm"
        />
      </section>
    </Popup>

    <div class="vb-profile-actions">
      <button class="button ghost wide" type="button" @click="emit('logout')">退出登录</button>
    </div>
  </div>
</template>

<style scoped>
.vb-region-picker {
  margin-top: 0;
}

.vb-area-popup {
  overflow: hidden;
}

.vb-area-panel {
  padding: 12px 0 max(12px, env(safe-area-inset-bottom));
  background: linear-gradient(180deg, #fffdf8 0%, #ffffff 100%);
}

.vb-area-summary {
  margin: 0 16px 12px;
  padding: 14px 16px;
  border-radius: 16px;
  background: linear-gradient(135deg, rgba(10, 58, 104, 0.08), rgba(188, 149, 92, 0.08));
  border: 1px solid rgba(10, 58, 104, 0.08);
}

.vb-area-summary span {
  display: block;
  font-size: 0.76rem;
  color: var(--text-muted);
}

.vb-area-summary strong {
  display: block;
  margin-top: 6px;
  font-size: 0.96rem;
  line-height: 1.45;
  color: var(--brand);
}
</style>
