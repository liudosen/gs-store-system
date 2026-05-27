import { reactive, ref } from "vue";
import { submitVeteranJoin } from "../../shared/api/onboarding";
import { VETERAN_PHONE_STORAGE_KEY } from "../../shared/constants/storage";

export function useVeteranJoinForm({ onSuccess }) {
  const form = reactive({
    name: "",
    idNumber: "",
    phone: localStorage.getItem(VETERAN_PHONE_STORAGE_KEY) || "",
    veteranCardNumber: "",
    agree: false,
  });

  const isSubmitting = ref(false);
  const formError = ref("");

  async function submitForm() {
    if (!form.agree || isSubmitting.value) {
      return;
    }

    isSubmitting.value = true;
    formError.value = "";

    try {
      const result = await submitVeteranJoin({
        name: form.name,
        id_number: form.idNumber,
        phone: form.phone,
        veteran_card_number: form.veteranCardNumber,
      });

      if (!result.ok || !result.data?.success) {
        throw new Error(result.data?.message || "入驻申请提交失败，请稍后重试");
      }

      localStorage.setItem(VETERAN_PHONE_STORAGE_KEY, form.phone);
      onSuccess?.();
    } catch (error) {
      formError.value =
        error instanceof Error ? error.message : "入驻申请提交失败，请稍后重试";
    } finally {
      isSubmitting.value = false;
    }
  }

  return {
    form,
    formError,
    isSubmitting,
    submitForm,
  };
}
