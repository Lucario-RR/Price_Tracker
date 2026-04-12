<script setup>
import { dateZhCN, zhCN } from "naive-ui";
import ReleaseApp from "./ReleaseApp.vue";

/** 全局略增大圆角，组件会继承 common 中的半径变量 */
const themeOverrides = {
  common: {
    borderRadius: "12px",
    borderRadiusSmall: "8px"
  },
  Card: {
    borderRadius: "14px"
  },
  Modal: {
    borderRadius: "16px"
  },
  Dialog: {
    borderRadius: "16px"
  },
  Drawer: {
    borderRadius: "14px"
  },
  Popover: {
    borderRadius: "12px"
  },
  Dropdown: {
    borderRadius: "12px"
  },
  Tooltip: {
    borderRadius: "8px"
  },
  Alert: {
    borderRadius: "12px"
  },
  Tag: {
    borderRadius: "8px"
  },
  Button: {
    borderRadius: "10px"
  },
  Input: {
    borderRadius: "10px"
  },
  Tabs: {
    tabBorderRadius: "10px"
  }
};
</script>

<template>
  <n-config-provider :locale="zhCN" :date-locale="dateZhCN" :theme-overrides="themeOverrides">
    <ReleaseApp />
  </n-config-provider>
</template>
