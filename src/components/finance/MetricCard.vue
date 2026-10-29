<script setup>
defineProps({
  label: { type: String, required: true },
  value: { type: [String, Number], required: true },
  tone: { type: String, default: 'blue' },
})
</script>

<template>
  <article class="card stat-card" :class="`accent-${tone}`">
    <label>{{ label }}</label>
    <strong>{{ value }}</strong>
  </article>
</template>
