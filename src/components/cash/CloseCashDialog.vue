<script setup>
import { ref, watch } from 'vue'
import { formatCurrency } from '../../lib/format'

const props = defineProps({
  open: { type: Boolean, default: false },
  expectedCash: { type: Number, default: 0 },
})

const emit = defineEmits(['close', 'cancel'])

const value = ref(0)

watch(
  () => props.open,
  (open) => {
    if (open) value.value = props.expectedCash / 100
  },
  { immediate: true },
)
</script>

<template>
  <div v-if="open" class="dav-modal-backdrop">
    <div
      class="dav-window dav-window--small"
      role="dialog"
      aria-modal="true"
      aria-label="Fechamento rápido"
    >
      <div class="dav-window-header">
        <span>Fechamento rápido</span>
        <button
          class="dav-window-close"
          type="button"
          aria-label="Fechar diálogo"
          @click="emit('cancel')"
        >
          ×
        </button>
      </div>
      <div class="dav-window-body">
        <div class="finalize-total">
          <span>Valor esperado</span>
          <strong>{{ formatCurrency(props.expectedCash) }}</strong>
        </div>
        <label
          >Valor pago<input v-model.number="value" type="number" min="0" step="0.01" required
        /></label>
        <p class="field-hint">Informe o valor contado para confirmar o fechamento.</p>
      </div>
      <div class="dav-window-footer centered">
        <div class="dav-actions">
          <button class="secondary" type="button" @click="emit('cancel')">Cancelar</button>
          <button
            class="primary"
            type="button"
            :disabled="false"
            @click="() => emit('close', value)"
          >
            Ok
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
