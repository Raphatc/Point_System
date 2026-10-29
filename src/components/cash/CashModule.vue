<script setup>
import CashPanel from './CashPanel.vue'
import SalesPanel from '../sales/SalesPanel.vue'

const emit = defineEmits(['navigate'])
</script>

<template>
  <section class="cash-module-grid">
    <CashPanel />
    <SalesPanel @navigate="emit('navigate', $event)" />
  </section>
</template>
