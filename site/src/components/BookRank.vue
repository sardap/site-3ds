<script setup lang="ts">
import { useRatingsStore } from '@/stores/ratings'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t, d } = useI18n({
  messages: {
    en: {
      author: 'Author: {0}',
      published: 'Published: {0}',
      completed: 'Read on: {0}',
      isbn: 'ISBN: {0}',
      rating: 'Rating: {0}',
      link_copied: 'Link copied',
      copy_link: 'Copy Link',
      user_rating: 'User Rating: {0}',
      user_rating_up: 'Good',
      user_rating_down: 'Bad',
      img_alt: 'Book cover for {0}',
    },
    kr: {
      author: '저자: {0}',
      published: '출판: {0}',
      completed: '완독일: {0}',
      isbn: 'ISBN: {0}',
      rating: '평점: {0}',
      link_copied: '링크 복사됨',
      copy_link: '링크 복사',
      user_rating: '사용자 평점: {0}',
      user_rating_up: '좋아요',
      user_rating_down: '싫어요',
      img_alt: '{0}의 책 표지',
    },
  },
})

export interface BookRankProps {
  id: number
  title: string
  author: string
  year: number
  picture: string
  completed_date: Date
  rating: string
  isbn: string
  review: string
}

const ratings = useRatingsStore()
const props = defineProps<BookRankProps>()

const loading = ref(false)

async function changeRating(positive: boolean) {
  loading.value = true
  // Send request to /api
  try {
    const response = await fetch(`/api/review_ratings`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ id: props.id, positive: positive }),
    })
    const body = await response.json()
    console.log(body)
    if (response.ok) {
      ratings.setRating(props.id, body.data.rating)
    }
  } catch {
    console.error('Failed to update rating')
  }

  loading.value = false
}

function copyLink() {
  navigator.clipboard.writeText(window.location.host + '#book_' + props.id)
  linkCopied.value = true
}

const linkCopied = ref(false)

function onImageLoad() {
  const placeholder = document.getElementById('book_placeholder_' + props.id)
  const image = document.getElementById('book_image_' + props.id)
  if (placeholder && image) {
    placeholder.style.display = 'none'
    image.style.display = ''
  }
}
</script>

<template>
  <div class="rank-row parent" :id="'book_' + id">
    <hr />
    <h3 class="mobile-title title">{{ title }}</h3>
    <div class="number child">
      <img :id="'book_placeholder_' + id" class="placeholder" src="../assets/book_placeholder.webp" />
      <img :id="'book_image_' + id" class="book-cover" style="display: none" :src="picture" @load="onImageLoad" />
    </div>
    <div class="child info">
      <div class="book">
        <h3 class="desktop-title title">{{ title }}</h3>
        <p>{{ t('isbn', [isbn]) }}</p>
        <p>{{ t('author', [author]) }}</p>
        <p>{{ t('published', [year]) }}</p>
        <p>{{ t('completed', [d(completed_date, 'date')]) }}</p>
        <p>{{ t('rating', [rating]) }}</p>
      </div>
      <br />
      <div class="review">
        <p class="body">{{ review }}</p>
      </div>
      <br />
      <div>
        <p :key="ratings.getRating(id)">{{ t('user_rating', [ratings.getRating(id)]) }}</p>
        <button class="user-rating good" @click="changeRating(true)" :disabled="loading">
          {{ t('user_rating_up') }}
        </button>
        <button class="user-rating bad" @click="changeRating(false)" :disabled="loading">
          {{ t('user_rating_down') }}
        </button>
      </div>
      <button @click="copyLink()" :disabled="linkCopied">
        {{ linkCopied ? t('link_copied') : t('copy_link') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.review .body {
  font-size: 1rem;
}

.rank-row .number {
  font-size: 2rem;
  font-weight: 500;
}

.title {
  font-weight: 1000;
}

.rank-row img {
  width: 150px;
  height: auto;
  padding-top: 5px;
  padding-bottom: 5px;
}

.parent {
  padding-top: 1em;
  padding-bottom: 1em;
  text-align: center;
}

@media (min-width: 800px) {
  .child {
    display: inline-block;
    vertical-align: middle;
  }

  .rank-row .info {
    padding-left: 3em;
    width: 400px;
    min-height: 250px;
  }

  .mobile-title {
    display: none;
  }
}

@media (max-width: 800px) {
  .parent {
    max-width: 90%;
    margin: auto;
  }

  .desktop-title {
    display: none;
  }
}

hr {
  max-width: 90%;
  width: 600px;
  color: #ff00ff;
  margin: auto;
}

.user-rating {
  font-size: 1rem;
  padding: 0.5em;
  margin: 0.5em;
  width: 100px;
  border-radius: 5px;
}

.good {
  background-color: #06930633;
}

.bad {
  background-color: #ff000033;
}

.placeholder {
  filter: blur(2px);
}

</style>
