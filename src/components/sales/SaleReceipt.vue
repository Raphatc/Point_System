<script setup>
import { computed } from 'vue'
import { formatCurrency } from '../../lib/format'

const props = defineProps({
  receipt: { type: Object, default: () => ({}) },
})

const installments = computed(() => props.receipt.installments ?? [])
</script>

<template>
  <section v-if="receipt.saleId" class="print-receipt" aria-hidden="true">
    <h1>Comprovante de venda</h1>
    <p>Venda #{{ receipt.saleId }} · {{ receipt.date }}</p>
    <p>Vendedor: {{ receipt.seller }}</p>
    <p v-if="receipt.client">Cliente: {{ receipt.client }}</p>
    <p>Pagamento: {{ receipt.paymentMethod }}</p>
    <hr />
    <div v-for="(item, index) in receipt.items" :key="index" class="receipt-line">
      <span
        >{{ item.quantity }} × {{ item.name }}
        <small v-if="item.gtin">({{ item.gtin }})</small></span
      >
      <span>{{ formatCurrency(item.quantity * item.unitPrice) }}</span>
    </div>
    <hr />
    <strong>Total: {{ formatCurrency(receipt.total) }}</strong>
    <ol v-if="installments.length" class="receipt-installments">
      <li v-for="installment in installments" :key="installment.number">
        Parcela {{ installment.number }}: {{ formatCurrency(installment.amount) }} em
        {{ new Date(`${installment.dueDate}T00:00:00`).toLocaleDateString('pt-BR') }}
      </li>
    </ol>
  </section>
</template>
