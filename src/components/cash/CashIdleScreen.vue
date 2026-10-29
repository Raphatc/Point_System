<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue'

const emit = defineEmits(['open'])

const props = defineProps({
  busy: { type: Boolean, default: false },
})

const clock = ref(formatClock())

function formatClock() {
  const now = new Date()
  const hours = String(now.getHours()).padStart(2, '0')
  const minutes = String(now.getMinutes()).padStart(2, '0')
  const seconds = String(now.getSeconds()).padStart(2, '0')
  return `${hours}:${minutes}:${seconds}`
}

let timer = null

onMounted(() => {
  timer = setInterval(() => {
    clock.value = formatClock()
  }, 1000)
})

onBeforeUnmount(() => {
  clearInterval(timer)
})
</script>

<template>
  <div class="cash-idle">
    <p class="cash-idle-slogan">Distribuidora Point</p>
    <h2 class="cash-idle-title">Caixa Livre</h2>
    <div class="cash-idle-clock" role="timer">{{ clock }}</div>
    <div class="cash-idle-actions">
      <button class="primary" type="button" :disabled="props.busy" @click="emit('open')">
        Abrir caixa
      </button>
    </div>
  </div>
</template>
