<script setup>
import { onMounted } from "vue";
import CustomerAppShell from "../components/customer/CustomerAppShell.vue";
import { useCustomerApp } from "../features/customer/useCustomerApp";

const props = defineProps({
  currentPath: {
    type: String,
    default: "",
  },
});

const emit = defineEmits(["navigate"]);
const { bootstrap, logout, profile } = useCustomerApp();

onMounted(() => {
  bootstrap();
});

function handleLogout() {
  logout();
  emit("navigate", "/xia-dao-jia/login");
}
</script>

<template>
  <CustomerAppShell
    :current-path="props.currentPath"
    title="我的"
    kicker="侠到家"
    side-label="地区"
    :side-value="profile?.selected_region_name || '未选择'"
    @navigate="emit('navigate', $event)"
  >
    <section class="xjd-section-card xjd-profile-card">
      <div class="xjd-profile-avatar">侠</div>
      <div class="xjd-profile-copy">
        <p class="xjd-section-kicker">账号</p>
        <h3>{{ profile?.phone || "未登录" }}</h3>
        <p>{{ profile?.selected_region_name || "未选择服务地区" }}</p>
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
        <article class="xjd-detail-card">
          <h3>服务地区</h3>
          <p>{{ profile?.selected_region_name || "未选择" }}</p>
        </article>
      </div>
    </section>

    <section class="xjd-section-card">
      <div class="xjd-section-head">
        <div>
          <p class="xjd-section-kicker">地址管理</p>
          <h3>维护上门服务地址</h3>
        </div>
      </div>

      <div class="xjd-link-list">
        <button class="xjd-link-row" type="button" @click="emit('navigate', '/xia-dao-jia/addresses')">
          <div>
            <strong>地址管理</strong>
            <p>新增、编辑默认地址，供下单时直接选择。</p>
          </div>
          <span>进入</span>
        </button>
      </div>
    </section>

    <section class="xjd-section-card">
      <button class="button ghost" type="button" @click="handleLogout">退出登录</button>
    </section>
  </CustomerAppShell>
</template>
