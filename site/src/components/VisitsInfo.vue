<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
const { t } = useI18n({
  messages: {
    en: {
      visits_line: 'Book club aficionados: {0}',
      loading: 'Loading visits...',
    },
    kr: {
      visits_line: '북 클럽 애호가: {0}',
      loading: '방문자 수를 불러오는 중...',
    },
  },
})

const loading = ref(true)
const visits = ref(0)

onMounted(async () => {
  const response = await fetch('/api/visits')
  const body = await response.json()
  visits.value = body.data.visits
  loading.value = false
})
</script>

<template>
  <div>
    <div v-if="loading">
      <p>{{ t('loading') }}</p>
    </div>
    <div v-else>
      <p :key="visits">{{ t('visits_line', [visits]) }}</p>
    </div>
  </div>
</template>

<style scoped>
h3 {
  margin-top: 20px;
}

table {
  border: 1px solid black;
  margin: 0px auto;
}

td {
  border: 1px solid black;
}

img {
  max-width: 90%;
}
</style>
