import { defineStore } from 'pinia'

export interface Ratings {
  [key: string]: number
}

export const useRatingsStore = defineStore('ratings', {
  state: () => ({
    ratings: {} as Ratings,
  }),
  getters: {
    getRating: (state) => (id: string | number) => {
      console.log('getting rating', id)
      return state.ratings[id.toString()] || 0
    },
  },
  actions: {
    setRating(id: string | number, rating: number) {
      console.log('setting rating', id, rating)
      this.ratings[id] = rating
    },
  },
})
