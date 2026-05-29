<script setup>
import { computed, onMounted, reactive, ref } from 'vue'
import { Message } from '@arco-design/web-vue'
import { fetchOrder, fetchOrders, updateOrderStatus } from '@/api/admin'

const loading = ref(false)
const detailLoading = ref(false)
const editLoading = ref(false)
const saving = ref(false)
const detailVisible = ref(false)
const editVisible = ref(false)
const errorText = ref('')
const detailError = ref('')
const list = ref([])
const total = ref(0)
const detail = ref(null)
const editingOrder = ref(null)

const filters = reactive({ page: 1, page_size: 10, status: undefined, order_no: '', openid: '' })
const editForm = reactive({
  status: 1,
  carrier: '',
  tracking_no: '',
  delivery_name: '',
  delivery_phone: '',
  logistics_remark: ''
})

const statuses = [
  { label: '待付款', value: 0 },
  { label: '待发货', value: 1 },
  { label: '待收货', value: 2 },
  { label: '已完成', value: 3 },
  { label: '已取消', value: 4 }
]

const statusOptions = computed(() => statuses)
const showLogisticsForm = computed(() => Number(editForm.status) === 2)

function read(record, camelKey, snakeKey, fallback = '') {
  if (!record) return fallback
  return record[camelKey] ?? record[snakeKey] ?? fallback
}

function money(value) {
  return `¥${(Number(value || 0) / 100).toFixed(2)}`
}

function statusLabel(record) {
  const status = Number(read(record, 'status', 'status'))
  return read(record, 'statusLabel', 'status_label') || statuses.find((item) => item.value === status)?.label || '-'
}

function statusColor(status) {
  return ['orange', 'blue', 'purple', 'green', 'gray'][Number(status)] || 'gray'
}

function orderItems(record) {
  return read(record, 'items', 'items', []) || []
}

function logistics(record) {
  return read(record, 'logistics', 'logistics', null)
}

function goodsSummary(record) {
  const items = orderItems(record)
  if (!items.length) return read(record, 'remark', 'remark', '-') || '-'
  const first = items[0]
  const title = read(first, 'goodsTitle', 'goods_title', '-')
  const quantity = read(first, 'quantity', 'quantity', 0)
  return items.length > 1 ? `${title} x${quantity} 等 ${items.length} 件` : `${title} x${quantity}`
}

function formatSpecInfo(item) {
  let specInfo = read(item, 'specInfo', 'spec_info', [])
  if (typeof specInfo === 'string') {
    try {
      specInfo = JSON.parse(specInfo)
    } catch {
      const text = specInfo.trim()
      return text.startsWith('{') || text.startsWith('[') || text.includes('spec') ? '-' : text || '-'
    }
  }

  if (Array.isArray(specInfo)) {
    const specs = specInfo
      .map((spec) => {
        if (!spec || typeof spec !== 'object') return ''
        const name = spec.specName || spec.spec_name || spec.name || ''
        const value = spec.specValue || spec.spec_value || spec.value || ''
        return name && value ? `${name}: ${value}` : value || name
      })
      .filter(Boolean)
    return specs.length ? specs.join(' / ') : '-'
  }

  if (specInfo && typeof specInfo === 'object') {
    const specs = Object.entries(specInfo)
      .map(([name, value]) => (value ? `${name}: ${value}` : name))
      .filter(Boolean)
    return specs.length ? specs.join(' / ') : '-'
  }

  return '-'
}

function logisticsSummary(record) {
  const info = logistics(record)
  if (!info) return '未填写'
  const trackingNo = read(info, 'trackingNo', 'tracking_no', '')
  const deliveryPhone = read(info, 'deliveryPhone', 'delivery_phone', '')
  const carrier = read(info, 'carrier', 'carrier', '')
  if (trackingNo) return carrier ? `${carrier} · ${trackingNo}` : trackingNo
  if (deliveryPhone) return `派件电话 ${deliveryPhone}`
  return '未填写'
}

function addressText(address) {
  if (!address) return '-'
  const province = read(address, 'province', 'province', '')
  const city = read(address, 'city', 'city', '')
  const district = read(address, 'district', 'district', '')
  const detailAddress = read(address, 'detailAddress', 'detail_address', '')
  return `${province}${city}${district}${detailAddress}` || '-'
}

function cleanParams() {
  return Object.fromEntries(
    Object.entries(filters).filter(([, value]) => value !== '' && value !== undefined && value !== null)
  )
}

function resetFilters() {
  Object.assign(filters, { page: 1, status: undefined, order_no: '', openid: '' })
  loadData()
}

async function loadData() {
  loading.value = true
  errorText.value = ''
  try {
    const data = await fetchOrders(cleanParams())
    list.value = data?.list || []
    total.value = data?.total || 0
  } catch (error) {
    errorText.value = error.message || '订单加载失败'
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

async function showDetail(record) {
  detailVisible.value = true
  detail.value = null
  detailError.value = ''
  detailLoading.value = true
  try {
    detail.value = await fetchOrder(read(record, 'id', 'id'))
  } catch (error) {
    detailError.value = error.message || '详情加载失败'
    Message.error(detailError.value)
  } finally {
    detailLoading.value = false
  }
}

function fillEditForm(order) {
  const info = logistics(order) || {}
  editForm.status = Number(read(order, 'status', 'status', 1))
  editForm.carrier = read(info, 'carrier', 'carrier', '')
  editForm.tracking_no = read(info, 'trackingNo', 'tracking_no', '')
  editForm.delivery_name = read(info, 'deliveryName', 'delivery_name', '')
  editForm.delivery_phone = read(info, 'deliveryPhone', 'delivery_phone', '')
  editForm.logistics_remark = read(info, 'remark', 'remark', '')
}

async function openEdit(record) {
  editVisible.value = true
  editLoading.value = true
  editingOrder.value = record
  fillEditForm(record)
  try {
    const order = await fetchOrder(read(record, 'id', 'id'))
    editingOrder.value = order
    fillEditForm(order)
  } catch (error) {
    Message.warning(error.message || '订单详情加载失败，已使用列表数据')
  } finally {
    editLoading.value = false
  }
}

async function saveEdit() {
  if (!editingOrder.value) return
  if (showLogisticsForm.value && !editForm.tracking_no.trim() && !editForm.delivery_phone.trim()) {
    Message.warning('修改为待收货时，需要填写物流单号或派件人手机号')
    return
  }

  saving.value = true
  try {
    await updateOrderStatus(read(editingOrder.value, 'id', 'id'), {
      status: Number(editForm.status),
      carrier: editForm.carrier.trim(),
      tracking_no: editForm.tracking_no.trim(),
      delivery_name: editForm.delivery_name.trim(),
      delivery_phone: editForm.delivery_phone.trim(),
      logistics_remark: editForm.logistics_remark.trim()
    })
    Message.success('订单已更新')
    editVisible.value = false
    await loadData()
    if (detailVisible.value && detail.value) {
      detail.value = await fetchOrder(read(editingOrder.value, 'id', 'id'))
    }
  } catch (error) {
    Message.error(error.message || '订单更新失败')
  } finally {
    saving.value = false
  }
}

onMounted(loadData)
</script>

<template>
  <div class="page-stack order-page">
    <a-card :bordered="false" class="page-card">
      <template #title>订单管理</template>
      <template #extra>
        <a-button :loading="loading" @click="loadData">刷新</a-button>
      </template>

      <div v-if="errorText" class="table-error">
        <span>{{ errorText }}</span>
        <a-button size="small" @click="loadData">重新加载</a-button>
      </div>

      <a-space wrap class="toolbar">
        <a-input-search v-model="filters.order_no" placeholder="订单号" allow-clear style="width: 220px" @search="search" />
        <a-input v-model="filters.openid" placeholder="用户 openid" allow-clear style="width: 220px" />
        <a-select v-model="filters.status" placeholder="订单状态" allow-clear style="width: 140px">
          <a-option v-for="item in statusOptions" :key="item.value" :value="item.value">{{ item.label }}</a-option>
        </a-select>
        <a-button type="primary" @click="search">查询</a-button>
        <a-button @click="resetFilters">重置</a-button>
        <a-tag color="arcoblue">共 {{ total }} 条</a-tag>
      </a-space>

      <a-spin :loading="loading" style="width: 100%">
        <div v-if="!list.length" class="table-empty">
          <a-empty description="暂无订单数据" />
        </div>
        <div v-else class="order-table-wrap">
          <table class="order-table">
            <thead>
              <tr>
                <th>订单号</th>
                <th>外部订单号</th>
                <th>用户</th>
                <th>商品摘要</th>
                <th>金额</th>
                <th>状态</th>
                <th>物流</th>
                <th>下单时间</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="record in list" :key="read(record, 'id', 'id')">
                <td>
                  <div class="order-no-cell">
                    <strong>{{ read(record, 'orderNo', 'order_no') }}</strong>
                  </div>
                </td>
                <td>
                  <span class="order-openid">{{ read(record, 'externalOrderNo', 'external_order_no') || '-' }}</span>
                </td>
                <td>
                  <span class="order-openid">{{ read(record, 'openid', 'openid') || '-' }}</span>
                </td>
                <td>
                  <span class="order-summary">{{ goodsSummary(record) }}</span>
                </td>
                <td>
                  <span class="strong-text">{{ money(read(record, 'paidAmount', 'paid_amount', 0)) }}</span>
                </td>
                <td>
                  <a-tag :color="statusColor(read(record, 'status', 'status'))">{{ statusLabel(record) }}</a-tag>
                </td>
                <td>
                  <span class="order-logistics">{{ logisticsSummary(record) }}</span>
                </td>
                <td>
                  <span class="muted-text">{{ read(record, 'createdAt', 'created_at') || '-' }}</span>
                </td>
                <td>
                  <a-space>
                    <a-button size="small" @click="showDetail(record)">详情</a-button>
                    <a-button size="small" type="primary" @click="openEdit(record)">编辑</a-button>
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

    <a-drawer v-model:visible="detailVisible" width="760px" title="订单详情">
      <a-spin :loading="detailLoading" style="width: 100%">
        <a-result v-if="detailError" status="error" :subtitle="detailError">
          <template #extra>
            <a-button @click="detailVisible = false">关闭</a-button>
          </template>
        </a-result>

        <template v-else-if="detail">
          <a-descriptions :column="2" bordered>
            <a-descriptions-item label="订单号">{{ read(detail, 'orderNo', 'order_no') }}</a-descriptions-item>
            <a-descriptions-item label="外部订单">{{ read(detail, 'externalOrderNo', 'external_order_no') || '-' }}</a-descriptions-item>
            <a-descriptions-item label="用户">{{ read(detail, 'openid', 'openid') || '-' }}</a-descriptions-item>
            <a-descriptions-item label="状态">
              <a-tag :color="statusColor(read(detail, 'status', 'status'))">{{ statusLabel(detail) }}</a-tag>
            </a-descriptions-item>
            <a-descriptions-item label="总额">{{ money(read(detail, 'totalAmount', 'total_amount', 0)) }}</a-descriptions-item>
            <a-descriptions-item label="实付">{{ money(read(detail, 'paidAmount', 'paid_amount', 0)) }}</a-descriptions-item>
            <a-descriptions-item label="备注" :span="2">{{ read(detail, 'remark', 'remark') || '-' }}</a-descriptions-item>
          </a-descriptions>

          <a-divider>收货信息</a-divider>
          <a-descriptions :column="1" bordered>
            <a-descriptions-item label="收货人">
              {{ read(read(detail, 'address', 'address'), 'receiverName', 'receiver_name') || '-' }}
              {{ read(read(detail, 'address', 'address'), 'phone', 'phone') || '' }}
            </a-descriptions-item>
            <a-descriptions-item label="收货地址">{{ addressText(read(detail, 'address', 'address')) }}</a-descriptions-item>
          </a-descriptions>

          <a-divider>物流信息</a-divider>
          <a-descriptions :column="2" bordered>
            <a-descriptions-item label="物流公司">{{ read(logistics(detail), 'carrier', 'carrier') || '-' }}</a-descriptions-item>
            <a-descriptions-item label="物流单号">{{ read(logistics(detail), 'trackingNo', 'tracking_no') || '-' }}</a-descriptions-item>
            <a-descriptions-item label="派件人">{{ read(logistics(detail), 'deliveryName', 'delivery_name') || '-' }}</a-descriptions-item>
            <a-descriptions-item label="派件电话">{{ read(logistics(detail), 'deliveryPhone', 'delivery_phone') || '-' }}</a-descriptions-item>
            <a-descriptions-item label="物流备注" :span="2">{{ read(logistics(detail), 'remark', 'remark') || '-' }}</a-descriptions-item>
          </a-descriptions>

          <a-divider>商品明细</a-divider>
          <div class="order-items-table-wrap">
            <table class="order-items-table">
              <thead>
                <tr>
                  <th>商品</th>
                  <th>规格</th>
                  <th>数量</th>
                  <th>单价</th>
                  <th>小计</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in orderItems(detail)" :key="read(item, 'id', 'id')">
                  <td>
                    <div class="order-item-product">
                      <a-avatar
                        class="order-item-image"
                        shape="square"
                        :image-url="read(item, 'goodsImage', 'goods_image')"
                      >
                        {{ String(read(item, 'goodsTitle', 'goods_title') || 'P').slice(0, 1) }}
                      </a-avatar>
                      <div class="order-item-copy">
                        <strong>{{ read(item, 'goodsTitle', 'goods_title') || '-' }}</strong>
                      </div>
                    </div>
                  </td>
                  <td>{{ formatSpecInfo(item) }}</td>
                  <td>{{ read(item, 'quantity', 'quantity') }}</td>
                  <td>{{ money(read(item, 'unitPrice', 'unit_price', 0)) }}</td>
                  <td>{{ money(read(item, 'subtotal', 'subtotal', 0)) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </a-spin>
    </a-drawer>

    <a-modal v-model:visible="editVisible" title="编辑订单" :footer="false" width="560px" unmount-on-close>
      <a-spin :loading="editLoading" style="width: 100%">
        <a-form :model="editForm" layout="vertical">
          <a-alert v-if="editingOrder" type="info" class="order-edit-alert">
            当前订单：{{ read(editingOrder, 'orderNo', 'order_no') }}
          </a-alert>
          <a-form-item label="订单状态" required>
            <a-select v-model="editForm.status" placeholder="请选择订单状态">
              <a-option v-for="item in statusOptions" :key="item.value" :value="item.value">{{ item.label }}</a-option>
            </a-select>
          </a-form-item>

          <div v-if="showLogisticsForm" class="order-logistics-form">
            <a-form-item label="物流公司">
              <a-input v-model="editForm.carrier" placeholder="如 顺丰、京东、同城配送" allow-clear />
            </a-form-item>
            <a-form-item label="物流单号">
              <a-input v-model="editForm.tracking_no" placeholder="填写物流单号" allow-clear />
            </a-form-item>
            <a-form-item label="派件人">
              <a-input v-model="editForm.delivery_name" placeholder="填写派件人姓名" allow-clear />
            </a-form-item>
            <a-form-item label="派件人手机号">
              <a-input v-model="editForm.delivery_phone" placeholder="无物流单号时必须填写" allow-clear />
            </a-form-item>
            <a-form-item label="物流备注">
              <a-textarea v-model="editForm.logistics_remark" placeholder="可填写配送说明" :auto-size="{ minRows: 2, maxRows: 4 }" />
            </a-form-item>
          </div>
        </a-form>

        <div class="modal-actions">
          <a-button @click="editVisible = false">取消</a-button>
          <a-button type="primary" :loading="saving" @click="saveEdit">保存</a-button>
        </div>
      </a-spin>
    </a-modal>
  </div>
</template>
