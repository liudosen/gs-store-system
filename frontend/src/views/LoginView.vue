<script setup>
import { computed, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Message } from '@arco-design/web-vue'
import { useAuthStore } from '@/stores/auth'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()
const loading = ref(false)
const formRef = ref(null)

const form = reactive({
  username: '',
  password: ''
})

const apiBaseUrl = computed(() => import.meta.env.VITE_API_BASE_URL || '同源代理 /auth 与 /api → 127.0.0.1:8081')

const rules = {
  username: [{ required: true, message: '请输入管理员账号' }],
  password: [{ required: true, message: '请输入登录密码' }]
}

function resolveRedirect() {
  const redirect = route.query.redirect
  return typeof redirect === 'string' && redirect.startsWith('/') && !redirect.startsWith('//') ? redirect : '/dashboard'
}

async function handleSubmit() {
  if (loading.value) return
  loading.value = true

  try {
    const validation = await formRef.value?.validate()
    if (validation) return

    await auth.login({
      username: form.username.trim(),
      password: form.password
    })
    Message.success('登录成功')
    router.replace(resolveRedirect())
  } catch (error) {
    const message = error.message === 'invalid credentials' ? '账号或密码错误' : error.message
    Message.error(message || '登录失败，请检查账号或密码')
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <main class="login-page" aria-labelledby="login-title">
    <section class="login-hero" aria-label="管理后台介绍">
      <div class="hero-badge">国膳甄选管理后台</div>
      <h1>国膳甄选运营中枢</h1>
      <p>
        集中处理国膳甄选商品、权益 SKU、订单、用户余额与系统日志，让运营动作更快、更稳、更可追踪。
      </p>
      <div class="hero-metrics" aria-label="后台能力概览">
        <div>
          <strong>11</strong>
          <span>权限能力</span>
        </div>
        <div>
          <strong>7</strong>
          <span>业务模块</span>
        </div>
        <div>
          <strong>24h</strong>
          <span>会话有效期</span>
        </div>
      </div>
    </section>

    <section class="login-panel" aria-label="登录表单">
      <div class="panel-header">
        <div>
          <h2 id="login-title">管理员登录</h2>
        </div>
      </div>

      <a-form
        ref="formRef"
        :model="form"
        :rules="rules"
        layout="vertical"
        size="large"
        @submit="handleSubmit"
      >
        <a-form-item field="username" label="管理员账号" hide-asterisk>
          <a-input
            v-model="form.username"
            placeholder="请输入管理员账号"
            autocomplete="username"
            allow-clear
          />
        </a-form-item>

        <a-form-item field="password" label="登录密码" hide-asterisk>
          <a-input-password
            v-model="form.password"
            placeholder="请输入登录密码"
            autocomplete="current-password"
          />
        </a-form-item>

        <div class="login-options">
          <span>登录后将自动加载当前账号权限</span>
          <span>Token Bearer</span>
        </div>

        <a-button
          class="login-submit"
          type="primary"
          html-type="submit"
          long
          :loading="loading"
        >
          {{ loading ? '正在登录...' : '进入管理后台' }}
        </a-button>
      </a-form>

      <footer class="panel-footer">
        <span>默认接口地址可通过 <code>VITE_API_BASE_URL</code> 调整</span>
      </footer>
    </section>
  </main>
</template>


