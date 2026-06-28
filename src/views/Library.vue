<script setup lang="ts">
import { ref, computed } from 'vue'

export interface Book {
  id: string
  title: string
  author: string
  progress: number
  text: string
}

const emit = defineEmits<{
  (e: 'select-book', book: Book): void
}>()

// Sample book items
const books = ref<Book[]>([
  {
    id: '1',
    title: 'Alice in Wonderland',
    author: 'Lewis Carroll',
    progress: 35,
    text: `Alice was beginning to get very tired of sitting by her sister on the bank, and of having nothing to do: once or twice she had peeped into the book her sister was reading, but it had no pictures or conversations in it, “and what is the use of a book,” thought Alice “without pictures or conversations?”\n\nSo she was considering in her own mind (as well as she could, for the hot day made her feel very sleepy and stupid), whether the pleasure of making a daisy-chain would be worth the trouble of getting up and picking the daisies, when suddenly a White Rabbit with pink eyes ran close by her.\n\nThere was nothing so very remarkable in that; nor did Alice think it so very much out of the way to hear the Rabbit say to itself, “Oh dear! Oh dear! I shall be late!” (when she thought it over afterwards, it occurred to her that she ought to have wondered at this, but at the time it all seemed quite natural); but when the Rabbit actually took a watch out of its waistcoat-pocket, and looked at it, and then hurried on, Alice started to her feet, for it flashed across her mind that she had never before seen a rabbit with either a waistcoat-pocket, or a watch to take out of it, and burning with curiosity, she ran across the field after it, and fortunately was just in time to see it pop down a large rabbit-hole under the hedge.`
  },
  {
    id: '2',
    title: 'The Odyssey',
    author: 'Homer',
    progress: 12,
    text: `Tell me, O muse, of that ingenious hero who travelled far and wide after he had sacked the famous town of Troy. Many cities did he visit, and many were the nations with whose manners and customs he was acquainted; moreover he suffered much by sea while trying to save his own life and bring his men safely home; but do what he might he could not save his men, for they perished through their own sheer folly in eating the cattle of the Sun-god Hyperion; so the god prevented them from ever reaching home. Tell me, too, of all these things, oh daughter of Jove, from whatsoever source you may know them.\n\nSo now all who had escaped death by land or sea had got safely home except Ulysses, and he, though he was longing to return to his wife and country, was detained by the goddess Calypso in a spacious cave, for she wanted to marry him. Moreover, when the year had come round in which the gods had destined that he should return home to Ithaca, his troubles were still not yet over, even among his own people.`
  },
  {
    id: '3',
    title: 'Metamorphosis',
    author: 'Franz Kafka',
    progress: 88,
    text: `One morning, when Gregor Samsa woke from troubled dreams, he found himself transformed in his bed into a horrible vermin. He lay on his armour-like back, and if he lifted his head a little he could see his brown belly, slightly domed and divided by arches into stiff sections. The bedding was hardly able to cover it and seemed ready to slide off any moment. His many legs, pitifully thin compared with the size of the rest of him, waved helplessly before his eyes.\n\n“What's happened to me?” he thought. It wasn't a dream. His room, a proper human room although a little too small, lay peacefully between its four familiar walls. A collection of textile samples lay spread out on the table—Samsa was a travelling salesman—and above it there hung a picture that he had recently cut out of an illustrated magazine and housed in a nice, gilded frame. It showed a lady fitted out with a fur hat and fur boa who sat upright, raising a heavy fur muff that covered the whole of her lower arm towards the viewer.`
  },
  {
    id: '4',
    title: 'Pride and Prejudice',
    author: 'Jane Austen',
    progress: 0,
    text: `It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife.\n\nHowever little known the feelings or views of such a man may be on his first entering a neighbourhood, this truth is so well fixed in the minds of the surrounding families, that he is considered the rightful property of some one or other of their daughters.\n\n“My dear Mr. Bennet,” said his lady to him one day, “have you heard that Netherfield Park is let at last?”\n\nMr. Bennet replied that he had not.\n\n“But it is,” returned she; “for Mrs. Long has just been here, and she told me all about it.”\n\nMr. Bennet made no answer.\n\n“Do you not want to know who has taken it?” cried his wife impatiently.\n\n“You want to tell me, and I have no objection to hearing it.”\n\nThis was invitation enough.`
  }
])

const searchQuery = ref('')
const sortBy = ref<'recent' | 'title' | 'progress'>('recent')

// Filter and sort computed list
const filteredBooks = computed(() => {
  let list = books.value.filter(book => {
    const q = searchQuery.value.toLowerCase()
    return book.title.toLowerCase().includes(q) || book.author.toLowerCase().includes(q)
  })

  if (sortBy.value === 'title') {
    list.sort((a, b) => a.title.localeCompare(b.title))
  } else if (sortBy.value === 'progress') {
    list.sort((a, b) => b.progress - a.progress)
  }
  // 'recent' uses original order as a mock for last-read
  return list
})
</script>

<template>
  <div class="w-full animate-fade-in">
    <!-- Header -->
    <header class="pb-6 mb-6 border-b border-[var(--border-color)]">
      <h1 class="text-xl font-semibold tracking-tight text-[var(--text-primary)]">Library</h1>
    </header>

    <!-- Toolbar: Search Bar & Filters (Zero Icons, Typographic) -->
    <div class="flex flex-col sm:flex-row gap-4 justify-between items-stretch sm:items-center mb-8">
      
      <!-- Styled Search Input -->
      <div class="flex-1 max-w-sm">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search by title or author"
          class="w-full bg-[var(--bg-card)] border border-[var(--border-color)] text-[var(--text-primary)] text-sm rounded-md px-4 py-2 focus-ring-minimal focus:outline-none transition-all placeholder:text-[var(--text-tertiary)]"
        />
      </div>

      <!-- Sorting Filter Toggles -->
      <div class="flex items-center gap-4 text-xs">
        <span class="text-[var(--text-tertiary)] uppercase tracking-wider font-medium">Sort</span>
        <div class="flex gap-2">
          <button
            v-for="sortOption in ['recent', 'title', 'progress']"
            :key="sortOption"
            @click="sortBy = sortOption as any"
            class="px-2.5 py-1 text-xs capitalize transition-all rounded focus-ring-minimal"
            :class="sortBy === sortOption ? 'text-[var(--text-primary)] font-semibold border border-[var(--border-color)] bg-[var(--accent-color-light)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'"
          >
            {{ sortOption }}
          </button>
        </div>
      </div>
    </div>

    <!-- Editorial Book List (No outer card boxes, clean listings) -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-6">
      <div
        v-for="book in filteredBooks"
        :key="book.id"
        @click="emit('select-book', book)"
        class="group cursor-pointer py-4 border-b border-[var(--border-color)] hover:border-[var(--text-secondary)] transition-all flex flex-col gap-2"
      >
        <div class="flex justify-between items-start gap-4">
          <h2 class="text-base font-medium tracking-tight text-[var(--text-primary)] group-hover:translate-x-0.5 transition-transform duration-200">
            {{ book.title }}
          </h2>
          <span class="text-xs text-[var(--text-tertiary)] tabular-nums">{{ book.progress }}%</span>
        </div>
        <div class="flex justify-between items-center text-xs">
          <span class="text-[var(--text-secondary)]">{{ book.author }}</span>
          <!-- Minimal progress bar line -->
          <div class="w-16 h-0.5 bg-[var(--border-color)] overflow-hidden">
            <div 
              class="h-full bg-[var(--accent-color)] transition-all duration-300"
              :style="{ width: book.progress + '%' }"
            ></div>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-if="filteredBooks.length === 0" class="col-span-2 py-12 text-left">
        <p class="text-base text-[var(--text-secondary)] leading-relaxed">
          No books match your search.
        </p>
      </div>
    </div>
  </div>
</template>
