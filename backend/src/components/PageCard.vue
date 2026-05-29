<template>
  <a-card class="page-card" :bordered="false">
    <template v-if="$slots.title" #title><slot name="title" /></template>
    <template v-if="$slots.extra" #extra><slot name="extra" /></template>
    <slot />
  </a-card>
</template>
