<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Message } from '@arco-design/web-vue'
import {
  IconApps,
  IconDashboard,
  IconFile,
  IconMenuFold,
  IconMenuUnfold,
  IconOrderedList,
  IconSafe,
  IconSubscribe,
  IconTags,
  IconUserGroup
} from '@arco-design/web-vue/es/icon'
import { ADMIN_PERMISSIONS } from '@/config/adminPermissions'
import { useAuthStore } from '@/stores/auth'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const menuGroups = [
  {
    title: '运营总览',
    items: [
      {
        key: '/dashboard',
        label: '首页概览',
        desc: '经营数据',
        icon: IconDashboard,
        permission: ADMIN_PERMISSIONS.DASHBOARD_VIEW
      }
    ]
  },
  {
    title: '商品中心',
    items: [
      {
        key: '/categories',
        label: '分类管理',
        desc: '类目结构',
        icon: IconTags,
        permission: ADMIN_PERMISSIONS.CATEGORY_LIST_VIEW
      },
      {
        key: '/products',
        label: '商品管理',
        desc: '基础商品',
        icon: IconApps,
        permission: ADMIN_PERMISSIONS.GOODS_VIEW
      }
    ]
  },
  {
    title: '交易履约',
    items: [
      {
        key: '/orders',
        label: '订单管理',
        desc: '交易履约',
        icon: IconOrderedList,
        permission: ADMIN_PERMISSIONS.ORDER_LIST_VIEW
      },
      {
        key: '/subscriptions',
        label: '订阅与充值',
        desc: '自动充值',
        icon: IconSubscribe,
        permission: ADMIN_PERMISSIONS.SUBSCRIPTION_VIEW
      },
      {
        key: '/subscription-records',
        label: '订阅记录',
        desc: '状态历史',
        icon: IconFile,
        permission: ADMIN_PERMISSIONS.SUBSCRIPTION_RECORD_VIEW
      }
    ]
  },
  {
    title: '会员与系统',
    items: [
      {
        key: '/users',
        label: '用户管理',
        desc: '会员资料',
        icon: IconUserGroup,
        permission: ADMIN_PERMISSIONS.WECHAT_USER_LIST_VIEW
      },
      {
        key: '/admins',
        label: '权限管理',
        desc: '后台账号',
        icon: IconSafe,
        permission: ADMIN_PERMISSIONS.ADMIN_USER_VIEW
      },
      {
        key: '/logs',
        label: '日志查询',
        desc: '运行日志',
        icon: IconFile,
        permission: ADMIN_PERMISSIONS.LOGS_VIEW
      }
    ]
  }
]

const visibleMenuGroups = computed(() =>
  menuGroups
    .map((group) => ({
      ...group,
      items: group.items.filter((item) => auth.canAccess(item.permission))
    }))
    .filter((group) => group.items.length)
)
const menuItems = computed(() => visibleMenuGroups.value.flatMap((group) => group.items))
const selectedKey = computed(() => `/${route.path.split('/')[1] || 'dashboard'}`)
const currentMenu = computed(() => menuItems.value.find((item) => item.key === selectedKey.value) || menuItems.value[0] || {})

const isMobile = ref(false)
const sidebarCollapsed = ref(false)
const mobileMenuVisible = ref(false)
let mobileMediaQuery

function syncViewportState() {
  isMobile.value = Boolean(mobileMediaQuery?.matches)
  if (!isMobile.value) {
    mobileMenuVisible.value = false
  }
}

onMounted(() => {
  if (typeof window === 'undefined') {
    return
  }
  mobileMediaQuery = window.matchMedia('(max-width: 960px)')
  syncViewportState()
  if (typeof mobileMediaQuery.addEventListener === 'function') {
    mobileMediaQuery.addEventListener('change', syncViewportState)
  } else {
    mobileMediaQuery.addListener(syncViewportState)
  }
})

onBeforeUnmount(() => {
  if (!mobileMediaQuery) {
    return
  }
  if (typeof mobileMediaQuery.removeEventListener === 'function') {
    mobileMediaQuery.removeEventListener('change', syncViewportState)
  } else {
    mobileMediaQuery.removeListener(syncViewportState)
  }
})

watch(
  () => route.path,
  () => {
    mobileMenuVisible.value = false
  }
)

function navigateTo(key) {
  router.push(key)
  mobileMenuVisible.value = false
}

function toggleNavigation() {
  if (isMobile.value) {
    mobileMenuVisible.value = true
    return
  }
  sidebarCollapsed.value = !sidebarCollapsed.value
}

const navigationToggleLabel = computed(() => {
  if (isMobile.value) {
    return '打开主菜单'
  }
  return sidebarCollapsed.value ? '展开主菜单' : '收起主菜单'
})
const navigationToggleIcon = computed(() => (isMobile.value || sidebarCollapsed.value ? IconMenuUnfold : IconMenuFold))

async function handleLogout() {
  await auth.logout()
  Message.success('已退出登录')
  router.replace('/login')
}
</script>

<template>
  <a-layout class="admin-shell">
    <a-layout-sider
      class="admin-sider"
      :class="{ 'admin-sider-collapsed': sidebarCollapsed }"
      :width="260"
      :collapsed-width="76"
      :collapsed="sidebarCollapsed"
      collapsible
      hide-trigger
    >
      <button class="brand-block" type="button" @click="navigateTo('/dashboard')" aria-label="返回首页概览">
        <span class="brand-mark">膳</span>
        <span class="brand-copy">
          <strong>国膳甄选</strong>
          <small>业务运营管理台</small>
        </span>
      </button>

      <nav class="nav-panel" aria-label="后台主导航">
        <section v-for="group in visibleMenuGroups" :key="group.title" class="nav-group">
          <div class="nav-group-title">{{ group.title }}</div>
          <button
            v-for="item in group.items"
            :key="item.key"
            type="button"
            :class="['nav-item', { 'nav-item-active': selectedKey === item.key }]"
            @click="navigateTo(item.key)"
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

    <a-drawer
      v-model:visible="mobileMenuVisible"
      class="mobile-menu-drawer"
      placement="left"
      :width="292"
      :footer="false"
      :closable="false"
      unmount-on-close
    >
      <aside class="mobile-sider-shell">
        <button class="brand-block" type="button" @click="navigateTo('/dashboard')" aria-label="返回首页概览">
          <span class="brand-mark">膳</span>
          <span class="brand-copy">
            <strong>国膳甄选</strong>
            <small>业务运营管理台</small>
          </span>
        </button>

        <nav class="nav-panel" aria-label="后台主导航">
          <section v-for="group in visibleMenuGroups" :key="`mobile-${group.title}`" class="nav-group">
            <div class="nav-group-title">{{ group.title }}</div>
            <button
              v-for="item in group.items"
              :key="item.key"
              type="button"
              :class="['nav-item', { 'nav-item-active': selectedKey === item.key }]"
              @click="navigateTo(item.key)"
            >
              <span class="nav-icon"><component :is="item.icon" /></span>
              <span class="nav-copy">
                <span class="menu-label">{{ item.label }}</span>
                <span class="menu-desc">{{ item.desc }}</span>
              </span>
            </button>
          </section>
        </nav>
      </aside>
    </a-drawer>

    <a-layout class="admin-main">
      <a-layout-header class="admin-topbar">
        <div class="topbar-left">
          <button
            class="sidebar-toggle"
            type="button"
            :aria-label="navigationToggleLabel"
            :aria-expanded="isMobile ? mobileMenuVisible : !sidebarCollapsed"
            @click="toggleNavigation"
          >
            <component :is="navigationToggleIcon" />
          </button>
          <div class="topbar-context">
            <a-tag color="arcoblue">{{ currentMenu.label }}</a-tag>
            <span>{{ currentMenu.desc }}</span>
          </div>
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
