<script setup>
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { Message } from '@arco-design/web-vue'
import { fetchRecentLogs } from '@/api/admin'

const loading = ref(false)
const pollingLoading = ref(false)
const autoRefresh = ref(true)
const refreshSeconds = ref(5)
const pollingTimer = ref(null)
const errorText = ref('')
const data = ref(null)
const filters = reactive({
  kind: 'app',
  limit: 200,
  level: '',
  keyword: ''
})

function read(record, camelKey, snakeKey, fallback = '') {
  if (!record) return fallback
  return record[camelKey] ?? record[snakeKey] ?? fallback
}

const lines = computed(() => read(data.value, 'lines', 'lines', []) || [])

const filteredLines = computed(() => {
  const keyword = filters.keyword.trim().toLowerCase()
  return lines.value.filter((item) => {
    const level = read(item, 'level', 'level')
    const line = read(item, 'line', 'line')
    if (filters.level && level !== filters.level) return false
    if (keyword && !String(line).toLowerCase().includes(keyword)) return false
    return true
  })
})

function levelColor(level) {
  return {
    ERROR: 'red',
    WARN: 'orange',
    INFO: 'blue',
    DEBUG: 'purple',
    TRACE: 'gray'
  }[level] || 'gray'
}

async function loadData(options = {}) {
  const silent = Boolean(options.silent)
  if (silent && (loading.value || pollingLoading.value)) return
  if (silent) {
    pollingLoading.value = true
  } else {
    loading.value = true
  }
  errorText.value = ''
  try {
    data.value = await fetchRecentLogs({ kind: filters.kind, limit: filters.limit })
  } catch (error) {
    errorText.value = error.message || '日志加载失败'
    if (!silent) Message.error(errorText.value)
  } finally {
    if (silent) {
      pollingLoading.value = false
    } else {
      loading.value = false
    }
  }
}

function resetFilters() {
  Object.assign(filters, { kind: 'app', limit: 200, level: '', keyword: '' })
  loadData()
}

function stopPolling() {
  if (pollingTimer.value) {
    window.clearInterval(pollingTimer.value)
    pollingTimer.value = null
  }
}

function startPolling() {
  stopPolling()
  if (!autoRefresh.value) return
  pollingTimer.value = window.setInterval(() => {
    loadData({ silent: true })
  }, refreshSeconds.value * 1000)
}

watch([autoRefresh, refreshSeconds], startPolling)

onMounted(() => {
  loadData()
  startPolling()
})

onBeforeUnmount(stopPolling)
</script>

<template>
  <div class="page-stack log-page">
    <a-card :bordered="false" class="page-card">
      <template #title>日志查询</template>
      <template #extra>
        <a-button :loading="loading" @click="loadData">刷新日志</a-button>
      </template>

      <div v-if="errorText" class="table-error">
        <span>{{ errorText }}</span>
        <a-button size="small" @click="loadData">重新加载</a-button>
      </div>

      <a-space wrap class="toolbar">
        <a-select v-model="filters.kind" placeholder="日志类型" style="width: 140px" @change="loadData">
          <a-option value="app">应用日志</a-option>
          <a-option value="error">错误日志</a-option>
        </a-select>
        <a-select v-model="filters.limit" placeholder="读取条数" style="width: 140px" @change="loadData">
          <a-option :value="100">最近 100 条</a-option>
          <a-option :value="200">最近 200 条</a-option>
          <a-option :value="500">最近 500 条</a-option>
          <a-option :value="1000">最近 1000 条</a-option>
        </a-select>
        <a-select v-model="filters.level" placeholder="日志级别" allow-clear style="width: 130px">
          <a-option value="ERROR">ERROR</a-option>
          <a-option value="WARN">WARN</a-option>
          <a-option value="INFO">INFO</a-option>
          <a-option value="DEBUG">DEBUG</a-option>
          <a-option value="TRACE">TRACE</a-option>
        </a-select>
        <a-input-search v-model="filters.keyword" placeholder="关键字过滤" allow-clear style="width: 260px" />
        <a-button type="primary" :loading="loading" @click="loadData">查询</a-button>
        <a-button @click="resetFilters">重置</a-button>
        <a-tag color="arcoblue">显示 {{ filteredLines.length }} / {{ lines.length }} 条</a-tag>
        <a-divider direction="vertical" />
        <a-switch v-model="autoRefresh" checked-text="自动刷新" unchecked-text="手动" />
        <a-select v-model="refreshSeconds" style="width: 110px" :disabled="!autoRefresh">
          <a-option :value="3">3 秒</a-option>
          <a-option :value="5">5 秒</a-option>
          <a-option :value="10">10 秒</a-option>
          <a-option :value="30">30 秒</a-option>
        </a-select>
        <a-tag v-if="autoRefresh" :color="pollingLoading ? 'orange' : 'green'">
          {{ pollingLoading ? '刷新中' : `${refreshSeconds} 秒轮询` }}
        </a-tag>
      </a-space>

      <a-spin :loading="loading" style="width: 100%">
        <a-descriptions v-if="data" :column="3" bordered class="log-meta">
          <a-descriptions-item label="来源">{{ read(data, 'source', 'source') }}</a-descriptions-item>
          <a-descriptions-item label="文件大小">{{ read(data, 'sizeBytes', 'size_bytes', 0) }} B</a-descriptions-item>
          <a-descriptions-item label="更新时间">{{ read(data, 'modifiedAt', 'modified_at') || '-' }}</a-descriptions-item>
          <a-descriptions-item label="文件路径" :span="3">{{ read(data, 'path', 'path') || '-' }}</a-descriptions-item>
        </a-descriptions>

        <div v-if="data && !read(data, 'exists', 'exists')" class="table-empty">
          <a-empty description="日志文件不存在或暂无日志" />
        </div>

        <div v-else class="log-list">
          <div v-for="(item, index) in filteredLines" :key="`${index}-${read(item, 'line', 'line')}`" class="log-row">
            <a-tag :color="levelColor(read(item, 'level', 'level'))">{{ read(item, 'level', 'level') }}</a-tag>
            <pre>{{ read(item, 'line', 'line') }}</pre>
          </div>
          <a-empty v-if="data && !filteredLines.length" class="table-empty" description="没有匹配的日志" />
        </div>
      </a-spin>
    </a-card>
  </div>
</template>
