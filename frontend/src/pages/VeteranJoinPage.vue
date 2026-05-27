<script setup>
import { onboardingSteps } from "../content/site-content";
import { useVeteranJoinForm } from "../features/onboarding/useVeteranJoinForm";

const emit = defineEmits(["navigate"]);

const { form, formError, isSubmitting, submitForm } = useVeteranJoinForm({
  onSuccess: () => emit("navigate", "/veteran-portal"),
});

function go(path) {
  emit("navigate", path);
}
</script>

<template>
  <div class="site-shell join-shell">
    <header class="simple-header">
      <button class="text-link" type="button" @click="go('/')">返回官网</button>
      <button class="button ghost" type="button" @click="go('/veteran-portal')">登录就业平台</button>
    </header>

    <main class="join-layout">
      <section class="join-intro">
        <p class="section-kicker">Veteran Onboarding</p>
        <h1>退役军人入驻申请</h1>
        <p>请提交基础身份信息，平台将完成入驻审核与联系确认。</p>

        <div class="step-list">
          <article v-for="(step, index) in onboardingSteps" :key="step" class="step-card">
            <span>0{{ index + 1 }}</span>
            <p>{{ step }}</p>
          </article>
        </div>
      </section>

      <section class="form-panel">
        <div class="panel-heading">
          <p class="section-kicker">优待证信息</p>
          <h2>提交基础资料</h2>
        </div>

        <form class="join-form" @submit.prevent="submitForm">
          <label>
            <span>姓名</span>
            <input v-model="form.name" type="text" placeholder="请输入真实姓名" required />
          </label>

          <label>
            <span>身份证号</span>
            <input
              v-model="form.idNumber"
              type="text"
              maxlength="18"
              placeholder="请输入身份证号码"
              required
            />
          </label>

          <label>
            <span>手机号</span>
            <input
              v-model="form.phone"
              type="tel"
              inputmode="numeric"
              maxlength="11"
              pattern="^1[3-9]\d{9}$"
              title="请输入 11 位中国大陆手机号"
              placeholder="请输入手机号"
              required
            />
          </label>

          <label>
            <span>优待证号</span>
            <input
              v-model="form.veteranCardNumber"
              type="text"
              placeholder="请输入优待证号码"
              required
            />
          </label>

          <label class="check-row">
            <input v-model="form.agree" type="checkbox" />
            <span>我已确认提交的信息真实有效，并愿意进入平台入驻审核流程。</span>
          </label>

          <div class="form-actions">
            <button class="button primary" type="submit" :disabled="isSubmitting || !form.agree">
              {{ isSubmitting ? "提交中..." : "提交入驻申请" }}
            </button>
            <button class="button ghost" type="button" @click="go('/veteran-portal')">已有账号去登录</button>
          </div>
        </form>

        <div v-if="formError" class="error-card" role="alert">
          <strong>提交失败</strong>
          <p>{{ formError }}</p>
        </div>
      </section>
    </main>
  </div>
</template>
