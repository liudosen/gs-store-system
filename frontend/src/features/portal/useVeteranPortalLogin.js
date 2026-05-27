import { computed, onBeforeUnmount, onMounted, reactive, ref } from "vue";
import { trainingModules, veteranProjects } from "../../content/veteran-portal-content";
import { getVeteranMe, loginVeteranBySms, sendVeteranSmsCode } from "../../shared/api/auth";
import {
  VETERAN_PHONE_STORAGE_KEY,
  VETERAN_TOKEN_STORAGE_KEY,
} from "../../shared/constants/storage";

const defaultProjectIds = veteranProjects.map((project) => project.id);

export function useVeteranPortalLogin({ onNeedOnboarding } = {}) {
  const token = ref(localStorage.getItem(VETERAN_TOKEN_STORAGE_KEY) || "");
  const veteranProfile = ref(null);
  const isBootstrapping = ref(Boolean(token.value));
  const credentials = reactive({
    phone: "",
    smsCode: "",
  });
  const isSendingCode = ref(false);
  const isLoggingIn = ref(false);
  const loginMessage = ref("");
  const loginError = ref("");
  const remainingSeconds = ref(0);
  const selectedProjects = ref([...defaultProjectIds]);
  let countdownTimer = null;

  const isLoggedIn = computed(() => Boolean(token.value));
  const completedCount = computed(
    () => trainingModules.filter((item) => item.status === "已完成").length,
  );

  function clearCountdown() {
    if (countdownTimer) {
      window.clearInterval(countdownTimer);
      countdownTimer = null;
    }
  }

  function startCountdown(seconds) {
    clearCountdown();
    remainingSeconds.value = seconds;
    countdownTimer = window.setInterval(() => {
      remainingSeconds.value -= 1;
      if (remainingSeconds.value <= 0) {
        remainingSeconds.value = 0;
        clearCountdown();
      }
    }, 1000);
  }

  function applyProfile(profile) {
    veteranProfile.value = profile;
    credentials.phone = profile?.phone || credentials.phone;

    if (profile?.phone) {
      localStorage.setItem(VETERAN_PHONE_STORAGE_KEY, profile.phone);
    }
  }

  function logout() {
    token.value = "";
    veteranProfile.value = null;
    loginMessage.value = "";
    loginError.value = "";
    credentials.smsCode = "";
    selectedProjects.value = [...defaultProjectIds];
    localStorage.removeItem(VETERAN_TOKEN_STORAGE_KEY);
  }

  async function bootstrap() {
    if (!token.value) {
      isBootstrapping.value = false;
      return false;
    }

    isBootstrapping.value = true;

    try {
      const result = await getVeteranMe(token.value);
      assertApiSuccess(result, "获取服务者信息失败");
      applyProfile(result.data.data.profile);
      return true;
    } catch (error) {
      if (isUnauthorizedError(error)) {
        logout();
      }

      loginError.value = error instanceof Error ? error.message : "获取服务者信息失败";
      return false;
    } finally {
      isBootstrapping.value = false;
    }
  }

  async function login() {
    if (isLoggingIn.value) {
      return false;
    }

    isLoggingIn.value = true;
    loginMessage.value = "";
    loginError.value = "";

    try {
      const result = await loginVeteranBySms({
        phone: credentials.phone,
        code: credentials.smsCode,
      });

      if (result.status === 403) {
        localStorage.setItem(VETERAN_PHONE_STORAGE_KEY, credentials.phone);
        onNeedOnboarding?.();
        return false;
      }

      assertApiSuccess(result, "登录失败，请检查验证码");

      token.value = result.data.data.token;
      localStorage.setItem(VETERAN_TOKEN_STORAGE_KEY, token.value);
      localStorage.setItem(VETERAN_PHONE_STORAGE_KEY, result.data.data.phone);

      const bootstrapOk = await bootstrap();
      if (!bootstrapOk) {
        throw new Error("登录成功，但获取首页信息失败");
      }

      loginMessage.value = result.data?.message || "登录成功";
      credentials.smsCode = "";
      return true;
    } catch (error) {
      logout();
      loginError.value = error instanceof Error ? error.message : "登录失败，请检查验证码";
      return false;
    } finally {
      isLoggingIn.value = false;
    }
  }

  async function sendSmsCode() {
    if (isSendingCode.value || remainingSeconds.value > 0) {
      return;
    }

    isSendingCode.value = true;
    credentials.smsCode = "";
    loginMessage.value = "";
    loginError.value = "";

    try {
      const result = await sendVeteranSmsCode({
        phone: credentials.phone,
      });

      if (result.status === 403) {
        localStorage.setItem(VETERAN_PHONE_STORAGE_KEY, credentials.phone);
        onNeedOnboarding?.();
        return;
      }

      assertApiSuccess(result, "验证码发送失败，请稍后重试");

      const nextSendInSeconds = Number(result.data?.data?.next_send_in_seconds || 60);
      localStorage.setItem(VETERAN_PHONE_STORAGE_KEY, credentials.phone);
      startCountdown(nextSendInSeconds);
      loginMessage.value =
        result.data?.message || `验证码已发送，请 ${nextSendInSeconds} 秒后重新获取`;
    } catch (error) {
      loginError.value =
        error instanceof Error ? error.message : "验证码发送失败，请稍后重试";
    } finally {
      isSendingCode.value = false;
    }
  }

  function toggleProject(projectId) {
    if (selectedProjects.value.includes(projectId)) {
      selectedProjects.value = selectedProjects.value.filter((item) => item !== projectId);
      return;
    }

    selectedProjects.value = [...selectedProjects.value, projectId];
  }

  onMounted(() => {
    const savedPhone = localStorage.getItem(VETERAN_PHONE_STORAGE_KEY);
    if (savedPhone) {
      credentials.phone = savedPhone;
    }

    if (token.value) {
      bootstrap();
      return;
    }

    isBootstrapping.value = false;
  });

  onBeforeUnmount(() => {
    clearCountdown();
  });

  return {
    availableProjects: veteranProjects,
    bootstrap,
    completedCount,
    credentials,
    isBootstrapping,
    isLoggedIn,
    isLoggingIn,
    isSendingCode,
    login,
    loginError,
    loginMessage,
    logout,
    remainingSeconds,
    selectedProjects,
    sendSmsCode,
    toggleProject,
    trainingModules,
    veteranProfile,
  };
}

function assertApiSuccess(response, fallbackMessage) {
  if (!response.ok || !response.data?.success) {
    const error = new Error(response.data?.message || fallbackMessage);
    error.status = response.status;
    throw error;
  }
}

function isUnauthorizedError(error) {
  return error?.status === 401;
}
