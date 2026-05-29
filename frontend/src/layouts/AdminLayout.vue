<script setup>
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Message } from '@arco-design/web-vue'
import {
  IconApps,
  IconDashboard,
  IconFile,
  IconOrderedList,
  IconSafe,
  IconSubscribe,
  IconTags,
  IconUserGroup
} from '@arco-design/web-vue/es/icon'
import { useAuthStore } from '@/stores/auth'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const menuGroups = [
  {
    title: '运营总览',
    items: [{ key: '/dashboard', label: '首页概览', desc: '经营数据', icon: IconDashboard }]
  },
  {
    title: '商品中心',
    items: [
      { key: '/categories', label: '分类管理', desc: '类目结构', icon: IconTags },
      { key: '/products', label: '商品管理', desc: '基础商品', icon: IconApps }
    ]
  },
  {
    title: '交易履约',
    items: [
      { key: '/orders', label: '订单管理', desc: '交易履约', icon: IconOrderedList },
      { key: '/subscriptions', label: '订阅与充值', desc: '自动充值', icon: IconSubscribe },
      { key: '/subscription-records', label: '订阅记录', desc: '状态历史', icon: IconFile }
    ]
  },
  {
    title: '会员与系统',
    items: [
      { key: '/users', label: '用户管理', desc: '会员资料', icon: IconUserGroup },
      { key: '/admins', label: '权限管理', desc: '后台账号', icon: IconSafe },
      { key: '/logs', label: '日志查询', desc: '运行日志', icon: IconFile }
    ]
  }
]

const menuItems = menuGroups.flatMap((group) => group.items)
const selectedKey = computed(() => `/${route.path.split('/')[1] || 'dashboard'}`)
const currentMenu = computed(() => menuItems.find((item) => item.key === selectedKey.value) || menuItems[0])

async function handleLogout() {
  await auth.logout()
  Message.success('已退出登录')
  router.replace('/login')
}
</script>

<template>
  <a-layout class="admin-shell">
    <a-layout-sider class="admin-sider" :width="260">
      <button class="brand-block" type="button" @click="router.push('/dashboard')" aria-label="返回首页概览">
        <span class="brand-mark">膳</span>
        <span class="brand-copy">
          <strong>国膳甄选</strong>
          <small>业务运营管理台</small>
        </span>
      </button>

      <nav class="nav-panel" aria-label="后台主导航">
        <section v-for="group in menuGroups" :key="group.title" class="nav-group">
          <div class="nav-group-title">{{ group.title }}</div>
          <button
            v-for="item in group.items"
            :key="item.key"
            type="button"
            :class="['nav-item', { 'nav-item-active': selectedKey === item.key }]"
            @click="router.push(item.key)"
          >
            <span class="nav-icon"><component :is="item.icon" /></span>
            <span class="nav-copy">
              <span class="menu-label">{{ item.label }}</span>
              <span class="menu-desc">{{ item.desc }}</span>
            </span>
          </button>
        </section>
      </nav>
    </a-layout-sider>

    <a-layout class="admin-main">
      <a-layout-header class="admin-topbar">
        <div class="topbar-context">
          <a-tag color="arcoblue">{{ currentMenu.label }}</a-tag>
          <span>{{ currentMenu.desc }}</span>
        </div>
        <div class="header-actions">
          <a-button shape="round" @click="router.go(0)">
            <template #icon><IconFile /></template>
            刷新数据
          </a-button>
          <a-dropdown trigger="click">
            <button class="user-chip" type="button">
              <span>{{ (auth.user?.username || 'A').slice(0, 1).toUpperCase() }}</span>
              <strong>{{ auth.user?.username || '管理员' }}</strong>
            </button>
            <template #content>
              <a-doption disabled>{{ auth.user?.role || 'admin' }}</a-doption>
              <a-doption @click="handleLogout">退出登录</a-doption>
            </template>
          </a-dropdown>
        </div>
      </a-layout-header>
      <a-layout-content class="admin-content">
        <router-view />
      </a-layout-content>
    </a-layout>
  </a-layout>
</template>
