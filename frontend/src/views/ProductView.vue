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
import {
  createGoods,
  deleteGoods,
  fetchCategories,
  fetchGoods,
  fetchGoodsDetail,
  fetchUploadSignature,
  updateGoods
} from '@/api/admin'
import PageCard from '@/components/PageCard.vue'

const loading = ref(false)
const saving = ref(false)
const visible = ref(false)
const editing = ref(null)
const list = ref([])
const total = ref(0)
const categories = ref([])
const errorText = ref('')
const formRef = ref(null)
const primaryImageFiles = ref([])
const carouselImageFiles = ref([])
const descImageFiles = ref([])

const filters = reactive({
  page: 1,
  page_size: 10,
  keyword: '',
  category_id: '',
  status: ''
})

const form = reactive({
  store_id: '',
  saas_id: '',
  title: '',
  primary_image: '',
  category_id: '',
  status: true,
  images_text: '',
  desc_images_text: '',
  tags_text: '',
  specRows: [],
  skus: []
})

const rules = {
  title: [{ required: true, message: '请输入商品标题' }],
  primary_image: [{ required: true, message: '请输入商品主图 URL' }]
}

const columns = [
  { title: '商品', dataIndex: 'title', slotName: 'product', width: 360, fixed: 'left' },
  { title: '分类', dataIndex: 'category_id', slotName: 'category', width: 150 },
  { title: '售价', dataIndex: 'min_sale_price', slotName: 'price', width: 130 },
  { title: '库存', dataIndex: 'spu_stock_quantity', slotName: 'stock', width: 120 },
  { title: '销量', dataIndex: 'sold_num', slotName: 'sold', width: 100 },
  { title: 'SKU', dataIndex: 'skus', slotName: 'skuCount', width: 100 },
  { title: '状态', dataIndex: 'status', slotName: 'status', width: 110 },
  { title: '操作', slotName: 'actions', width: 180, fixed: 'right' }
]

const categoryNameMap = computed(() => Object.fromEntries(categories.value.map((item) => [String(item.id), item.name])))
const activeCount = computed(() => list.value.filter((item) => item.status).length)
const soldOutCount = computed(() => list.value.filter((item) => item.is_sold_out || Number(item.spu_stock_quantity || 0) <= 0).length)
const pageStock = computed(() => list.value.reduce((sum, item) => sum + Number(item.spu_stock_quantity || 0), 0))
const pageSales = computed(() => list.value.reduce((sum, item) => sum + Number(item.sold_num || 0), 0))

function key() {
  return `${Date.now()}_${Math.random().toString(16).slice(2)}`
}

function money(value) {
  return `¥${(Number(value || 0) / 100).toFixed(2)}`
}

function yuanToCents(value) {
  return Math.round(Number(value || 0) * 100)
}

function centsToYuan(value) {
  return Number((Number(value || 0) / 100).toFixed(2))
}

function splitLines(text) {
  return String(text || '').split(/\n|,/).map((item) => item.trim()).filter(Boolean)
}

function splitValues(text) {
  return String(text || '').split(/\n|,|，|;|；|、/).map((item) => item.trim()).filter(Boolean)
}

function uniqueValues(values) {
  return Array.from(new Set((values || []).map((item) => String(item || '').trim()).filter(Boolean)))
}

function specValueItemsFromValues(values, rowIndex = 0) {
  return uniqueValues(values).map((value, valueIndex) => ({
    key: key(),
    id: `sv_${rowIndex + 1}_${valueIndex + 1}`,
    value
  }))
}

function normalizeSpecValueItems(row) {
  row.valueItems = (row.valueItems || []).map((item) => ({
    key: item.key || key(),
    id: item.id || '',
    value: String(item.value || '').trim()
  }))
  row.values = uniqueValues(row.valueItems.map((item) => item.value))
  row.values_text = row.values.join(',')
}

function fileNameFromUrl(url, fallback) {
  try {
    const pathname = new URL(url).pathname
    return decodeURIComponent(pathname.split('/').pop() || fallback)
  } catch {
    return String(url || fallback).split('/').pop() || fallback
  }
}

function filesFromUrls(urls, prefix) {
  return (urls || []).filter(Boolean).map((url, index) => ({
    uid: `${prefix}_${index}_${key()}`,
    name: fileNameFromUrl(url, `图片${index + 1}`),
    status: 'done',
    percent: 1,
    url,
    response: { url }
  }))
}

function fileUrl(file) {
  return file?.response?.url || file?.url || ''
}

function imageUrlsFromFiles(files) {
  return Array.from(new Set((files || []).map(fileUrl).filter(Boolean)))
}

function syncPrimaryImage(fileList = primaryImageFiles.value) {
  primaryImageFiles.value = fileList
  form.primary_image = imageUrlsFromFiles(fileList)[0] || ''
}

function syncCarouselImages(fileList = carouselImageFiles.value) {
  carouselImageFiles.value = fileList
  form.images_text = imageUrlsFromFiles(fileList).join('\n')
}

function syncDescImages(fileList = descImageFiles.value) {
  descImageFiles.value = fileList
  form.desc_images_text = imageUrlsFromFiles(fileList).join('\n')
}

function syncSkuImage(sku, fileList = sku.sku_image_files || []) {
  sku.sku_image_files = fileList
  sku.sku_image = imageUrlsFromFiles(fileList)[0] || ''
}

function validateImageFile(file) {
  if (!file?.type?.startsWith('image/')) {
    Message.error('只能上传图片文件')
    return false
  }
  if (file.size > 10 * 1024 * 1024) {
    Message.error('单张图片不能超过 10MB')
    return false
  }
  return true
}

function uploadImageRequest({ fileItem, onProgress, onSuccess, onError }) {
  const xhr = new XMLHttpRequest()
  let aborted = false

  async function upload() {
    try {
      const file = fileItem.file
      if (!file) throw new Error('请选择图片文件')
      const signature = await fetchUploadSignature(file.name)
      if (aborted) return

      const uploadHost = signature.host || signature.url
      const imageUrl = `${String(uploadHost).replace(/\/$/, '')}/${signature.key}`
      const formData = new FormData()
      formData.append('key', signature.key)
      formData.append('policy', signature.policy)
      formData.append('OSSAccessKeyId', signature.OSSAccessKeyId)
      formData.append('success_action_status', '200')
      formData.append('signature', signature.signature)
      formData.append('file', file)

      xhr.upload.onprogress = (event) => {
        if (event.total > 0) onProgress(Number((event.loaded / event.total).toFixed(2)), event)
      }
      xhr.onerror = () => onError(new Error('图片上传失败'))
      xhr.onload = () => {
        if (xhr.status >= 200 && xhr.status < 300) onSuccess({ url: imageUrl, key: signature.key })
        else onError(new Error('图片上传失败'))
      }
      xhr.open('POST', uploadHost, true)
      xhr.send(formData)
    } catch (error) {
      onError(error)
    }
  }

  upload()

  return {
    abort() {
      aborted = true
      xhr.abort()
    }
  }
}

function handleImageUploadError() {
  Message.error('图片上传失败，请稍后重试')
}

function handlePrimaryImageChange(fileList) {
  syncPrimaryImage(fileList)
  formRef.value?.clearValidate?.('primary_image')
}

function handleCarouselImageChange(fileList) {
  syncCarouselImages(fileList)
}

function handleDescImageChange(fileList) {
  syncDescImages(fileList)
}

function normalizeSpecRowValues(row, values = row.values) {
  row.valueItems = specValueItemsFromValues(values)
  normalizeSpecValueItems(row)
}

function syncSpecRows() {
  form.specRows.forEach((row) => {
    if (!row.valueItems?.length && row.values_text) {
      row.valueItems = specValueItemsFromValues(splitValues(row.values_text))
    }
    normalizeSpecValueItems(row)
  })
}

function hasSpecDraft() {
  return form.specRows.some((row) => row.title.trim() || (row.valueItems || []).some((item) => item.value.trim()))
}

function addSpecValue(row) {
  row.valueItems.push({ key: key(), id: '', value: '' })
}

function handleSpecValueEnter(row, index) {
  const item = row.valueItems[index]
  if (item?.value?.trim()) {
    addSpecValue(row)
  }
}

function removeSpecValue(row, index) {
  row.valueItems.splice(index, 1)
  normalizeSpecValueItems(row)
}

function categoryName(record) {
  return categoryNameMap.value[String(record.category_id)] || record.category_id || '-'
}

function stockColor(value) {
  const stock = Number(value || 0)
  if (stock <= 0) return 'red'
  if (stock <= 10) return 'orange'
  return 'green'
}

function cleanParams() {
  const params = {
    page: filters.page,
    page_size: filters.page_size,
    keyword: filters.keyword.trim(),
    category_id: filters.category_id,
    status: filters.status
  }
  return Object.fromEntries(Object.entries(params).filter(([, value]) => value !== '' && value !== undefined && value !== null))
}

function makeDefaultSku() {
  return {
    key: key(),
    sku_image: '',
    sku_image_files: [],
    spec_info: [],
    sale_price_yuan: 0,
    line_price_yuan: 0,
    stock_quantity: 0
  }
}

function ensureDefaultSku() {
  if (!form.skus.length) {
    form.skus = [makeDefaultSku()]
  }
  return form.skus[0]
}

function collapseToDefaultSku() {
  const old = ensureDefaultSku()
  form.skus = [{
    ...makeDefaultSku(),
    ...old,
    key: old.key || key(),
    spec_info: [],
    sku_image_files: old.sku_image_files || filesFromUrls(old.sku_image ? [old.sku_image] : [], 'sku')
  }]
  return form.skus[0]
}

function skuLabel(sku) {
  const label = (sku.spec_info || []).map((item) => item.specValue || item.spec_value || item.specValueId || item.spec_value_id).filter(Boolean).join(' / ')
  return label || '默认规格'
}

function skuComboKey(sku) {
  const parts = (sku.spec_info || []).map((item) => {
    const specId = item.specId || item.spec_id || ''
    const valueId = item.specValueId || item.spec_value_id || item.specValue || item.spec_value || ''
    return `${specId}:${valueId}`
  })
  return parts.join('|') || 'default'
}

function resetForm(record = null) {
  editing.value = record
  Object.assign(form, {
    store_id: record?.store_id || '',
    saas_id: record?.saas_id || '',
    title: record?.title || '',
    primary_image: record?.primary_image || '',
    category_id: record?.category_id || '',
    status: record?.status ?? true,
    images_text: (record?.images || []).join('\n'),
    desc_images_text: (record?.desc_images || []).join('\n'),
    tags_text: (record?.spu_tag_list || []).map((item) => item.title).join('\n')
  })

  primaryImageFiles.value = filesFromUrls(form.primary_image ? [form.primary_image] : [], 'primary')
  carouselImageFiles.value = filesFromUrls(record?.images || [], 'carousel')
  descImageFiles.value = filesFromUrls(record?.desc_images || [], 'desc')

  form.specRows = (record?.spec_list || []).map((spec, index) => ({
    key: key(),
    spec_id: spec.specId || `spec_${index + 1}`,
    title: spec.title || '',
    values: (spec.specValueList || []).map((item) => item.specValue).filter(Boolean),
    values_text: (spec.specValueList || []).map((item) => item.specValue).join('，'),
    valueItems: (spec.specValueList || []).map((item, idx) => ({
      key: key(),
      id: item.specValueId || `sv_${index + 1}_${idx + 1}`,
      value: item.specValue || ''
    })),
    valueMap: Object.fromEntries((spec.specValueList || []).map((item, idx) => [item.specValue, item.specValueId || `sv_${index + 1}_${idx + 1}`]))
  }))

  form.skus = (record?.skus || []).map((sku) => ({
    key: key(),
    sku_image: sku.sku_image || '',
    sku_image_files: filesFromUrls(sku.sku_image ? [sku.sku_image] : [], 'sku'),
    spec_info: sku.spec_info || [],
    sale_price_yuan: centsToYuan(sku.sale_price),
    line_price_yuan: centsToYuan(sku.line_price),
    stock_quantity: sku.stock_quantity || 0
  }))

  if (!form.skus.length) {
    form.skus = [{
      ...makeDefaultSku(),
      sku_image: record?.primary_image || '',
      sku_image_files: filesFromUrls(record?.primary_image ? [record.primary_image] : [], 'sku'),
      sale_price_yuan: centsToYuan(record?.min_sale_price),
      line_price_yuan: centsToYuan(record?.max_line_price),
      stock_quantity: record?.spu_stock_quantity || 0
    }]
  }
  formRef.value?.clearValidate?.()
}

function addSpecRow() {
  form.specRows.push({
    key: key(),
    spec_id: `spec_${form.specRows.length + 1}`,
    title: '',
    values: [],
    values_text: '',
    valueItems: [{ key: key(), id: '', value: '' }],
    valueMap: {}
  })
}

function removeSpecRow(index) {
  form.specRows.splice(index, 1)
  if (!form.specRows.length) collapseToDefaultSku()
}

function buildSpecList() {
  syncSpecRows()
  return form.specRows
    .map((row, rowIndex) => {
      const values = uniqueValues((row.valueItems || []).map((item) => item.value))
      return {
        specId: row.spec_id || `spec_${rowIndex + 1}`,
        title: row.title.trim(),
        specValueList: values.map((value, valueIndex) => ({
          specValueId: (row.valueItems || []).find((item) => item.value.trim() === value)?.id || row.valueMap?.[value] || `sv_${rowIndex + 1}_${valueIndex + 1}`,
          specValue: value,
          image: null
        }))
      }
    })
    .filter((spec) => spec.title && spec.specValueList.length)
}

function cartesian(specs, index = 0, current = []) {
  if (index >= specs.length) return [current]
  return specs[index].specValueList.flatMap((value) => cartesian(specs, index + 1, [...current, { spec: specs[index], value }]))
}

function generateSkus() {
  const specList = buildSpecList()
  if (!specList.length) {
    if (hasSpecDraft()) {
      Message.warning('请填写完整的规格名和规格值')
      return
    }
    collapseToDefaultSku()
    Message.info('未配置规格，已保留默认 SKU')
    return
  }

  const previousByKey = new Map(form.skus.map((sku) => [skuComboKey(sku), sku]))
  const previousByLabel = new Map(form.skus.map((sku) => [skuLabel(sku), sku]))
  form.skus = cartesian(specList).map((combo) => {
    const specInfo = combo.map(({ spec, value }) => ({
      specId: spec.specId,
      specValueId: value.specValueId,
      specValue: value.specValue
    }))
    const label = specInfo.map((item) => item.specValue).join(' / ')
    const comboKey = specInfo.map((item) => `${item.specId}:${item.specValueId}`).join('|')
    const old = previousByKey.get(comboKey) || previousByLabel.get(label) || previousByLabel.get('默认规格') || {}
    return {
      key: key(),
      sku_image: old.sku_image || form.primary_image,
      sku_image_files: old.sku_image_files || filesFromUrls((old.sku_image || form.primary_image) ? [old.sku_image || form.primary_image] : [], 'sku'),
      spec_info: specInfo,
      sale_price_yuan: old.sale_price_yuan || 0,
      line_price_yuan: old.line_price_yuan || 0,
      stock_quantity: old.stock_quantity || 0
    }
  })
  Message.success(`已生成 ${form.skus.length} 个 SKU`)
}

function skuListMatchesSpecs(specList) {
  if (!specList.length) return true
  const expectedCount = specList.reduce((count, spec) => count * spec.specValueList.length, 1)
  if (form.skus.length !== expectedCount) return false

  const specsById = new Map(specList.map((spec) => [
    spec.specId,
    new Map(spec.specValueList.map((value) => [value.specValueId, value.specValue]))
  ]))
  const seen = new Set()

  return form.skus.every((sku) => {
    const specInfo = sku.spec_info || []
    if (specInfo.length !== specList.length) return false

    const combo = []
    for (const item of specInfo) {
      const specId = item.specId || item.spec_id
      const valueId = item.specValueId || item.spec_value_id
      const valueLabel = item.specValue || item.spec_value
      const values = specsById.get(specId)
      if (!values?.has(valueId)) return false
      if (valueLabel && values.get(valueId) !== valueLabel) return false
      combo.push(`${specId}:${valueId}`)
    }

    combo.sort()
    const comboKey = combo.join('|')
    if (seen.has(comboKey)) return false
    seen.add(comboKey)
    return true
  })
}

function buildPayload() {
  const specList = buildSpecList()
  if (specList.length && !skuListMatchesSpecs(specList)) {
    generateSkus()
  } else if (!specList.length) {
    collapseToDefaultSku()
  }
  const imageUrls = Array.from(new Set([...imageUrlsFromFiles(carouselImageFiles.value), ...splitLines(form.images_text)]))
  const descImageUrls = Array.from(new Set([...imageUrlsFromFiles(descImageFiles.value), ...splitLines(form.desc_images_text)]))
  const skus = (form.skus.length ? form.skus : [makeDefaultSku()]).map((sku) => ({
    sku_image: sku.sku_image || null,
    spec_info: specList.length ? sku.spec_info : [],
    sale_price: yuanToCents(sku.sale_price_yuan),
    line_price: yuanToCents(sku.line_price_yuan),
    stock_quantity: Number(sku.stock_quantity) || 0
  }))

  return {
    store_id: form.store_id || undefined,
    saas_id: form.saas_id || undefined,
    title: form.title.trim(),
    primary_image: form.primary_image.trim(),
    images: imageUrls,
    desc_images: descImageUrls,
    spec_list: specList,
    spu_tag_list: splitLines(form.tags_text).map((title, index) => ({ id: `tag_${index + 1}`, title })),
    category_id: form.category_id || null,
    status: form.status,
    skus
  }
}

async function loadData() {
  loading.value = true
  errorText.value = ''
  try {
    const [goodsData, categoryData] = await Promise.all([
      fetchGoods(cleanParams()),
      fetchCategories().catch(() => [])
    ])
    list.value = goodsData?.list || []
    total.value = goodsData?.total || 0
    categories.value = Array.isArray(categoryData) ? categoryData : []
  } catch (error) {
    errorText.value = error.message || '商品加载失败'
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
  Object.assign(filters, { page: 1, keyword: '', category_id: '', status: '' })
  loadData()
}

function openCreate() {
  resetForm()
  visible.value = true
}

async function openEdit(record) {
  loading.value = true
  try {
    const detail = await fetchGoodsDetail(record.spu_id)
    resetForm(detail)
    visible.value = true
  } catch (error) {
    Message.error(error.message || '商品详情加载失败')
  } finally {
    loading.value = false
  }
}

async function saveProduct() {
  const validation = await formRef.value?.validate()
  if (validation) return
  const specList = buildSpecList()
  if (!specList.length && hasSpecDraft()) {
    Message.warning('请先补全规格名和规格值，或删除未完成的规格行')
    return
  }
  if (!specList.length) {
    collapseToDefaultSku()
  }
  if (!form.skus.length) {
    Message.error('请至少配置一个 SKU')
    return
  }

  saving.value = true
  try {
    const payload = buildPayload()
    if (editing.value) await updateGoods(editing.value.spu_id, payload)
    else await createGoods(payload)
    Message.success(editing.value ? '商品已更新' : '商品已创建')
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
    title: '确认删除商品',
    content: `商品「${record.title}」删除后不可恢复，关联 SKU 也会一并删除。`,
    okText: '删除',
    okButtonProps: { status: 'danger' },
    async onOk() {
      try {
        await deleteGoods(record.spu_id)
        Message.success('商品已删除')
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
  <div class="page-stack product-page">
    <div class="product-hero">
      <div>
        <div class="hero-kicker"><IconTags /> 国膳甄选商品</div>
        <h1>商品管理</h1>
        <p>管理前台售卖商品的基础信息、图片素材、分类、上下架状态、价格和 SKU 库存。</p>
      </div>
      <a-space>
        <a-button :loading="loading" @click="loadData">
          <template #icon><IconRefresh /></template>
          刷新
        </a-button>
        <a-button type="primary" @click="openCreate">
          <template #icon><IconPlus /></template>
          新建商品
        </a-button>
      </a-space>
    </div>

    <div class="product-stats">
      <div class="product-stat">
        <span>商品总数</span>
        <strong>{{ total }}</strong>
      </div>
      <div class="product-stat">
        <span>当前页上架</span>
        <strong>{{ activeCount }}</strong>
      </div>
      <div class="product-stat">
        <span>当前页售罄</span>
        <strong>{{ soldOutCount }}</strong>
      </div>
      <div class="product-stat">
        <span>当前页库存 / 销量</span>
        <strong>{{ pageStock }} / {{ pageSales }}</strong>
      </div>
    </div>

    <PageCard>
      <template #title>商品列表</template>
      <template #extra><a-tag color="arcoblue">共 {{ total }} 条</a-tag></template>

      <div v-if="errorText" class="table-error">
        <span>{{ errorText }}</span>
        <a-button size="small" @click="loadData">重新加载</a-button>
      </div>

      <div class="toolbar product-toolbar">
        <a-input-search
          v-model="filters.keyword"
          allow-clear
          placeholder="搜索商品标题"
          class="product-search"
          @search="search"
        >
          <template #prefix><IconSearch /></template>
        </a-input-search>
        <a-select v-model="filters.category_id" placeholder="分类" allow-clear class="product-filter">
          <a-option v-for="item in categories" :key="item.id" :value="String(item.id)">{{ item.name }}</a-option>
        </a-select>
        <a-select v-model="filters.status" placeholder="状态" allow-clear class="product-filter">
          <a-option :value="1">上架</a-option>
          <a-option :value="0">下架</a-option>
        </a-select>
        <a-button type="primary" @click="search">查询</a-button>
        <a-button @click="resetFilters">重置</a-button>
      </div>

      <a-table
        :columns="columns"
        :data="list"
        :loading="loading"
        row-key="spu_id"
        :scroll="{ x: 1250 }"
        :pagination="{ current: filters.page, pageSize: filters.page_size, total, showTotal: true, showPageSize: true }"
        @page-change="(page) => { filters.page = page; loadData() }"
        @page-size-change="(pageSize) => { filters.page = 1; filters.page_size = pageSize; loadData() }"
      >
        <template #empty>
          <a-empty class="table-empty" description="暂无商品数据，可点击右上角新建商品" />
        </template>

        <template #product="{ record }">
          <div class="product-name-cell">
            <a-avatar shape="square" :image-url="record.primary_image" class="product-thumb">
              {{ record.title?.slice(0, 1) || '商' }}
            </a-avatar>
            <div>
              <span class="strong-text">{{ record.title }}</span>
              <span class="muted-text">SPU {{ record.spu_id }} · {{ record.store_id || '默认门店' }}</span>
            </div>
          </div>
        </template>

        <template #category="{ record }">{{ categoryName(record) }}</template>

        <template #price="{ record }">
          <div class="price-stack">
            <span class="strong-text">{{ money(record.min_sale_price) }}</span>
            <span v-if="Number(record.max_line_price || 0) > 0" class="muted-text">划线 {{ money(record.max_line_price) }}</span>
          </div>
        </template>

        <template #stock="{ record }">
          <a-tag :color="stockColor(record.spu_stock_quantity)">{{ record.spu_stock_quantity }}</a-tag>
        </template>

        <template #sold="{ record }">{{ record.sold_num || 0 }}</template>

        <template #skuCount="{ record }">
          <a-tag>{{ record.skus?.length || 0 }} 个</a-tag>
        </template>

        <template #status="{ record }">
          <a-tag :color="record.status ? 'green' : 'gray'">{{ record.status ? '上架' : '下架' }}</a-tag>
        </template>

        <template #actions="{ record }">
          <a-space>
            <a-tooltip content="编辑商品">
              <a-button size="small" @click="openEdit(record)">
                <template #icon><IconEdit /></template>
              </a-button>
            </a-tooltip>
            <a-tooltip content="删除商品">
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
      :width="920"
      :title="editing ? '编辑商品' : '新建商品'"
      unmount-on-close
    >
      <a-form ref="formRef" :model="form" :rules="rules" layout="vertical">
        <a-tabs default-active-key="base">
          <a-tab-pane key="base" title="基础信息">
            <a-row :gutter="16">
              <a-col :span="16">
                <a-form-item field="title" label="商品标题">
                  <a-input v-model="form.title" placeholder="请输入商品标题" allow-clear />
                </a-form-item>
              </a-col>
              <a-col :span="8">
                <a-form-item field="status" label="销售状态">
                  <a-switch v-model="form.status" checked-text="上架" unchecked-text="下架" />
                </a-form-item>
              </a-col>
            </a-row>

            <a-row :gutter="16">
              <a-col :span="12">
                <a-form-item field="category_id" label="商品分类">
                  <a-select v-model="form.category_id" allow-clear placeholder="选择分类">
                    <a-option v-for="item in categories" :key="item.id" :value="String(item.id)">{{ item.name }}</a-option>
                  </a-select>
                </a-form-item>
              </a-col>
              <a-col :span="6">
                <a-form-item field="store_id" label="门店 ID">
                  <a-input v-model="form.store_id" placeholder="默认可留空" allow-clear />
                </a-form-item>
              </a-col>
              <a-col :span="6">
                <a-form-item field="saas_id" label="SaaS ID">
                  <a-input v-model="form.saas_id" placeholder="默认可留空" allow-clear />
                </a-form-item>
              </a-col>
            </a-row>

            <a-form-item field="primary_image" label="商品主图">
              <div class="image-field">
                <a-upload
                  v-model:file-list="primaryImageFiles"
                  list-type="picture-card"
                  accept="image/*"
                  :limit="1"
                  :custom-request="uploadImageRequest"
                  :on-before-upload="validateImageFile"
                  image-preview
                  @change="handlePrimaryImageChange"
                  @error="handleImageUploadError"
                >
                  <template #upload-button>
                    <div class="image-upload-button">
                      <IconPlus />
                      <span>上传主图</span>
                    </div>
                  </template>
                </a-upload>
              </div>
            </a-form-item>

            <a-form-item field="tags_text" label="商品标签">
              <a-textarea v-model="form.tags_text" :auto-size="{ minRows: 2 }" placeholder="每行一个标签，例如：甄选 / 热销 / 新品" />
            </a-form-item>
          </a-tab-pane>

          <a-tab-pane key="media" title="图片素材">
            <a-form-item field="images_text" label="轮播图">
              <div class="image-field">
                <a-upload
                  v-model:file-list="carouselImageFiles"
                  list-type="picture-card"
                  accept="image/*"
                  multiple
                  :limit="12"
                  :custom-request="uploadImageRequest"
                  :on-before-upload="validateImageFile"
                  image-preview
                  @change="handleCarouselImageChange"
                  @error="handleImageUploadError"
                >
                  <template #upload-button>
                    <div class="image-upload-button">
                      <IconPlus />
                      <span>上传图片</span>
                    </div>
                  </template>
                </a-upload>
              </div>
            </a-form-item>
            <a-form-item field="desc_images_text" label="详情图">
              <div class="image-field">
                <a-upload
                  v-model:file-list="descImageFiles"
                  list-type="picture-card"
                  accept="image/*"
                  multiple
                  :limit="20"
                  :custom-request="uploadImageRequest"
                  :on-before-upload="validateImageFile"
                  image-preview
                  @change="handleDescImageChange"
                  @error="handleImageUploadError"
                >
                  <template #upload-button>
                    <div class="image-upload-button">
                      <IconPlus />
                      <span>上传详情图</span>
                    </div>
                  </template>
                </a-upload>
              </div>
            </a-form-item>
          </a-tab-pane>

          <a-tab-pane key="sku" title="规格与 SKU">
            <div class="sku-section">
              <div class="sku-section-head">
                <div>
                  <strong>规格组</strong>
                  <span>例如：重量 = 5kg，10kg；口味 = 原味，低糖。生成后可逐个维护 SKU 价格和库存。</span>
                </div>
                <a-space>
                  <a-button @click="addSpecRow">添加规格</a-button>
                  <a-button type="primary" @click="generateSkus">生成 SKU</a-button>
                </a-space>
              </div>

              <div v-if="form.specRows.length" class="spec-grid">
                <div v-for="(row, index) in form.specRows" :key="row.key" class="spec-row">
                  <a-input v-model="row.title" placeholder="规格名，如重量" />
                  <div class="spec-values-editor">
                    <div v-for="(item, valueIndex) in row.valueItems" :key="item.key" class="spec-value-item">
                      <a-input
                        v-model="item.value"
                        placeholder="规格值"
                        allow-clear
                        @change="() => normalizeSpecValueItems(row)"
                        @press-enter="handleSpecValueEnter(row, valueIndex)"
                      />
                      <a-button
                        v-if="row.valueItems.length > 1"
                        class="spec-value-remove"
                        size="mini"
                        status="danger"
                        @click="removeSpecValue(row, valueIndex)"
                      >
                        <template #icon><IconDelete /></template>
                      </a-button>
                    </div>
                    <a-button class="spec-value-add" size="small" @click="addSpecValue(row)">
                      <template #icon><IconPlus /></template>
                      添加规格值
                    </a-button>
                  </div>
                  <a-button status="danger" @click="removeSpecRow(index)">删除</a-button>
                </div>
              </div>
              <a-alert v-else type="info" show-icon>
                未配置规格时，系统会保存为一个默认 SKU。
              </a-alert>

              <div class="sku-detail-panel">
                <div class="sku-detail-head">
                  <strong>SKU 明细</strong>
                  <span>{{ form.skus.length }} 个 SKU</span>
                </div>

                <div class="sku-editor-table-wrap">
                  <table class="sku-editor-table">
                    <colgroup>
                      <col style="width: 200px" />
                      <col style="width: 150px" />
                      <col style="width: 160px" />
                      <col style="width: 160px" />
                      <col style="width: 140px" />
                    </colgroup>
                    <thead>
                      <tr>
                        <th>规格组合</th>
                        <th>SKU 图</th>
                        <th>售价（元）</th>
                        <th>划线价（元）</th>
                        <th>库存</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr v-for="sku in form.skus" :key="sku.key">
                        <td>
                          <span class="sku-spec-label">{{ skuLabel(sku) }}</span>
                        </td>
                        <td>
                          <a-upload
                            v-model:file-list="sku.sku_image_files"
                            class="sku-image-upload"
                            list-type="picture-card"
                            accept="image/*"
                            :limit="1"
                            :custom-request="uploadImageRequest"
                            :on-before-upload="validateImageFile"
                            image-preview
                            @change="(fileList) => syncSkuImage(sku, fileList)"
                            @error="handleImageUploadError"
                          >
                            <template #upload-button>
                              <div class="sku-image-upload-button">
                                <IconPlus />
                                <span>SKU 图</span>
                              </div>
                            </template>
                          </a-upload>
                        </td>
                        <td>
                          <a-input-number v-model="sku.sale_price_yuan" :min="0" :precision="2" style="width: 100%" />
                        </td>
                        <td>
                          <a-input-number v-model="sku.line_price_yuan" :min="0" :precision="2" style="width: 100%" />
                        </td>
                        <td>
                          <a-input-number v-model="sku.stock_quantity" :min="0" style="width: 100%" />
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </a-tab-pane>
        </a-tabs>
      </a-form>

      <template #footer>
        <a-space>
          <a-button @click="visible = false">取消</a-button>
          <a-button type="primary" :loading="saving" @click="saveProduct">保存商品</a-button>
        </a-space>
      </template>
    </a-drawer>
  </div>
</template>
