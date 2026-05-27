<script setup>
import { computed, onMounted } from "vue";
import CustomerAppShell from "../components/customer/CustomerAppShell.vue";
import {
  persistSelectedCustomerService,
  useCustomerApp,
} from "../features/customer/useCustomerApp";

const props = defineProps({
  currentPath: {
    type: String,
    default: "",
  },
});

const emit = defineEmits(["navigate"]);
const { bootstrap, chooseRegion, loading, messages, orders, profile, regions, selectedRegionCode, services } =
  useCustomerApp();

const selectedRegionName = computed(
  () =>
    profile.value?.selected_region_name ||
    regions.value.find((item) => item.code === selectedRegionCode.value)?.name ||
    "未选择",
);

const latestOrder = computed(() => orders.value[0] || null);
const previewServices = computed(() => services.value.slice(0, 4));
const serviceCountLabel = computed(() => `${services.value.length} 项服务可预约`);
const addressReadiness = computed(() => (profile.value?.selected_region_code ? "已选择服务地区" : "待选择服务地区"));
const topRegions = computed(() => regions.value.slice(0, 4));

onMounted(() => {
  bootstrap();
});

function goToService(serviceId) {
  persistSelectedCustomerService(serviceId);
  emit("navigate", "/xia-dao-jia/services");
}
</script>

<template>
  <CustomerAppShell
    :current-path="props.currentPath"
    title="首页"
    kicker="侠到家"
    side-label="地区"
    :side-value="selectedRegionName"
    @navigate="emit('navigate', $event)"
  >
    <section class="xjd-home-section">
      <div class="xjd-home-layout">
        <div class="xjd-home-grid">
          <article class="xjd-home-card">
            <p class="mini-kicker">当前地区</p>
            <h3>{{ selectedRegionName }}</h3>
            <p>{{ loading.bootstrap ? "正在同步服务清单与上门能力范围。" : "按地区分配可预约服务与服务者能力范围。" }}</p>
          </article>

          <article class="xjd-home-card">
            <p class="mini-kicker">服务供给</p>
            <h3>{{ loading.bootstrap ? "加载中" : serviceCountLabel }}</h3>
            <p>服务目录、价格与时长都来自后端接口和数据库，不走前端静态 mock。</p>
          </article>

          <article class="xjd-home-card">
            <p class="mini-kicker">预约准备</p>
            <h3>{{ addressReadiness }}</h3>
            <p>先选地区，再确认地址，之后即可发起到家服务预约。</p>
          </article>

          <article class="xjd-home-card">
            <p class="mini-kicker">最近动态</p>
            <h3>{{ latestOrder ? latestOrder.status_label : "暂无订单" }}</h3>
            <p>{{ latestOrder ? `${latestOrder.service_item_name} · ${latestOrder.service_date}` : "创建订单后会在这里显示最近的派单进度。" }}</p>
          </article>
        </div>

        <aside class="xjd-home-entry">
          <p class="section-kicker">侠到家</p>
          <strong v-if="loading.bootstrap">正在准备可预约服务</strong>
          <strong v-else-if="messages.appError">服务目录加载失败</strong>
          <strong v-else-if="services.length">面向家庭的高信任到家服务入口</strong>
          <strong v-else>切换地区后查看可预约服务</strong>
          <p v-if="messages.appError">{{ messages.appError }}</p>
          <p v-else-if="services.length">
            选择服务类型、预约时段和上门地址后，系统会进入匹配与派单流程，你可以在订单中心实时查看状态。
          </p>
          <p v-else>
            当前地区还没有可预约服务，可以切换到其他地区继续查看。
          </p>

          <div class="xjd-home-chip-row" aria-label="服务地区快速切换">
            <button
              v-for="region in topRegions"
              :key="region.code"
              class="xjd-home-chip"
              :class="{ active: region.code === selectedRegionCode }"
              type="button"
              @click="chooseRegion(region.code)"
            >
              {{ region.name }}
            </button>
          </div>

          <div class="xjd-home-actions">
            <button class="button primary" :disabled="loading.bootstrap" type="button" @click="emit('navigate', '/xia-dao-jia/services')">
              去预约
            </button>
            <button class="button ghost" :disabled="loading.bootstrap" type="button" @click="emit('navigate', '/xia-dao-jia/orders')">
              查看订单
            </button>
          </div>
        </aside>
      </div>
    </section>

    <section v-if="!loading.bootstrap" class="xjd-section-card">
      <div class="xjd-section-head">
        <div>
          <p class="xjd-section-kicker">服务预览</p>
          <h3>热门可预约服务</h3>
        </div>
        <button
          v-if="services.length > 4"
          class="xjd-app-chip"
          type="button"
          @click="emit('navigate', '/xia-dao-jia/services')"
        >
          全部服务
        </button>
      </div>

      <div v-if="services.length > 0" class="xjd-service-list">
        <button
          v-for="service in previewServices"
          :key="service.id"
          class="xjd-service-row"
          :class="{ active: latestOrder?.service_item_id === service.id }"
          type="button"
          @click="goToService(service.id)"
        >
          <div class="xjd-service-row-main">
            <span class="xjd-service-row-badge">{{ service.badge }}</span>
            <strong>{{ service.name }}</strong>
          </div>
          <div class="xjd-service-row-side">
            <b>¥{{ service.base_price }}</b>
            <small>{{ service.duration_minutes }} 分钟</small>
          </div>
        </button>
      </div>

      <div v-else class="xjd-empty-inline">
        <p>当前地区暂无可预约服务</p>
        <div class="xjd-chip-strip">
          <button
            v-for="region in regions"
            :key="region.code"
            class="xjd-app-chip"
            :class="{ active: region.code === selectedRegionCode }"
            type="button"
            @click="chooseRegion(region.code)"
          >
            {{ region.name }}
          </button>
        </div>
      </div>
    </section>

    <!-- 最近订单 -->
    <section v-if="!loading.bootstrap" class="xjd-section-card">
      <div class="xjd-section-head">
        <div>
          <p class="xjd-section-kicker">订单追踪</p>
          <h3>最近订单进度</h3>
        </div>
        <button class="xjd-app-chip" type="button" @click="emit('navigate', '/xia-dao-jia/orders')">订单中心</button>
      </div>

      <div v-if="latestOrder" class="xjd-progress-body">
        <span class="xjd-status-pill">{{ latestOrder.status_label }}</span>
        <p>{{ latestOrder.service_item_name }}</p>
        <p style="color: var(--xjd-text-muted)">{{ latestOrder.service_date }} {{ latestOrder.service_time_slot }}</p>
        <p style="color: var(--xjd-text-muted)">{{ latestOrder.dispatch_message }}</p>
      </div>

      <div v-else class="xjd-empty-inline">
        <p>还没有订单记录</p>
        <button
          class="xjd-app-chip"
          type="button"
          @click="emit('navigate', '/xia-dao-jia/services')"
        >
          查看服务
        </button>
      </div>
    </section>
  </CustomerAppShell>
</template>
