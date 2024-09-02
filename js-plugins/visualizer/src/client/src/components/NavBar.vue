<template>
  <nav class="bg-gray-800">
    <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
      <div class="flex h-16 items-center justify-between">
        <div class="flex items-center">
          <div class="flex-shrink-0">
            <img
              class="h-8 w-8"
              src="../assets/logo.png"
              alt="Farm"
            />
          </div>
          <div class="hidden md:block">
            <div class="ml-10 flex items-baseline space-x-4">
              <router-link
                v-for="item in navigation"
                :key="item.name"
                :to="item.href"
                :class="['text-gray-300 hover:bg-gray-700 hover:text-white rounded-md px-3 py-2 text-sm font-medium', route.path === item.href ? 'bg-gray-900 text-white' : '']"
                >{{ item.name }}</router-link
              >
            </div>
          </div>
        </div>

        <div class="-mr-2 flex md:hidden">
          <!-- Mobile menu button -->
          <button
            @click="isOpen = !isOpen"
            type="button"
            class="relative inline-flex items-center justify-center rounded-md bg-gray-800 p-2 text-gray-400 hover:bg-gray-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800"
            aria-controls="mobile-menu"
            aria-expanded="false"
          >
            <span class="absolute -inset-0.5"></span>
            <span class="sr-only">Open main menu</span>
            <!-- Menu open: "hidden", Menu closed: "block" -->
            <svg
              v-show="!isOpen"
              class="block h-6 w-6"
              fill="none"
              viewBox="0 0 24 24"
              stroke-width="1.5"
              stroke="currentColor"
              aria-hidden="true"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
              />
            </svg>
            <!-- Menu open: "block", Menu closed: "hidden" -->
            <svg
              v-show="isOpen"
              class="block h-6 w-6"
              fill="none"
              viewBox="0 0 24 24"
              stroke-width="1.5"
              stroke="currentColor"
              aria-hidden="true"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Mobile menu, show/hide based on menu state. -->
    <div class="md:hidden" id="mobile-menu" v-show="isOpen">
      <div class="space-y-1 px-2 pb-3 pt-2 sm:px-3">
        <router-link
          v-for="item in navigation"
          :key="item.name"
          :to="item.href"
          :class="['block text-gray-300 hover:bg-gray-700 hover:text-white rounded-md px-3 py-2 text-base font-medium', route.path === item.href ? 'bg-gray-900 text-white' : '']"
          >{{ item.name }}</router-link>
      </div>
    </div>
  </nav>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { useRoute } from 'vue-router';

export default defineComponent({
  name: 'NavBar',
  setup() {
    const isOpen = ref(false);
    const route = useRoute();

    const navigation = [
      { name: 'Dashboard', href: '/dashboard' },
      { name: 'Compilation Analysis', href: '/analysis/compilation' },
      // { name: 'Compilation Context', href: '/analysis/module' },
      { name: 'Plugin Performance', href: '/analysis/plugin' },
      { name: 'Bundle Analysis', href: '/analysis/bundle' }
    ];
    return { isOpen, navigation, route };
  }
});
</script>
