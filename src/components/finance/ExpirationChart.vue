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
    type: 'doughnut',
    data: {
      labels: props.data.map((item) => item.label),
      datasets: [
        {
          data: props.data.map((item) => item.quantity),
          backgroundColor: ['#e89135', '#bd4e3f', '#147d68', '#5a8290'],
          borderWidth: 0,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: { legend: { position: 'bottom' } },
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
