<script setup>
const props = defineProps({
  currentPath: {
    type: String,
    default: "",
  },
  title: {
    type: String,
    required: true,
  },
  kicker: {
    type: String,
    default: "侠到家",
  },
  sideLabel: {
    type: String,
    default: "",
  },
  sideValue: {
    type: String,
    default: "",
  },
  sideInteractive: {
    type: Boolean,
    default: false,
  },
  activePath: {
    type: String,
    default: "",
  },
  tabs: {
    type: Array,
    default: () => [
      { key: "services", label: "服务", path: "/xiadaojia" },
      { key: "orders", label: "订单", path: "/xiadaojia" },
      { key: "profile", label: "我的", path: "/xiadaojia" },
    ],
  },
  activeTab: {
    type: String,
    default: "",
  },
  useInternalTabs: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(["navigate", "side-action", "tab-change"]);

function isActive(path) {
  return (props.activePath || props.currentPath) === path;
}

function isTabActive(tab) {
  if (props.useInternalTabs) {
    return props.activeTab === (tab.key || tab.path);
  }

  return isActive(tab.path);
}

function handleTabClick(tab) {
  if (props.useInternalTabs) {
    emit("tab-change", tab.key || tab.path);
    return;
  }

  emit("navigate", tab.path);
}
</script>

<template>
  <div class="xjd-page xjd-app-shell">
    <header class="xjd-header">
      <div class="xjd-header-main">
        <p class="xjd-header-kicker">{{ kicker }}</p>
        <h1>{{ title }}</h1>
      </div>

      <button
        v-if="sideValue"
        class="xjd-header-chip"
        :class="{ interactive: sideInteractive }"
        type="button"
        :disabled="!sideInteractive"
        @click="emit('side-action')"
      >
        <span v-if="sideLabel">{{ sideLabel }}</span>
        <strong>{{ sideValue }}</strong>
      </button>
    </header>

    <main class="xjd-screen">
      <slot />
    </main>

    <nav class="xjd-tabbar" aria-label="侠到家导航">
      <button
        v-for="tab in tabs"
        :key="tab.key || tab.path"
        :class="{ active: isTabActive(tab) }"
        type="button"
        @click="handleTabClick(tab)"
      >
        {{ tab.label }}
      </button>
    </nav>
  </div>
</template>
