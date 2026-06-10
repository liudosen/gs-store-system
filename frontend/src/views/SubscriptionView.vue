<script setup>
import { computed, reactive, ref } from 'vue'
import { Message, Modal } from '@arco-design/web-vue'
import { fetchBalanceTransactions, triggerAutoRecharge } from '@/api/admin'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const recharging = ref(false)
const loading = ref(false)
const balanceInfo = ref(null)
const form = reactive({ identity_no: '' })

const canExecuteAutoRecharge = computed(() => auth.accessCodes.includes('subscription:auto-recharge:execute'))

function read(record, camelKey, snakeKey, fallback = '') {
  if (!record) return fallback
  return record[camelKey] ?? record[snakeKey] ?? fallback
}

function money(value) {
  return `¥${(Number(value || 0) / 100).toFixed(2)}`
}

function statusLabel(status) {
  return Number(status) === 1 ? '成功' : '失败'
}

function statusColor(status) {
  return Number(status) === 1 ? 'green' : 'red'
}

function autoRecharge() {
  Modal.confirm({
    title: '确认执行自动充值',
    content: '系统将扫描当前已开启订阅且符合充值条件的用户，并发起自动充值。',
    async onOk() {
      recharging.value = true
      try {
        await triggerAutoRecharge()
        Message.success('自动充值任务已触发')
      } catch (error) {
        Message.error(error.message || '自动充值触发失败')
      } finally {
        recharging.value = false
      }
    }
  })
}

async function queryBalance() {
  const identityNo = form.identity_no.trim()
  if (!identityNo) {
    Message.warning('请输入认证号')
    return
  }

  loading.value = true
  balanceInfo.value = null
  try {
    balanceInfo.value = await fetchBalanceTransactions(identityNo)
  } catch (error) {
    Message.error(error.message || '余额流水加载失败')
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="page-stack subscription-page">
    <a-card :bordered="false" class="page-card">
      <template #title>订阅与充值</template>

      <div class="subscription-actions">
        <section class="operation-panel">
          <strong>自动充值</strong>
          <p>对当前最终状态为“开启”的订阅用户执行充值任务，执行结果会写入订单与余额流水。</p>
          <a-button v-if="canExecuteAutoRecharge" type="primary" :loading="recharging" @click="autoRecharge">开始执行</a-button>
          <a-alert v-else type="warning">当前账号没有“执行自动充值”按钮权限。</a-alert>
        </section>
        <section class="operation-panel">
          <strong>余额流水查询</strong>
          <p>输入认证号查看当前余额与最近充值、扣款流水。</p>
          <a-space wrap>
            <a-input-search
              v-model="form.identity_no"
              placeholder="健康卡权益号/身份证号"
              allow-clear
              style="width: 320px"
              @search="queryBalance"
            />
            <a-button :loading="loading" @click="queryBalance">查询流水</a-button>
          </a-space>
        </section>
      </div>
    </a-card>

    <a-card :bordered="false" class="page-card">
      <template #title>余额流水</template>
      <a-spin :loading="loading" style="width: 100%">
        <a-empty v-if="!balanceInfo" class="table-empty" description="请输入认证号后查询余额流水" />
        <template v-else>
          <a-descriptions :column="2" bordered>
            <a-descriptions-item label="认证号">{{ form.identity_no }}</a-descriptions-item>
            <a-descriptions-item label="当前余额">{{ money(read(balanceInfo, 'balance', 'balance', 0)) }}</a-descriptions-item>
          </a-descriptions>
          <a-divider>流水明细</a-divider>
          <div class="management-table-wrap">
            <table class="management-table balance-table">
              <thead>
                <tr>
                  <th>金额</th>
                  <th>变动后余额</th>
                  <th>状态</th>
                  <th>外部单号</th>
                  <th>备注</th>
                  <th>时间</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="tx in read(balanceInfo, 'transactions', 'transactions', [])" :key="read(tx, 'id', 'id')">
                  <td><span class="strong-text">{{ money(read(tx, 'amount', 'amount', 0)) }}</span></td>
                  <td>{{ money(read(tx, 'balanceAfter', 'balance_after', 0)) }}</td>
                  <td><a-tag :color="statusColor(read(tx, 'status', 'status', 0))">{{ statusLabel(read(tx, 'status', 'status', 0)) }}</a-tag></td>
                  <td>{{ read(tx, 'externalOrderNo', 'external_order_no') || '-' }}</td>
                  <td>{{ read(tx, 'remark', 'remark') || '-' }}</td>
                  <td>{{ read(tx, 'createdAt', 'created_at') || '-' }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </a-spin>
    </a-card>
  </div>
</template>
