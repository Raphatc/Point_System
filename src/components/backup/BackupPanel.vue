<script setup>
import { onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useBackupStore } from '../../stores/backup'

const backupStore = useBackupStore()
const { backups, busy, error, message, loading } = storeToRefs(backupStore)

async function handleCreate() {
  await backupStore.create()
}

async function handleRestore(fileName) {
  const restored = await backupStore.restore(fileName)
  if (!restored) return
}

onMounted(() => {
  backupStore.load()
})
</script>

<template>
  <section class="narrow-content">
    <div v-if="loading" class="loading-state" role="status">Carregando backups...</div>
    <div v-if="error" class="notice error" role="alert">{{ error }}</div>
    <div v-if="message" class="notice success">{{ message }}</div>

    <article class="card">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Dados</p>
          <h3>Backup do banco</h3>
        </div>
      </div>
      <p class="field-hint">
        Crie um backup do arquivo SQLite do aplicativo ou restaure um backup anterior.
      </p>
      <button class="primary" type="button" :disabled="busy" @click="handleCreate">
        Criar backup
      </button>

      <ul v-if="backups.length" class="backup-list">
        <li v-for="fileName in backups" :key="fileName">
          <span>{{ fileName }}</span>
          <button class="secondary" type="button" :disabled="busy" @click="handleRestore(fileName)">
            Restaurar
          </button>
        </li>
      </ul>
      <p v-else-if="!loading" class="field-hint">Nenhum backup encontrado.</p>
    </article>
  </section>
</template>
