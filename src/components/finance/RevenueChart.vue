<script setup>
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import Chart from 'chart.js/auto'

const props = defineProps({ data: { type: Array, default: () => [] } })
const canvas = ref(null)
let chart

function render() {
  chart?.destroy()
  if (!canvas.value) return
  chart = new Chart(canvas.value, {
    type: 'line',
    data: {
      labels: props.data.map((item) => item.month),
      datasets: [
        {
          label: 'Faturamento',
          data: props.data.map((item) => Number(item.total) / 100),
          borderColor: '#147d68',
          backgroundColor: 'rgba(20,125,104,0.12)',
          tension: 0.32,
          fill: true,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: { legend: { display: false } },
    },
  })
}

onMounted(render)
watch(() => props.data, render, { deep: true })
onBeforeUnmount(() => chart?.destroy())
</script>

<template>
  <div class="chart-frame"><canvas ref="canvas"></canvas></div>
</template>
