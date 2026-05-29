<script setup>
import { computed, onMounted, reactive, ref } from 'vue'
import { Message, Modal } from '@arco-design/web-vue'
import { fetchAdminUsers, fetchPermissionCatalog, updateAdminUserPermissions } from '@/api/admin'

const loading = ref(false)
const saving = ref(false)
const visible = ref(false)
const errorText = ref('')
const users = ref([])
const catalog = ref({ groups: [] })
const editing = ref(null)
const form = reactive({ role: 'operator', is_active: true, permission_codes: [] })

const permissionOptions = computed(() =>
  (catalog.value.groups || []).flatMap((group) => (group.items || []).map((item) => ({ ...item, group: group.name })))
)

function read(record, camelKey, snakeKey, fallback = '') {
  if (!record) return fallback
  return record[camelKey] ?? record[snakeKey] ?? fallback
}

function permissionName(code) {
  return permissionOptions.value.find((item) => item.code === code)?.name || code
}

function permissionSummary(record) {
  const role = read(record, 'role', 'role')
  const codes = read(record, 'permissionCodes', 'permission_codes', []) || []
  if (role === 'admin') return '全部权限'
  if (!codes.length) return '未配置'
  const names = codes.slice(0, 3).map(permissionName)
  return codes.length > 3 ? `${names.join('、')} 等 ${codes.length} 项` : names.join('、')
}

function roleColor(role) {
  return role === 'admin' ? 'red' : 'blue'
}

function isPermissionChecked(code) {
  return form.permission_codes.includes(code)
}

function togglePermission(code, checked) {
  const next = new Set(form.permission_codes)
  if (checked) next.add(code)
  else next.delete(code)
  form.permission_codes = [...next]
}

function isGroupChecked(group) {
  const codes = (group.items || []).map((item) => item.code)
  return Boolean(codes.length) && codes.every((code) => form.permission_codes.includes(code))
}

function toggleGroup(group, checked) {
  const next = new Set(form.permission_codes)
  for (const item of group.items || []) {
    if (checked) next.add(item.code)
    else next.delete(item.code)
  }
  form.permission_codes = [...next]
}

async function loadData() {
  loading.value = true
  errorText.value = ''
  try {
    const [userData, catalogData] = await Promise.all([fetchAdminUsers(), fetchPermissionCatalog()])
    users.value = userData?.list || []
    catalog.value = catalogData || { groups: [] }
  } catch (error) {
    errorText.value = error.message || '权限数据加载失败'
    Message.error(errorText.value)
  } finally {
    loading.value = false
  }
}

function openEdit(record) {
  editing.value = record
  form.role = read(record, 'role', 'role', 'operator')
  form.is_active = Boolean(read(record, 'isActive', 'is_active', true))
  form.permission_codes = [...(read(record, 'permissionCodes', 'permission_codes', []) || [])]
  visible.value = true
}

function savePermissions() {
  if (!editing.value) return
  Modal.confirm({
    title: '确认更新权限',
    content: `将更新管理员「${read(editing.value, 'username', 'username')}」的角色与权限。`,
    async onOk() {
      saving.value = true
      try {
        await updateAdminUserPermissions(read(editing.value, 'id', 'id'), {
          role: form.role,
          is_active: form.is_active,
          permission_codes: form.permission_codes
        })
        Message.success('权限已更新')
        visible.value = false
        await loadData()
      } catch (error) {
        Message.error(error.message || '权限保存失败')
      } finally {
        saving.value = false
      }
    }
  })
}

onMounted(loadData)
</script>

<template>
  <div class="page-stack admin-user-page">
    <a-card :bordered="false" class="page-card">
      <template #title>权限管理</template>
      <template #extra>
        <a-button :loading="loading" @click="loadData">刷新</a-button>
      </template>

      <div v-if="errorText" class="table-error">
        <span>{{ errorText }}</span>
        <a-button size="small" @click="loadData">重新加载</a-button>
      </div>

      <a-spin :loading="loading" style="width: 100%">
        <div v-if="!users.length" class="table-empty">
          <a-empty description="暂无管理员数据" />
        </div>
        <div v-else class="management-table-wrap admin-user-table-wrap">
          <table class="management-table admin-user-table">
            <thead>
              <tr>
                <th>账号</th>
                <th>角色</th>
                <th>状态</th>
                <th>权限概要</th>
                <th>创建时间</th>
                <th>更新时间</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="record in users" :key="read(record, 'id', 'id')">
                <td><span class="strong-text">{{ read(record, 'username', 'username') }}</span></td>
                <td><a-tag :color="roleColor(read(record, 'role', 'role'))">{{ read(record, 'role', 'role') }}</a-tag></td>
                <td>
                  <a-tag :color="read(record, 'isActive', 'is_active') ? 'green' : 'gray'">
                    {{ read(record, 'isActive', 'is_active') ? '启用' : '停用' }}
                  </a-tag>
                </td>
                <td>
                  <span class="permission-summary">{{ permissionSummary(record) }}</span>
                </td>
                <td><span class="muted-text">{{ read(record, 'createdAt', 'created_at') || '-' }}</span></td>
                <td><span class="muted-text">{{ read(record, 'updatedAt', 'updated_at') || '-' }}</span></td>
                <td><a-button size="small" @click="openEdit(record)">编辑权限</a-button></td>
              </tr>
            </tbody>
          </table>
        </div>
      </a-spin>
    </a-card>

    <a-card :bordered="false" class="page-card">
      <template #title>权限目录</template>
      <div class="permission-catalog-grid">
        <section v-for="group in catalog.groups" :key="group.name" class="permission-catalog-group">
          <h3>{{ group.name }}</h3>
          <div class="permission-catalog-items">
            <div v-for="item in group.items" :key="item.code" class="permission-catalog-item">
              <strong>{{ item.name }}</strong>
              <span>{{ item.code }}</span>
              <p>{{ item.description }}</p>
            </div>
          </div>
        </section>
      </div>
    </a-card>

    <a-modal
      v-model:visible="visible"
      title="编辑管理员权限"
      width="920px"
      modal-class="permission-modal"
      :body-style="{ maxHeight: 'calc(100vh - 180px)', overflowY: 'auto', padding: '20px 24px' }"
      :confirm-loading="saving"
      @ok="savePermissions"
    >
      <a-alert type="warning" show-icon>变更当前账号关键权限可能导致菜单或接口访问受限，请确认后保存。</a-alert>
      <a-form :model="form" layout="vertical" style="margin-top: 16px">
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item field="role" label="角色">
              <a-select v-model="form.role">
                <a-option value="admin">admin</a-option>
                <a-option value="operator">operator</a-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item field="is_active" label="账号状态">
              <a-switch v-model="form.is_active" checked-text="启用" unchecked-text="停用" />
            </a-form-item>
          </a-col>
        </a-row>

        <div class="permission-editor">
          <section v-for="group in catalog.groups" :key="group.name" class="permission-editor-group">
            <div class="permission-editor-head">
              <strong>{{ group.name }}</strong>
              <a-checkbox :model-value="isGroupChecked(group)" @change="(checked) => toggleGroup(group, checked)">全选本组</a-checkbox>
            </div>
            <div class="permission-editor-items">
              <a-checkbox
                v-for="item in group.items"
                :key="item.code"
                :model-value="isPermissionChecked(item.code)"
                @change="(checked) => togglePermission(item.code, checked)"
              >
                <span class="strong-text">{{ item.name }}</span>
                <span class="muted-text">{{ item.code }}</span>
              </a-checkbox>
            </div>
          </section>
        </div>
      </a-form>
    </a-modal>
  </div>
</template>
