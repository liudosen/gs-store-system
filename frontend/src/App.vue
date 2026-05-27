<script setup>
import { onBeforeUnmount, onMounted, watch } from "vue";
import { useAppRouting } from "./app/useAppRouting";
import { usePageMeta } from "./app/usePageMeta";
import { CUSTOMER_TOKEN_STORAGE_KEY } from "./shared/constants/storage";

const { activePage, currentPath, navigate, syncRoute } = useAppRouting();
const { updatePageMeta } = usePageMeta();

onMounted(() => {
  window.addEventListener("popstate", syncRoute);
});

onBeforeUnmount(() => {
  window.removeEventListener("popstate", syncRoute);
});

watch(
  currentPath,
  (path) => {
    if (path.startsWith("/xiadaojia") && path !== "/xia-dao-jia/login") {
      const token = localStorage.getItem(CUSTOMER_TOKEN_STORAGE_KEY);
      if (!token) {
        navigate("/xia-dao-jia/login");
        return;
      }
    }

    updatePageMeta(path);
  },
  { immediate: true },
);
</script>

<template>
  <component :is="activePage" :current-path="currentPath" @navigate="navigate" />
</template>
