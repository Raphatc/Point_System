<script setup>
import { onMounted, ref } from 'vue'
import { useClientsStore } from '../../stores/clients'

const clientsStore = useClientsStore()
const clients = clientsStore.clients
const error = clientsStore.error
const message = clientsStore.message
const loading = clientsStore.loading
const form = ref({ name: '' })

async function load() {
  await clientsStore.load()
}

async function create() {
  const created = await clientsStore.createClient({ name: form.value.name })
  if (created) {
    form.value = { name: '' }
    message.value = 'Cliente cadastrado.'
  }
}

onMounted(load)
</script>

<template>
  <section>
    <div v-if="loading" class="loading-state" role="status">Carregando clientes...</div>
    <div v-if="error" class="notice error" role="alert">{{ error }}</div>
    <div v-if="message" class="notice success">{{ message }}</div>
    <article class="card">
      <h3>Novo cliente</h3>
      <form class="seller-form" @submit.prevent="create">
        <label>Nome do cliente<input v-model="form.name" required maxlength="120" /></label>
        <button class="primary" type="submit">Cadastrar cliente</button>
      </form>
    </article>
    <article class="card table-card">
      <div class="section-heading">
        <h3>Clientes</h3>
        <span>{{ clients.length }} cadastrados</span>
      </div>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Nome</th>
              <th>Cadastro</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="client in clients" :key="client.id">
              <td>
                {{ client.name }} <span v-if="client.is_demo" class="demo-badge">Demonstração</span>
              </td>
              <td>{{ new Date(client.created_at).toLocaleDateString('pt-BR') }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <p v-if="!clients.length" class="empty-state">Nenhum cliente cadastrado.</p>
    </article>
  </section>
</template>
