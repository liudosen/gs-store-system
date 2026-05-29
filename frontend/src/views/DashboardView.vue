<script setup>
import { computed, onMounted, reactive, ref } from 'vue'
import { Message } from '@arco-design/web-vue'
import { fetchDashboard } from '@/api/admin'
import PageCard from '@/components/PageCard.vue'

const loading = ref(false)
const errorText = ref('')
const dashboard = ref(null)
const filters = reactive({
  range: [],
  granularity: 'day'
})

const statusOptions = [
  { value: 0, label: '待付款', color: 'orange' },
  { value: 1, label: '待发货', color: 'blue' },
  { value: 2, label: '待收货', color: 'purple' },
  { value: 3, label: '已完成', color: 'green' },
  { value: 4, label: '已取消', color: 'gray' }
]

const metrics = computed(() => [
  {
    key: 'todayOrders',
    label: '今日订单',
    value: formatNumber(field('todayOrders', 'today_orders')),
    helper: `待处理 ${formatNumber(field('pendingOrders', 'pending_orders'))} 单`,
    tone: 'blue'
  },
  {
    key: 'todayRevenue',
    label: '今日收入',
    value: formatAmount(field('todayRevenue', 'today_revenue')),
    helper: `累计 ${formatAmount(field('totalRevenue', 'total_revenue'))}`,
    tone: 'green'
  },
  {
    key: 'totalUsers',
    label: '用户总数',
    value: formatNumber(field('totalUsers', 'total_users')),
    helper: `近 7 日新增 ${formatNumber(field('newUsers7d', 'new_users_7d'))}`,
    tone: 'cyan'
  },
  {
    key: 'activeGoods',
    label: '商品',
    value: `${formatNumber(field('activeGoods', 'active_goods'))}/${formatNumber(field('totalGoods', 'total_goods'))}`,
    helper: `低库存 SKU ${formatNumber(field('lowStockSkus', 'low_stock_skus'))}`,
    tone: field('lowStockSkus', 'low_stock_skus') > 0 ? 'orange' : 'slate'
  },
  {
    key: 'completedOrders',
    label: '履约完成',
    value: formatNumber(field('completedOrders', 'completed_orders')),
    helper: `取消 ${formatNumber(field('cancelledOrders', 'cancelled_orders'))} 单`,
    tone: 'purple'
  }
])

const trend = computed(() => arrayField('trend'))
const statusBreakdown = computed(() => {
  const rows = arrayField('statusBreakdown', 'status_breakdown')
  return rows.map((item) => {
    const status = Number(read(item, 'status'))
    const fallback = statusOptions.find((option) => option.value === status)
    return {
      ...item,
      status,
      label: read(item, 'statusLabel', 'status_label') || fallback?.label || '未知状态',
      color: fallback?.color || 'gray',
      count: Number(read(item, 'count') || 0)
    }
  })
})
const topProducts = computed(() => arrayField('topProducts', 'top_products'))
const recentOrders = computed(() => arrayField('recentOrders', 'recent_orders'))
const generatedAt = computed(() => field('generatedAt', 'generated_at') || '等待加载')

const maxOrderCount = computed(() => Math.max(...trend.value.map((item) => Number(read(item, 'orderCount', 'order_count') || 0)), 1))
const maxRevenue = computed(() => Math.max(...trend.value.map((item) => Number(read(item, 'paidAmount', 'paid_amount') || 0)), 1))
const statusTotal = computed(() => statusBreakdown.value.reduce((sum, item) => sum + item.count, 0))

function read(source, ...keys) {
  if (!source) return undefined
  for (const key of keys) {
    if (source[key] !== undefined && source[key] !== null) return source[key]
  }
  return undefined
}

function field(...keys) {
  return read(dashboard.value, ...keys)
}

function arrayField(...keys) {
  const value = field(...keys)
  return Array.isArray(value) ? value : []
}

function formatNumber(value) {
  if (value === null || value === undefined || value === '') return '-'
  return Number(value).toLocaleString('zh-CN')
}

function formatAmount(value) {
  if (value === null || value === undefined || value === '') return '-'
  return `¥${(Number(value) / 100).toLocaleString('zh-CN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  })}`
}

function formatCustomer(record) {
  return read(record, 'customerName', 'customer_name') || read(record, 'openid') || '-'
}

function statusColor(status) {
  return statusOptions.find((item) => item.value === Number(status))?.color || 'gray'
}

function statusLabel(record) {
  const status = Number(read(record, 'status'))
  return read(record, 'statusLabel', 'status_label') || statusOptions.find((item) => item.value === status)?.label || status || '-'
}

function buildParams() {
  const params = { granularity: filters.granularity }
  if (filters.range?.[0] && filters.range?.[1]) {
    params.start_date = filters.range[0]
    params.end_date = filters.range[1]
  }
  return params
}

async function loadData() {
  loading.value = true
  errorText.value = ''
  try {
    dashboard.value = await fetchDashboard(buildParams())
  } catch (error) {
    errorText.value = error.message || '首页数据加载失败'
    Message.error(errorText.value)
  } finally {
    loading.value = false
  }
}

function resetFilters() {
  filters.range = []
  filters.granularity = 'day'
  loadData()
}

onMounted(loadData)
</script>

<template>
  <div class="page-stack dashboard-page">
    <PageCard>
      <template #title>首页概览</template>
      <template #extra>
        <a-space wrap>
          <a-range-picker
            v-model="filters.range"
            value-format="YYYY-MM-DD"
            style="width: 260px"
            @change="loadData"
          />
          <a-radio-group v-model="filters.granularity" type="button" size="small" @change="loadData">
            <a-radio value="day">日</a-radio>
            <a-radio value="week">周</a-radio>
            <a-radio value="month">月</a-radio>
          </a-radio-group>
          <a-button @click="resetFilters">重置</a-button>
          <a-button type="primary" :loading="loading" @click="loadData">刷新</a-button>
        </a-space>
      </template>

      <div v-if="errorText" class="table-error">
        <span>{{ errorText }}</span>
        <a-button size="small" @click="loadData">重新加载</a-button>
      </div>

      <a-alert
        v-else
        class="dashboard-summary"
        type="info"
        :show-icon="false"
        :content="`统计范围：${generatedAt}`"
      />
    </PageCard>

    <a-skeleton v-if="loading && !dashboard" animation :loading="true">
      <a-space direction="vertical" fill>
        <a-skeleton-line :rows="4" />
      </a-space>
    </a-skeleton>

    <template v-else>
      <div class="metric-grid dashboard-metrics">
        <div v-for="item in metrics" :key="item.key" class="metric-card" :class="`metric-card-${item.tone}`">
          <div>
            <div class="metric-label">{{ item.label }}</div>
            <div class="metric-value">{{ item.value }}</div>
          </div>
          <div class="metric-helper">{{ item.helper }}</div>
        </div>
      </div>

      <div class="dashboard-grid">
        <PageCard>
          <template #title>订单与收入趋势</template>
          <template #extra><a-tag color="arcoblue">{{ trend.length }} 个周期</a-tag></template>
          <a-empty v-if="!trend.length" class="table-empty" description="暂无趋势数据；可调整时间范围后重试" />
          <div v-else class="trend-chart" role="img" aria-label="订单数量和收入趋势">
            <div v-for="item in trend" :key="read(item, 'date')" class="trend-item">
              <div class="trend-bars">
                <span
                  class="trend-bar trend-bar-orders"
                  :style="{ height: `${Math.max(8, (Number(read(item, 'orderCount', 'order_count') || 0) / maxOrderCount) * 100)}%` }"
                  :title="`订单 ${formatNumber(read(item, 'orderCount', 'order_count'))}`"
                />
                <span
                  class="trend-bar trend-bar-revenue"
                  :style="{ height: `${Math.max(8, (Number(read(item, 'paidAmount', 'paid_amount') || 0) / maxRevenue) * 100)}%` }"
                  :title="`收入 ${formatAmount(read(item, 'paidAmount', 'paid_amount'))}`"
                />
              </div>
              <strong>{{ read(item, 'date') }}</strong>
              <small>{{ formatNumber(read(item, 'orderCount', 'order_count')) }} 单</small>
            </div>
          </div>
          <div v-if="trend.length" class="chart-legend">
            <span><i class="legend-dot legend-orders" />订单数</span>
            <span><i class="legend-dot legend-revenue" />实付收入</span>
          </div>
        </PageCard>

        <PageCard>
          <template #title>订单状态分布</template>
          <template #extra><a-tag>共 {{ formatNumber(statusTotal) }} 单</a-tag></template>
          <a-empty v-if="!statusBreakdown.length" class="table-empty" description="暂无状态数据" />
          <div v-else class="status-list">
            <div v-for="item in statusBreakdown" :key="item.status" class="status-row">
              <span class="status-name">
                <a-tag :color="item.color">{{ item.label }}</a-tag>
              </span>
              <div class="status-track">
                <span
                  :class="['status-fill', `status-fill-${item.color}`]"
                  :style="{ width: `${statusTotal ? (item.count / statusTotal) * 100 : 0}%` }"
                />
              </div>
              <strong>{{ formatNumber(item.count) }}</strong>
            </div>
          </div>
        </PageCard>
      </div>

      <div class="dashboard-grid">
        <PageCard>
          <template #title>热销商品</template>
          <a-table :data="topProducts" :loading="loading" :pagination="false" row-key="id" size="small">
            <template #empty>
              <a-empty class="table-empty" description="暂无热销商品数据；当前范围内可能没有订单明细" />
            </template>
            <a-table-column title="商品" :ellipsis="true" :tooltip="true">
              <template #cell="{ record }">
                <span class="strong-text">{{ read(record, 'name') || '-' }}</span>
              </template>
            </a-table-column>
            <a-table-column title="销量" :width="100">
              <template #cell="{ record }">{{ formatNumber(read(record, 'salesCount', 'sales_count')) }}</template>
            </a-table-column>
            <a-table-column title="库存" :width="100">
              <template #cell="{ record }">
                <a-tag :color="Number(read(record, 'stockQuantity', 'stock_quantity') || 0) <= 10 ? 'orange' : 'green'">
                  {{ formatNumber(read(record, 'stockQuantity', 'stock_quantity')) }}
                </a-tag>
              </template>
            </a-table-column>
            <a-table-column title="最低售价" :width="120">
              <template #cell="{ record }">{{ formatAmount(read(record, 'minSalePrice', 'min_sale_price')) }}</template>
            </a-table-column>
          </a-table>
        </PageCard>

        <PageCard>
          <template #title>履约提醒</template>
          <div class="ops-checklist">
            <div class="ops-check-item">
              <span>待处理订单</span>
              <strong>{{ formatNumber(field('pendingOrders', 'pending_orders')) }}</strong>
            </div>
            <div class="ops-check-item">
              <span>待发货订单</span>
              <strong>{{ formatNumber(field('shippingOrders', 'shipping_orders')) }}</strong>
            </div>
            <div class="ops-check-item">
              <span>低库存 SKU</span>
              <strong>{{ formatNumber(field('lowStockSkus', 'low_stock_skus')) }}</strong>
            </div>
            <div class="ops-check-item">
              <span>今日收入</span>
              <strong>{{ formatAmount(field('todayRevenue', 'today_revenue')) }}</strong>
            </div>
          </div>
        </PageCard>
      </div>

      <PageCard>
        <template #title>最新订单</template>
        <template #extra><a-button :loading="loading" @click="loadData">刷新</a-button></template>
        <a-table :data="recentOrders" :loading="loading" :pagination="false" row-key="id">
          <template #empty>
            <a-empty class="table-empty" description="暂无最新订单；接口已联通但当前没有返回订单记录" />
          </template>
          <a-table-column title="订单号" :width="190">
            <template #cell="{ record }">{{ read(record, 'orderNo', 'order_no') || '-' }}</template>
          </a-table-column>
          <a-table-column title="客户" :width="170">
            <template #cell="{ record }">
              <div class="strong-text">{{ formatCustomer(record) }}</div>
              <div class="muted-text">{{ read(record, 'customerPhone', 'customer_phone') || '-' }}</div>
            </template>
          </a-table-column>
          <a-table-column title="商品" :ellipsis="true" :tooltip="true">
            <template #cell="{ record }">{{ read(record, 'goodsSummary', 'goods_summary') || '-' }}</template>
          </a-table-column>
          <a-table-column title="件数" :width="90">
            <template #cell="{ record }">{{ formatNumber(read(record, 'itemCount', 'item_count')) }}</template>
          </a-table-column>
          <a-table-column title="实付金额" :width="130">
            <template #cell="{ record }">{{ formatAmount(read(record, 'paidAmount', 'paid_amount')) }}</template>
          </a-table-column>
          <a-table-column title="状态" :width="120">
            <template #cell="{ record }">
              <a-tag :color="statusColor(read(record, 'status'))">{{ statusLabel(record) }}</a-tag>
            </template>
          </a-table-column>
          <a-table-column title="下单时间" :width="190">
            <template #cell="{ record }">{{ read(record, 'createdAt', 'created_at') || '-' }}</template>
          </a-table-column>
        </a-table>
      </PageCard>
    </template>
  </div>
</template>
