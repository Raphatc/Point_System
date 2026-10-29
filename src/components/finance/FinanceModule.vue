<script setup>
import DashboardPanel from './DashboardPanel.vue'
import ReceivablesPanel from './ReceivablesPanel.vue'

const emit = defineEmits(['navigate'])

function handleDashboardNavigation(target) {
  if (target === 'receivables') {
    document.getElementById('finance-receivables')?.scrollIntoView({ behavior: 'smooth' })
    return
  }
  emit('navigate', target)
}
</script>

<template>
  <section>
    <DashboardPanel @navigate="handleDashboardNavigation" />
    <div id="finance-receivables" class="finance-receivables">
      <ReceivablesPanel />
    </div>
  </section>
</template>
