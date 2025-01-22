<script setup lang="ts">
import { useRatingsStore } from '@/stores/ratings'
import BookRank from './BookRank.vue'
import { type BookRankProps } from './BookRank.vue'
import { onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
const { t, locale } = useI18n({
  messages: {
    en: {
      book_1: {
        title: 'Paul Keating: the big-picture leader',
        author: 'Tony Bramston',
        rating: '5 Stars',
        review: 'Book is Good',
      },
      book_2: {
        title: 'Bob Hawke: Demons and Destiny',
        author: 'Tony Bramston',
        rating: '5 Stars',
        review: 'Book is Good',
      },
      book_3: {
        title: 'Triumph and Demise The Broken Promise of a Labor Generation: Updated Edition',
        author: 'Paul Kelly',
        rating: '5 Stars',
        review:
          "A interesting recap of the Rudd-Gillard government shortly after it's demise. I never realised how many of the key figures of the Rudd-Gillard government had not been in government before, Rudd (1998), Gillard (1998), Burke (2004), Shorten (2007), Arbib (2008) never being in government with Swan (1993) only being a backbencher for one term prior to being in government. It being written before the knifing of Abbott makes a fascinating window post the chaos of the previous Labor government prior to the chaos of the Abbott-Turnbull chaos.",
      },
      book_4: {
        title: 'John Curtin: A Life',
        author: 'David Day',
        rating: '5 Stars',
        review: 'A great window into life in Victoria during the start of federation. Also into the anti-conscription campaign, The strange Victoria Socialist Party and it\'s struggle to influence the Labor party. The rough years of the Scullin government. How Labor was able to wrestle control from the UAP and Menzies during WW2. Then political pressures from the war in the Pacific. While giving an insight into Curtin and his struggles with "Rotten moods" and persistent drinking.',
      },
      book_5: {
        title: 'The Big Fella: Jack Lang and the Australian Labor Party, 1891–1949',
        author: 'Bede Nairn',
        rating: '5 Stars',
        review: 'A great read about how a single man can capture an entire political party completely but only in a single state. How that on man was then able to  cause schism inside the largest state. Causing failures for Labor first at the Federal level, then at the state level when he refused to let go.',
      },
      book_6: {
        title: 'Chifley : A Life',
        author: 'David Day',
        rating: '5 Stars',
        review: 'The same way the curtain book gave great insight into life in Victoria during the start of federation. This gives great insight into life in Bathurst until the mid 20th century. It gives less perspective on Chiefly feelings and has more speculation than I would like because he burned all his letters. But is still historically sound for the events surrounding him. Covers greatly his bitter struggle with Jack Lang over the control of the NSW Labor party. His career as a train driver, Care fo council politics and of course his place in the Curtin government then his rise to the top job.',
      },
    },
    kr: {
      book_1: {
        rating: '별5개',
        review: '책이 좋다',
      },
      book_2: {
        rating: '별5개',
        review: '책이 좋다',
      },
      book_3: {
        rating: '별5개',
      },
      book_4: {
        rating: '별5개',
      },
      book_5: {
        rating: '별5개',
      },
      book_6: {
        rating: '별5개',
      },
    },
  },
})

function get_books(): BookRankProps[] {
  const result: BookRankProps[] = [
    {
      id: 1,
      title: t('book_1.title'),
      author: t('book_1.author'),
      year: 2016,
      picture: '/books/HNI_0002.jpg',
      completed_date: new Date('2024-09-16'),
      rating: t('book_1.rating'),
      isbn: '9781925321746',
      review: t('book_1.review'),
    },
    {
      id: 2,
      title: t('book_2.title'),
      author: t('book_2.author'),
      year: 2022,
      picture: '/books/HNI_0003.jpg',
      completed_date: new Date('2024-10-18'),
      rating: t('book_2.rating'),
      isbn: '9780143788096',
      review: t('book_2.review'),
    },
    {
      id: 3,
      title: t('book_3.title'),
      author: t('book_3.author'),
      year: 2014,
      picture: '/books/HNI_0004.jpg',
      completed_date: new Date('2024-11-04'),
      rating: t('book_3.rating'),
      isbn: '9780522862102',
      review: t('book_3.review'),
    },
    {
      id: 4,
      title: t('book_4.title'),
      author: t('book_4.author'),
      year: 1999,
      picture: '/books/HNI_0006.jpg',
      completed_date: new Date('2024-11-19'),
      rating: t('book_4.rating'),
      isbn: '9780732264130',
      review: t('book_4.review'),
    },
    {
      id: 5,
      title: t('book_5.title'),
      author: t('book_5.author'),
      year: 1986,
      picture: '/books/HNI_0005.jpg',
      completed_date: new Date('2024-12-12'),
      rating: t('book_5.rating'),
      isbn: '9781761280740',
      review: t('book_5.review'),
    },
    {
      id: 6,
      title: t('book_6.title'),
      author: t('book_6.author'),
      year: 2001,
      picture: '/books/HNI_0007.jpg',
      completed_date: new Date('2025-01-20'),
      rating: t('book_6.rating'),
      isbn: '9781460706169',
      review: t('book_6.review'),
    },
  ]

  return result
}

const booksSorted = ref<BookRankProps[]>([])

type OrderByTypes = 'read-asc' | 'read-desc' | 'published-asc' | 'published-desc'

const selectedOrderBy = ref<OrderByTypes>('read-desc')

function orderBy(value: OrderByTypes) {
  const books = get_books()
  switch (value) {
    case 'read-asc':
      booksSorted.value = books.sort(
        (a, b) => a.completed_date.getTime() - b.completed_date.getTime(),
      )
      break
    case 'read-desc':
      booksSorted.value = books.sort(
        (a, b) => b.completed_date.getTime() - a.completed_date.getTime(),
      )
      break
    case 'published-asc':
      booksSorted.value = books.sort((a, b) => a.year - b.year)
      break
    case 'published-desc':
      booksSorted.value = books.sort((a, b) => b.year - a.year)
      break
  }
}

watch(locale, () => {
  console.log('locale changed')
  orderBy(selectedOrderBy.value)
})

watch(selectedOrderBy, (value) => {
  orderBy(value)
})

const ratings = useRatingsStore()

onMounted(async () => {
  orderBy(selectedOrderBy.value)
  // Get book ratings form /api
  const response = await fetch(`/api/review_ratings`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
    },
  })
  if (!response.ok) {
    console.error('Failed to get book ratings')
    return
  }
  const body = await response.json()

  // Im a fucking idiot who wants to stop this
  for (let i = 0; i < 256; i++) {
    const value = body.data.review_ratings[i]
    if (value) {
      ratings.setRating(i, value)
    }
  }
})
</script>

<template>
  <div>
    <select v-model="selectedOrderBy">
      <option value="read-desc">Latest read</option>
      <option value="read-asc">Oldest read</option>
      <option value="published-desc">Latest published</option>
      <option value="published-asc">Oldest published</option>
    </select>
    <BookRank v-for="book in booksSorted" :key="book.title" v-bind="book" />
  </div>
</template>

<style scoped></style>
