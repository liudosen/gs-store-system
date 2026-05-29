<script setup>
import { onMounted, reactive, ref } from 'vue'
import { Message, Modal } from '@arco-design/web-vue'
import {
  deleteWechatUser,
  fetchBalanceTransactions,
  fetchPaymentPassword,
  fetchWechatUsers,
  updateWechatUser
} from '@/api/admin'

const loading = ref(false)
const saving = ref(false)
const visible = ref(false)
const balanceVisible = ref(false)
const balanceLoading = ref(false)
const editing = ref(null)
const list = ref([])
const total = ref(0)
const errorText = ref('')
const balanceInfo = ref(null)
const balanceUser = ref(null)
const formRef = ref(null)

const filters = reactive({
  page: 1,
  page_size: 10,
  openid: '',
  phone: '',
  gender: undefined,
  real_name_status: undefined
})

const form = reactive({
  openid: '',
  real_name: '',
  phone: '',
  gender: 0,
  id_card_number: '',
  country: '',
  province: '',
  city: '',
  avatar_url: ''
})

function read(record, camelKey, snakeKey, fallback = '') {
  if (!record) return fallback
  return record[camelKey] ?? record[snakeKey] ?? fallback
}

function miniAppId(record) {
  return read(record, 'openId', 'open_id') || read(record, 'openid', 'openid')
}

function cleanParams() {
  return Object.fromEntries(
    Object.entries(filters).filter(([, value]) => value !== '' && value !== undefined && value !== null)
  )
}

function money(value) {
  return `¥${(Number(value || 0) / 100).toFixed(2)}`
}

function mask(value) {
  const text = String(value || '')
  return text.length > 6 ? `${text.slice(0, 3)}****${text.slice(-3)}` : text || '-'
}

function genderLabel(value) {
  return ['未知', '男', '女'][Number(value)] || '未知'
}

function genderColor(value) {
  return ['gray', 'arcoblue', 'pinkpurple'][Number(value)] || 'gray'
}

function realNameStatus(record) {
  return read(record, 'realName', 'real_name') && read(record, 'idCardNumber', 'id_card_number')
}

function addressText(record) {
  const parts = [
    read(record, 'addressProvince', 'address_province'),
    read(record, 'addressCity', 'address_city'),
    read(record, 'addressDistrict', 'address_district'),
    read(record, 'detailAddress', 'detail_address')
  ].filter(Boolean)
  return parts.join('') || '-'
}

function resetForm(record) {
  editing.value = record
  Object.assign(form, {
    openid: miniAppId(record),
    real_name: read(record, 'realName', 'real_name'),
    phone: read(record, 'phone', 'phone'),
    gender: read(record, 'gender', 'gender', 0),
    id_card_number: read(record, 'idCardNumber', 'id_card_number'),
    country: read(record, 'country', 'country'),
    province: read(record, 'province', 'province'),
    city: read(record, 'city', 'city'),
    avatar_url: read(record, 'avatarUrl', 'avatar_url')
  })
}

async function loadData() {
  loading.value = true
  errorText.value = ''
  try {
    const data = await fetchWechatUsers(cleanParams())
    list.value = data?.list || []
    total.value = data?.total || 0
  } catch (error) {
    errorText.value = error.message || '用户加载失败'
    Message.error(errorText.value)
  } finally {
    loading.value = false
  }
}

function search() {
  filters.page = 1
  loadData()
}

function resetFilters() {
  Object.assign(filters, { page: 1, openid: '', phone: '', gender: undefined, real_name_status: undefined })
  loadData()
}

function handlePageChange(page) {
  filters.page = page
  loadData()
}

function openEdit(record) {
  resetForm(record)
  visible.value = true
}

async function saveUser() {
  if (!editing.value) return
  const validation = await formRef.value?.validate()
  if (validation) return
  saving.value = true
  try {
    await updateWechatUser(read(editing.value, 'id', 'id'), { ...form, gender: Number(form.gender) })
    Message.success('用户已更新')
    visible.value = false
    await loadData()
  } catch (error) {
    Message.error(error.message || '保存失败')
  } finally {
    saving.value = false
  }
}

function confirmDelete(record) {
  Modal.confirm({
    title: '确认删除用户',
    content: `用户 ${miniAppId(record)} 删除后不可恢复。`,
    okText: '删除',
    okButtonProps: { status: 'danger' },
    async onOk() {
      await deleteWechatUser(read(record, 'id', 'id'))
      Message.success('用户已删除')
      await loadData()
    }
  })
}

function revealPassword(record) {
  Modal.confirm({
    title: '查看支付密码',
    content: '支付密码属于敏感信息，请确认当前操作已获授权。',
    async onOk() {
      const data = await fetchPaymentPassword(miniAppId(record))
      Modal.info({ title: '支付密码', content: data.payment_password || data.paymentPassword || '未设置' })
    }
  })
}

async function showBalance(record) {
  balanceVisible.value = true
  balanceUser.value = record
  balanceInfo.value = null
  balanceLoading.value = true
  try {
    balanceInfo.value = await fetchBalanceTransactions(miniAppId(record))
  } catch (error) {
    Message.error(error.message || '余额加载失败')
  } finally {
    balanceLoading.value = false
  }
}

onMounted(loadData)
</script>

<template>
  <div class="page-stack wechat-user-page">
    <a-card :bordered="false" class="page-card">
      <template #title>用户管理</template>

      <div v-if="errorText" class="table-error">
        <span>{{ errorText }}</span>
        <a-button size="small" @click="loadData">重新加载</a-button>
      </div>

      <a-space wrap class="toolbar">
        <a-input-search v-model="filters.openid" placeholder="小程序ID / openId" allow-clear style="width: 240px" @search="search" />
        <a-input v-model="filters.phone" placeholder="手机号" allow-clear style="width: 180px" />
        <a-select v-model="filters.gender" placeholder="性别" allow-clear style="width: 120px">
          <a-option :value="0">未知</a-option>
          <a-option :value="1">男</a-option>
          <a-option :value="2">女</a-option>
        </a-select>
        <a-select v-model="filters.real_name_status" placeholder="实名状态" allow-clear style="width: 140px">
          <a-option :value="0">未实名</a-option>
          <a-option :value="1">已实名</a-option>
        </a-select>
        <a-button type="primary" @click="search">查询</a-button>
        <a-button @click="resetFilters">重置</a-button>
        <a-tag color="arcoblue">共 {{ total }} 人</a-tag>
      </a-space>

      <a-spin :loading="loading" style="width: 100%">
        <div v-if="!list.length" class="table-empty">
          <a-empty description="暂无用户数据" />
        </div>
        <div v-else class="management-table-wrap wechat-user-table-wrap">
          <table class="management-table wechat-user-table">
            <thead>
              <tr>
                <th>用户</th>
                <th>小程序ID</th>
                <th>手机</th>
                <th>性别</th>
                <th>实名状态</th>
                <th>身份证</th>
                <th>默认地址</th>
                <th>最近登录</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="record in list" :key="read(record, 'id', 'id')">
                <td>
                  <div class="user-profile-cell">
                    <a-avatar class="user-avatar" :image-url="read(record, 'avatarUrl', 'avatar_url')">
                      {{ String(read(record, 'realName', 'real_name') || '用').slice(0, 1) }}
                    </a-avatar>
                    <div class="user-identity-cell">
                      <strong>{{ read(record, 'realName', 'real_name') || '未命名用户' }}</strong>
                      <span>内部ID {{ read(record, 'id', 'id') || '-' }}</span>
                    </div>
                  </div>
                </td>
                <td><span class="mini-app-id">{{ miniAppId(record) || '-' }}</span></td>
                <td>{{ read(record, 'phone', 'phone') || read(record, 'addressPhone', 'address_phone') || '-' }}</td>
                <td><a-tag :color="genderColor(read(record, 'gender', 'gender', 0))">{{ genderLabel(read(record, 'gender', 'gender', 0)) }}</a-tag></td>
                <td>
                  <a-tag :color="realNameStatus(record) ? 'green' : 'gray'">
                    {{ realNameStatus(record) ? '已实名' : '未实名' }}
                  </a-tag>
                </td>
                <td>{{ mask(read(record, 'idCardNumber', 'id_card_number')) }}</td>
                <td>
                  <div class="address-cell">
                    <strong>{{ read(record, 'receiverName', 'receiver_name') || '-' }}</strong>
                    <span>{{ addressText(record) }}</span>
                  </div>
                </td>
                <td><span class="muted-text">{{ read(record, 'lastLoginAt', 'last_login_at') || '-' }}</span></td>
                <td>
                  <a-space>
                    <a-button size="small" @click="openEdit(record)">编辑</a-button>
                    <a-button size="small" @click="showBalance(record)">余额</a-button>
                    <a-button size="small" @click="revealPassword(record)">支付密码</a-button>
                    <a-button size="small" status="danger" @click="confirmDelete(record)">删除</a-button>
                  </a-space>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </a-spin>

      <div class="table-footer">
        <a-pagination
          :current="filters.page"
          :page-size="filters.page_size"
          :total="total"
          show-total
          @change="handlePageChange"
        />
      </div>
    </a-card>

    <a-modal v-model:visible="visible" title="编辑用户" :confirm-loading="saving" width="720px" @ok="saveUser">
      <a-form ref="formRef" :model="form" layout="vertical">
        <a-form-item field="openid" label="小程序ID（openId）">
          <a-input v-model="form.openid" disabled />
        </a-form-item>
        <a-row :gutter="16">
          <a-col :span="12"><a-form-item field="real_name" label="姓名"><a-input v-model="form.real_name" /></a-form-item></a-col>
          <a-col :span="12"><a-form-item field="phone" label="手机号"><a-input v-model="form.phone" /></a-form-item></a-col>
        </a-row>
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item field="gender" label="性别">
              <a-select v-model="form.gender">
                <a-option :value="0">未知</a-option>
                <a-option :value="1">男</a-option>
                <a-option :value="2">女</a-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="12"><a-form-item field="id_card_number" label="身份证号"><a-input v-model="form.id_card_number" /></a-form-item></a-col>
        </a-row>
        <a-row :gutter="16">
          <a-col :span="8"><a-form-item field="country" label="国家"><a-input v-model="form.country" /></a-form-item></a-col>
          <a-col :span="8"><a-form-item field="province" label="省份"><a-input v-model="form.province" /></a-form-item></a-col>
          <a-col :span="8"><a-form-item field="city" label="城市"><a-input v-model="form.city" /></a-form-item></a-col>
        </a-row>
        <a-form-item field="avatar_url" label="头像 URL"><a-input v-model="form.avatar_url" /></a-form-item>
      </a-form>
    </a-modal>

    <a-drawer v-model:visible="balanceVisible" title="余额与流水" width="680px">
      <a-spin :loading="balanceLoading" style="width: 100%">
        <template v-if="balanceInfo">
          <a-descriptions :column="2" bordered>
            <a-descriptions-item label="用户">{{ read(balanceUser, 'realName', 'real_name') || '-' }}</a-descriptions-item>
            <a-descriptions-item label="当前余额">{{ money(read(balanceInfo, 'balance', 'balance', 0)) }}</a-descriptions-item>
            <a-descriptions-item label="小程序ID" :span="2">{{ miniAppId(balanceUser) || '-' }}</a-descriptions-item>
          </a-descriptions>
          <a-divider>流水明细</a-divider>
          <div class="management-table-wrap">
            <table class="management-table balance-table">
              <thead>
                <tr>
                  <th>金额</th>
                  <th>变动后余额</th>
                  <th>状态</th>
                  <th>备注</th>
                  <th>时间</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="tx in read(balanceInfo, 'transactions', 'transactions', [])" :key="read(tx, 'id', 'id')">
                  <td><span class="strong-text">{{ money(read(tx, 'amount', 'amount', 0)) }}</span></td>
                  <td>{{ money(read(tx, 'balanceAfter', 'balance_after', 0)) }}</td>
                  <td><a-tag :color="Number(read(tx, 'status', 'status', 0)) === 1 ? 'green' : 'red'">{{ Number(read(tx, 'status', 'status', 0)) === 1 ? '成功' : '失败' }}</a-tag></td>
                  <td>{{ read(tx, 'remark', 'remark') || '-' }}</td>
                  <td>{{ read(tx, 'createdAt', 'created_at') || '-' }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </a-spin>
    </a-drawer>
  </div>
</template>
