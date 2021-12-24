<template>
  <v-app>
    <v-app-bar app color="#ffffff" dense flat hide-on-scroll>
      <v-icon color="#323233" size="24" @click="$router.back()">{{ icon.back }}</v-icon>
      <v-app-bar-title class="ml-4">关于</v-app-bar-title>
    </v-app-bar>
    <v-main class="d-flex align-center">
      <div class="text-center text-body-1">Vault</div>
      <div class="text-center mt-2">
        <a href="https://gitee.com/luoshuqi/vault" target="_blank">https://gitee.com/luoshuqi/vault</a>
      </div>
    </v-main>
  </v-app>
</template>

<script>
import {mdiArrowLeft} from '@mdi/js';
export default {
  name: 'About',
  data() {
    return {
      icon: {
        back: mdiArrowLeft,
      }
    }
  },
}
</script>
