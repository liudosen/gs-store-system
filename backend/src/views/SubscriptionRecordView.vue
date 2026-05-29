<script setup>
import { onMounted, reactive, ref } from 'vue'
import { Message } from '@arco-design/web-vue'
import { fetchSubscriptionRecords } from '@/api/admin'

const loading = ref(false)
const errorText = ref('')
const list = ref([])
const total = ref(0)

const filters = reactive({
  page: 1,
  page_size: 10,
  openid: '',
  action: undefined,
  date_range: []
})

function read(record, camelKey, snakeKey, fallback = '') {
  if (!record) return fallback
  return record[camelKey] ?? record[snakeKey] ?? fallback
}

function actionLabel(record) {
  const action = Number(read(record, 'action', 'action', -1))
  return read(record, 'actionLabel', 'action_label') || (action === 1 ? '开启' : action === 0 ? '关闭' : '未知')
}

function actionColor(record) {
  return Number(read(record, 'action', 'action', -1)) === 1 ? 'green' : 'gray'
}

function cleanParams() {
  const params = Object.fromEntries(
    Object.entries(filters).filter(([, value]) => value !== '' && value !== undefined && value !== null)
  )
  delete params.date_range
  if (filters.date_range?.[0]) params.start_date = filters.date_range[0]
  if (filters.date_range?.[1]) params.end_date = filters.date_range[1]
  return params
}

function resetFilters() {
  Object.assign(filters, { page: 1, openid: '', action: undefined, date_range: [] })
  loadData()
}

async function loadData() {
  loading.value = true
  errorText.value = ''
  try {
    const data = await fetchSubscriptionRecords(cleanParams())
    list.value = data?.list || []
    total.value = data?.total || 0
  } catch (error) {
    errorText.value = error.message || '订阅记录加载失败'
    Message.error(errorText.value)
  } finally {
    loading.value = false
  }
}

function search() {
  filters.page = 1
  loadData()
}

function handlePageChange(page) {
  filters.page = page
  loadData()
}

onMounted(loadData)
</script>

<template>
  <div class="page-stack subscription-record-page">
    <a-card :bordered="false" class="page-card">
      <template #title>订阅记录</template>

      <div v-if="errorText" class="table-error">
        <span>{{ errorText }}</span>
        <a-button size="small" @click="loadData">重新加载</a-button>
      </div>

      <a-space wrap class="toolbar">
        <a-input-search v-model="filters.openid" placeholder="小程序ID / openId" allow-clear style="width: 260px" @search="search" />
        <a-select v-model="filters.action" placeholder="订阅状态" allow-clear style="width: 140px">
          <a-option :value="1">开启</a-option>
          <a-option :value="0">关闭</a-option>
        </a-select>
        <a-range-picker v-model="filters.date_range" value-format="YYYY-MM-DD" style="width: 260px" />
        <a-button type="primary" @click="search">查询</a-button>
        <a-button @click="resetFilters">重置</a-button>
        <a-tag color="arcoblue">共 {{ total }} 条</a-tag>
      </a-space>

      <a-spin :loading="loading" style="width: 100%">
        <div v-if="!list.length" class="table-empty">
          <a-empty description="暂无订阅记录" />
        </div>
        <div v-else class="management-table-wrap subscription-table-wrap">
          <table class="management-table subscription-table">
            <thead>
              <tr>
                <th>记录 ID</th>
                <th>用户</th>
                <th>小程序ID</th>
                <th>手机</th>
                <th>订阅状态</th>
                <th>最后修改时间</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="record in list" :key="read(record, 'id', 'id')">
                <td><span class="strong-text">{{ read(record, 'id', 'id') }}</span></td>
                <td>
                  <div class="user-identity-cell">
                    <strong>{{ read(record, 'realName', 'real_name') || '未实名用户' }}</strong>
                    <span>{{ read(record, 'phone', 'phone') || '-' }}</span>
                  </div>
                </td>
                <td><span class="mini-app-id">{{ read(record, 'openId', 'open_id') || read(record, 'openid', 'openid') || '-' }}</span></td>
                <td>{{ read(record, 'phone', 'phone') || '-' }}</td>
                <td><a-tag :color="actionColor(record)">{{ actionLabel(record) }}</a-tag></td>
                <td><span class="muted-text">{{ read(record, 'createdAt', 'created_at') || '-' }}</span></td>
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
  </div>
</template>
