<script setup>
import { ref, watch } from 'vue'

const props = defineProps({
  open: { type: Boolean, default: false },
  openingAmount: { type: Number, default: 0 },
  operator: { type: String, default: 'Operador' },
  busy: { type: Boolean, default: false },
})

const emit = defineEmits(['open', 'cancel'])

const draft = ref(0)
const operatorName = ref(props.operator)

watch(
  () => props.open,
  (open) => {
    if (open) {
      draft.value = props.openingAmount
      operatorName.value = props.operator
    }
  },
  { immediate: true },
)
</script>

<template>
  <div v-if="open" class="dav-modal-backdrop">
    <div class="dav-window" role="dialog" aria-modal="true" aria-label="Abrir caixa">
      <div class="dav-window-header">
        <span>Abrir Caixa</span>
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
        <p class="eyebrow">Controle diário</p>
        <p class="field-hint">Informe o fundo inicial e o operador para abrir o turno.</p>
        <div class="field-grid compact">
          <label
            >Fundo inicial<input v-model.number="draft" type="number" min="0" step="0.01" required
          /></label>
          <label
            >Operador<input
              v-model="operatorName"
              type="text"
              maxlength="60"
              placeholder="Operador"
          /></label>
        </div>
      </div>
      <div class="dav-window-footer">
        <div class="dav-actions">
          <button class="secondary" type="button" @click="emit('cancel')">Cancelar</button>
          <button class="primary" type="button" :disabled="busy" @click="emit('open')">
            Abrir
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
