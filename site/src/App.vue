<script setup lang="ts">
import HostingInfo from './components/HostingInfo.vue'
import BookRanking from './components/BookRanking.vue'
import { useI18n } from 'vue-i18n'
const { t, locale } = useI18n({
  useScope: 'global',
  messages: {
    en: {
      app: {
        title: 'PAUL SARDA BOOK CLUB',
        book_reviews_title: 'BOOK REVIEWS',
        hosting_info_title: 'ALSO THIS WEBSITE IS HOSTED ON A 2DS?',
        language: 'Select language',
      },
    },
    kr: {
      app: {
        title: '사르다 폴 북 클럽',
        book_reviews_title: '책 리뷰',
        hosting_info_title: '이 웹사이트는 2DS에서 호스팅되었습니다?',
        language: '언어 선택',
      },
    },
  },
})

</script>

<template>
  <header>
    <div class="wrapper">
      <h1>{{ t('app.title') }}</h1>
    </div>
  </header>
  <main>
    <div class="language">
      <p>{{ t('app.language') }}</p>
      <select v-model="locale">
        <option value="en">🇦🇺 Australian</option>
        <option value="kr">🇰🇷 한국어</option>
      </select>
    </div>

    <div class="section">
      <h2>{{ t('app.book_reviews_title') }}</h2>
      <BookRanking />
    </div>
    <hr />
    <div class="section">
      <h2>{{ t('app.hosting_info_title') }}</h2>
      <HostingInfo />
    </div>
  </main>
</template>

<style scoped>
.language {
  padding-bottom: 1em;
}

header {
  line-height: 1.5;
}

.section {
  padding-bottom: 2em;
}
</style>
