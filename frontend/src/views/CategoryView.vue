<script setup>
import { computed, onMounted, reactive, ref } from 'vue'
import { Message, Modal } from '@arco-design/web-vue'
import {
  IconDelete,
  IconEdit,
  IconPlus,
  IconRefresh,
  IconSearch,
  IconTags
} from '@arco-design/web-vue/es/icon'
import { createCategory, deleteCategory, fetchCategories, updateCategory } from '@/api/admin'
import PageCard from '@/components/PageCard.vue'

const loading = ref(false)
const saving = ref(false)
const visible = ref(false)
const editing = ref(null)
const list = ref([])
const errorText = ref('')
const formRef = ref(null)
const filters = reactive({ keyword: '', status: 'all' })
const form = reactive({ name: '', sort_order: 0, status: true })
const rules = {
  name: [
    { required: true, message: '请输入分类名称' },
    { minLength: 2, message: '分类名称至少 2 个字符' }
  ]
}
const columns = [
  { title: '分类', dataIndex: 'name', slotName: 'name', width: 260 },
  { title: '排序', dataIndex: 'sort_order', slotName: 'sortOrder', width: 120 },
  { title: '绑定商品', dataIndex: 'goods_count', slotName: 'goodsCount', width: 130 },
  { title: '状态', dataIndex: 'status', slotName: 'status', width: 120 },
  { title: '操作', slotName: 'actions', width: 190, fixed: 'right' }
]

const totalGoods = computed(() => list.value.reduce((sum, item) => sum + Number(read(item, 'goods_count', 'goodsCount') || 0), 0))
const enabledCount = computed(() => list.value.filter((item) => Boolean(read(item, 'status'))).length)
const disabledCount = computed(() => list.value.length - enabledCount.value)

const filteredList = computed(() => {
  const text = filters.keyword.trim().toLowerCase()
  return list.value.filter((item) => {
    const nameMatched = !text || String(read(item, 'name') || '').toLowerCase().includes(text)
    const status = Boolean(read(item, 'status'))
    const statusMatched = filters.status === 'all' || (filters.status === 'enabled' ? status : !status)
    return nameMatched && statusMatched
  })
})

function read(source, ...keys) {
  if (!source) return undefined
  for (const key of keys) {
    if (source[key] !== undefined && source[key] !== null) return source[key]
  }
  return undefined
}

function resetForm(record = null) {
  editing.value = record
  form.name = read(record, 'name') || ''
  form.sort_order = read(record, 'sort_order', 'sortOrder') ?? 0
  form.status = read(record, 'status') ?? true
  formRef.value?.clearValidate?.()
}

async function loadData() {
  loading.value = true
  errorText.value = ''
  try {
    const data = await fetchCategories()
    list.value = Array.isArray(data) ? data : []
  } catch (error) {
    errorText.value = error.message || '分类加载失败'
    Message.error(errorText.value)
  } finally {
    loading.value = false
  }
}

function openCreate() {
  resetForm()
  visible.value = true
}

function openEdit(record) {
  resetForm(record)
  visible.value = true
}

function resetFilters() {
  filters.keyword = ''
  filters.status = 'all'
}

async function saveCategory() {
  const validation = await formRef.value?.validate()
  if (validation) return
  saving.value = true
  try {
    const payload = {
      name: form.name.trim(),
      sort_order: Number(form.sort_order) || 0,
      status: Boolean(form.status)
    }
    if (editing.value) await updateCategory(read(editing.value, 'id'), payload)
    else await createCategory(payload)
    Message.success(editing.value ? '分类已更新' : '分类已创建')
    visible.value = false
    await loadData()
  } catch (error) {
    Message.error(error.message || '保存失败')
  } finally {
    saving.value = false
  }
}

function confirmDelete(record) {
  const count = Number(read(record, 'goods_count', 'goodsCount') || 0)
  Modal.confirm({
    title: '确认删除分类',
    content: count > 0
      ? `分类「${read(record, 'name')}」下还有 ${count} 个商品，后端会拒绝删除。请先迁移商品后再操作。`
      : `分类「${read(record, 'name')}」删除后不可恢复。`,
    okText: '删除',
    okButtonProps: { status: 'danger' },
    async onOk() {
      try {
        await deleteCategory(read(record, 'id'))
        Message.success('分类已删除')
        await loadData()
      } catch (error) {
        Message.error(error.message || '删除失败')
      }
    }
  })
}

onMounted(loadData)
</script>

<template>
  <div class="page-stack category-page">
    <div class="category-hero">
      <div>
        <div class="hero-kicker"><IconTags /> 商品类目</div>
        <h1>分类管理</h1>
        <p>维护商品的展示类目、排序和启停状态，保持前台货架结构清晰。</p>
      </div>
      <a-space>
        <a-button :loading="loading" @click="loadData">
          <template #icon><IconRefresh /></template>
          刷新
        </a-button>
        <a-button type="primary" @click="openCreate">
          <template #icon><IconPlus /></template>
          新建分类
        </a-button>
      </a-space>
    </div>

    <div class="category-stats">
      <div class="category-stat">
        <span>分类总数</span>
        <strong>{{ list.length }}</strong>
      </div>
      <div class="category-stat">
        <span>启用中</span>
        <strong>{{ enabledCount }}</strong>
      </div>
      <div class="category-stat">
        <span>已停用</span>
        <strong>{{ disabledCount }}</strong>
      </div>
      <div class="category-stat">
        <span>绑定商品</span>
        <strong>{{ totalGoods }}</strong>
      </div>
    </div>

    <PageCard>
      <template #title>分类列表</template>
      <template #extra>
        <a-tag color="arcoblue">当前 {{ filteredList.length }} 条</a-tag>
      </template>

      <div v-if="errorText" class="table-error">
        <span>{{ errorText }}</span>
        <a-button size="small" @click="loadData">重新加载</a-button>
      </div>

      <div class="toolbar category-toolbar">
        <a-input
          v-model="filters.keyword"
          allow-clear
          placeholder="搜索分类名称"
          class="category-search"
        >
          <template #prefix><IconSearch /></template>
        </a-input>
        <a-radio-group v-model="filters.status" type="button">
          <a-radio value="all">全部</a-radio>
          <a-radio value="enabled">启用</a-radio>
          <a-radio value="disabled">停用</a-radio>
        </a-radio-group>
        <a-button @click="resetFilters">重置</a-button>
      </div>

      <a-table
        :columns="columns"
        :data="filteredList"
        :loading="loading"
        row-key="id"
        :pagination="{ pageSize: 10, showTotal: true }"
      >
        <template #empty>
          <a-empty class="table-empty" description="暂无分类数据，可点击右上角新建分类" />
        </template>
        <template #name="{ record }">
          <div class="category-name-cell">
            <span class="category-avatar"><IconTags /></span>
            <div>
              <span class="strong-text">{{ read(record, 'name') }}</span>
              <span class="muted-text">ID {{ read(record, 'id') }}</span>
            </div>
          </div>
        </template>
        <template #sortOrder="{ record }">
          <span class="number-pill">{{ read(record, 'sort_order', 'sortOrder') ?? 0 }}</span>
        </template>
        <template #goodsCount="{ record }">
          <a-tag :color="Number(read(record, 'goods_count', 'goodsCount') || 0) > 0 ? 'arcoblue' : 'gray'">
            {{ read(record, 'goods_count', 'goodsCount') ?? 0 }} 个
          </a-tag>
        </template>
        <template #status="{ record }">
          <a-tag :color="read(record, 'status') ? 'green' : 'gray'">
            {{ read(record, 'status') ? '启用' : '停用' }}
          </a-tag>
        </template>
        <template #actions="{ record }">
          <a-space>
            <a-tooltip content="编辑分类">
              <a-button size="small" @click="openEdit(record)">
                <template #icon><IconEdit /></template>
              </a-button>
            </a-tooltip>
            <a-tooltip content="删除分类">
              <a-button size="small" status="danger" @click="confirmDelete(record)">
                <template #icon><IconDelete /></template>
              </a-button>
            </a-tooltip>
          </a-space>
        </template>
      </a-table>
    </PageCard>

    <a-drawer
      v-model:visible="visible"
      :width="460"
      :title="editing ? '编辑分类' : '新建分类'"
      unmount-on-close
    >
      <a-form ref="formRef" :model="form" :rules="rules" layout="vertical">
        <a-form-item field="name" label="分类名称">
          <a-input v-model="form.name" placeholder="例如：健康食品、生活百货" allow-clear />
        </a-form-item>
        <a-form-item field="sort_order" label="排序值">
          <a-input-number v-model="form.sort_order" :min="0" style="width: 100%" />
          <template #help>数值越小越靠前。</template>
        </a-form-item>
        <a-form-item field="status" label="启用状态">
          <a-switch v-model="form.status" checked-text="启用" unchecked-text="停用" />
        </a-form-item>
      </a-form>
      <template #footer>
        <a-space>
          <a-button @click="visible = false">取消</a-button>
          <a-button type="primary" :loading="saving" @click="saveCategory">保存</a-button>
        </a-space>
      </template>
    </a-drawer>
  </div>
</template>
