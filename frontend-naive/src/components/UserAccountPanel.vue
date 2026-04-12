<script setup>
import { ref } from "vue";

import { SUPPORTED_PROFILE_CURRENCIES } from "../data/currencies";

defineProps({
  profile: {
    type: Object,
    default: null
  },
  loading: {
    type: Boolean,
    default: false
  },
  busy: {
    type: Boolean,
    default: false
  },
  profileForm: {
    type: Object,
    required: true
  },
  emailDraft: {
    type: Object,
    required: true
  },
  phoneDraft: {
    type: Object,
    required: true
  },
  passwordForm: {
    type: Object,
    required: true
  },
  avatarState: {
    type: Object,
    required: true
  },
  cookiePreferences: {
    type: Object,
    required: true
  },
  emails: {
    type: Array,
    required: true
  },
  phones: {
    type: Array,
    required: true
  }
});

defineEmits([
  "refresh",
  "save-profile",
  "pick-avatar",
  "upload-avatar",
  "remove-avatar",
  "add-email",
  "delete-email",
  "verify-email",
  "make-primary-email",
  "add-phone",
  "delete-phone",
  "verify-phone",
  "make-primary-phone",
  "change-password",
  "save-cookie-preferences"
]);

const avatarFileRef = ref(null);

const currencyOptions = SUPPORTED_PROFILE_CURRENCIES.map((c) => ({
  label: c.label,
  value: c.code
}));
</script>

<template>
  <n-space vertical size="large" style="width: 100%">
    <n-card title="个人资料" :segmented="{ content: true }">
      <template #header-extra>
        <n-space align="center">
          <n-tag type="success" size="small">{{ profile?.status || "active" }}</n-tag>
          <n-button size="small" quaternary :disabled="busy || loading" @click="$emit('refresh')">
            重新加载
          </n-button>
        </n-space>
      </template>

      <n-spin :show="loading">
        <n-form label-placement="top" label-width="auto">
          <n-grid :cols="2" :x-gap="16" responsive="screen">
            <n-gi span="2 m:1">
              <n-form-item label="显示名称">
                <n-input v-model:value="profileForm.displayName" :disabled="busy" />
              </n-form-item>
            </n-gi>
            <n-gi span="2 m:1">
              <n-form-item label="默认货币">
                <n-select v-model:value="profileForm.defaultCurrency" :options="currencyOptions" :disabled="busy" />
              </n-form-item>
            </n-gi>
            <n-gi span="2 m:1">
              <n-form-item label="区域语言 (locale)">
                <n-input v-model:value="profileForm.locale" :disabled="busy" />
              </n-form-item>
            </n-gi>
            <n-gi span="2 m:1">
              <n-form-item label="时区">
                <n-input v-model:value="profileForm.timezoneName" :disabled="busy" />
              </n-form-item>
            </n-gi>
            <n-gi span="2">
              <n-form-item label="个人简介">
                <n-input v-model:value="profileForm.profileBio" type="textarea" :rows="4" :disabled="busy" />
              </n-form-item>
            </n-gi>
          </n-grid>
        </n-form>

        <n-space style="margin: 12px 0">
          <n-tag type="info" size="small">
            {{ profile?.security?.passwordSet ? "已设置密码" : "未设置密码" }}
          </n-tag>
          <n-tag type="warning" size="small">
            {{ profile?.security?.mfaEnabled ? "已开 MFA" : "未开 MFA" }}
          </n-tag>
          <n-tag type="success" size="small">{{ profile?.security?.passkeyCount || 0 }} 通行密钥</n-tag>
        </n-space>

        <n-button type="primary" :disabled="busy" @click="$emit('save-profile')">
          {{ busy ? "保存中…" : "保存资料" }}
        </n-button>
      </n-spin>
    </n-card>

    <n-card title="头像" :segmented="{ content: true }">
      <template #header-extra>
        <n-tag size="small">{{ profile?.avatarFilename || profile?.avatarFileId || "无头像" }}</n-tag>
      </template>
      <n-text depth="3" style="display: block">上传走文件意图流程，完成后绑定为头像。</n-text>
      <n-form-item label="已选文件">
        <n-input :value="avatarState.selectedFileName || ''" disabled placeholder="未选择" />
      </n-form-item>
      <n-space>
        <input
          ref="avatarFileRef"
          type="file"
          accept="image/*"
          style="display: none"
          @change="$emit('pick-avatar', $event)"
        />
        <n-button @click="avatarFileRef?.click()">选择图片</n-button>
        <n-button type="primary" :disabled="busy || !avatarState.selectedFile" @click="$emit('upload-avatar')">
          {{ busy ? "上传中…" : "上传头像" }}
        </n-button>
        <n-button quaternary :disabled="busy || !profile?.avatarFileId" @click="$emit('remove-avatar')">
          移除头像
        </n-button>
      </n-space>
    </n-card>

    <n-card title="邮箱" :segmented="{ content: true }">
      <template #header-extra>
        <n-tag size="small">{{ emails.length }} 条</n-tag>
      </template>
      <n-grid :cols="2" :x-gap="16" responsive="screen">
        <n-gi span="2 m:1">
          <n-form-item label="新邮箱">
            <n-input v-model:value="emailDraft.email" :disabled="busy" />
          </n-form-item>
        </n-gi>
        <n-gi span="2 m:1">
          <n-form-item label="角色">
            <n-select
              v-model:value="emailDraft.emailRole"
              :disabled="busy"
              :options="[
                { label: '次要', value: 'SECONDARY' },
                { label: '主要', value: 'PRIMARY' }
              ]"
            />
          </n-form-item>
        </n-gi>
      </n-grid>
      <n-checkbox v-model:checked="emailDraft.isLoginEnabled" :disabled="busy">允许用此邮箱登录</n-checkbox>
      <n-button style="margin-top: 12px" type="primary" :disabled="busy" @click="$emit('add-email')">
        {{ busy ? "保存中…" : "添加邮箱" }}
      </n-button>

      <n-divider />

      <n-space vertical>
        <n-card v-for="email in emails" :key="email.id" size="small" embedded>
          <n-space vertical>
            <n-space justify="space-between" align="center">
              <n-text strong>{{ email.email }}</n-text>
              <n-tag size="small">{{ email.emailRole }}</n-tag>
            </n-space>
            <n-space>
              <n-tag :type="email.isPrimaryForAccount ? 'success' : 'default'" size="small">
                {{ email.isPrimaryForAccount ? "主邮箱" : "次要" }}
              </n-tag>
              <n-tag :type="email.verifiedAt ? 'success' : 'warning'" size="small">
                {{ email.verifiedAt ? "已验证" : "未验证" }}
              </n-tag>
            </n-space>
            <n-space>
              <n-button
                size="small"
                quaternary
                :disabled="busy || email.isPrimaryForAccount"
                @click="$emit('make-primary-email', email.id)"
              >
                设为主邮箱
              </n-button>
              <n-button size="small" quaternary :disabled="busy || email.verifiedAt" @click="$emit('verify-email', email.id)">
                验证
              </n-button>
              <n-button
                size="small"
                type="error"
                tertiary
                :disabled="busy || email.isPrimaryForAccount"
                @click="$emit('delete-email', email.id)"
              >
                删除
              </n-button>
            </n-space>
            <n-text v-if="email.isPrimaryForAccount" depth="3" style="font-size: 12px">
              删除前请先将其他邮箱设为主邮箱。
            </n-text>
          </n-space>
        </n-card>
      </n-space>
    </n-card>

    <n-card title="手机号" :segmented="{ content: true }">
      <template #header-extra>
        <n-tag size="small">{{ phones.length }} 条</n-tag>
      </template>
      <n-form-item label="新号码">
        <n-input v-model:value="phoneDraft.phoneNumber" :disabled="busy" />
      </n-form-item>
      <n-button type="primary" :disabled="busy" @click="$emit('add-phone')">
        {{ busy ? "保存中…" : "添加号码" }}
      </n-button>
      <n-divider />
      <n-space vertical>
        <n-card v-for="phone in phones" :key="phone.id" size="small" embedded>
          <n-space vertical>
            <n-text strong>{{ phone.phoneNumber }}</n-text>
            <n-tag :type="phone.verifiedAt ? 'success' : 'warning'" size="small">
              {{ phone.verifiedAt ? "已验证" : "未验证" }}
            </n-tag>
            <n-space>
              <n-button
                size="small"
                quaternary
                :disabled="busy || phone.isPrimaryForAccount"
                @click="$emit('make-primary-phone', phone.id)"
              >
                设为主号码
              </n-button>
              <n-button size="small" quaternary :disabled="busy || phone.verifiedAt" @click="$emit('verify-phone', phone.id)">
                验证
              </n-button>
              <n-button size="small" type="error" tertiary :disabled="busy" @click="$emit('delete-phone', phone.id)">
                删除
              </n-button>
            </n-space>
          </n-space>
        </n-card>
      </n-space>
    </n-card>

    <n-card title="密码与 Cookie" :segmented="{ content: true }">
      <n-grid :cols="1" :y-gap="12">
        <n-gi>
          <n-form-item label="当前密码">
            <n-input
              v-model:value="passwordForm.currentPassword"
              :disabled="busy"
              type="password"
              show-password-on="mousedown"
              placeholder="当前密码"
            />
          </n-form-item>
        </n-gi>
        <n-gi>
          <n-form-item label="新密码">
            <n-input
              v-model:value="passwordForm.newPassword"
              :disabled="busy"
              type="password"
              show-password-on="mousedown"
              placeholder="新密码"
            />
          </n-form-item>
        </n-gi>
        <n-gi>
          <n-form-item label="确认新密码">
            <n-input
              v-model:value="passwordForm.confirmPassword"
              :disabled="busy"
              type="password"
              show-password-on="mousedown"
              placeholder="再次输入"
            />
          </n-form-item>
        </n-gi>
      </n-grid>
      <n-button type="primary" :disabled="busy" @click="$emit('change-password')">
        {{ busy ? "保存中…" : "修改密码" }}
      </n-button>

      <n-divider title-placement="left">Cookie 偏好</n-divider>
      <n-space vertical>
        <n-checkbox v-model:checked="cookiePreferences.analytics" :disabled="busy">分析类 Cookie</n-checkbox>
        <n-checkbox v-model:checked="cookiePreferences.marketing" :disabled="busy">营销类 Cookie</n-checkbox>
        <n-checkbox v-model:checked="cookiePreferences.preferences" :disabled="busy">偏好类 Cookie</n-checkbox>
      </n-space>
      <n-button style="margin-top: 12px" :disabled="busy" @click="$emit('save-cookie-preferences')">
        {{ busy ? "保存中…" : "保存 Cookie 设置" }}
      </n-button>
    </n-card>
  </n-space>
</template>
