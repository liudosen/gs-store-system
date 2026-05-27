import { computed, ref } from "vue";
import { appRoutes } from "./routes";

export function useAppRouting() {
  const currentPath = ref(window.location.pathname);

  function navigate(path) {
    if (path === currentPath.value) {
      return;
    }

    window.history.pushState({}, "", path);
    currentPath.value = path;
    window.scrollTo({ top: 0, behavior: "auto" });
  }

  function syncRoute() {
    currentPath.value = window.location.pathname;
  }

  const activePage = computed(() => appRoutes[currentPath.value] || appRoutes["/"]);

  return {
    activePage,
    currentPath,
    navigate,
    syncRoute,
  };
}
