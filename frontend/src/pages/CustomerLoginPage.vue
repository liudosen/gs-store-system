<script setup>
import logoImage from "../assets/gs-store-system-logo.jpg";
import { useCustomerApp } from "../features/customer/useCustomerApp";

const emit = defineEmits(["navigate"]);
const { loading, login, loginForm, messages, remainingSeconds, sendCode } = useCustomerApp();

async function submitLogin() {
  const success = await login();
  if (success) {
    emit("navigate", "/xiadaojia");
  }
}
</script>

<template>
  <div class="xjd-page xjd-login-page">
    <main class="xjd-login-simple">
      <section class="xjd-login-card">
        <div class="xjd-login-card-head">
          <div class="xjd-login-brand">
            <img class="xjd-login-logo" :src="logoImage" alt="mcx 侠到�? />
            <div>
              <p class="xjd-section-kicker">侠到�?/p>
              <h2>短信登录</h2>
            </div>
          </div>
          <p class="xjd-login-card-note">登录后可继续查看服务、预约记录和派单进度�?/p>
        </div>

        <form class="xjd-form" @submit.prevent="submitLogin">
          <label class="xjd-field">
            <span>手机�?/span>
            <input
              v-model="loginForm.phone"
              type="tel"
              inputmode="numeric"
              maxlength="11"
              autocomplete="tel"
              placeholder="请输入手机号"
              required
            />
          </label>

          <label class="xjd-field">
            <span>验证�?/span>
            <input
              v-model="loginForm.code"
              type="text"
              inputmode="numeric"
              maxlength="6"
              autocomplete="one-time-code"
              placeholder="请输入验证码"
              required
            />
          </label>

          <div class="xjd-login-actions">
            <button
              class="button ghost xjd-code-button"
              type="button"
              :disabled="loading.sendingCode || remainingSeconds > 0"
              @click="sendCode"
            >
              {{ loading.sendingCode ? "发送中" : remainingSeconds > 0 ? `${remainingSeconds}s 后重试` : "获取验证�? }}
            </button>

            <button class="button primary wide" type="submit" :disabled="loading.loggingIn">
              {{ loading.loggingIn ? "登录�?.." : "进入侠到�? }}
            </button>
          </div>

          <p v-if="messages.loginSuccess" class="form-note success">{{ messages.loginSuccess }}</p>
          <p v-if="messages.loginError" class="form-note error">{{ messages.loginError }}</p>
        </form>

        <div class="xjd-login-footer">
          <button class="text-link" type="button" @click="emit('navigate', '/')">返回官网</button>
        </div>
      </section>
    </main>
  </div>
</template>

