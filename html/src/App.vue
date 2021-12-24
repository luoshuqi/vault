<template>
  <transition :name="transition">
    <router-view/>
  </transition>
</template>

<script>

export default {
  name: 'App',
  data() {
    return {
      transition: 'slide-x-transition',
    }
  },
  watch: {
    $route(to, from) {
      this.transition = to.meta.level > from.meta.level ? 'slide-x-transition' : 'slide-x-reverse-transition';
    }
  },
};
</script>

<style>
html, body, #app {
  height: 100%;
  overflow: hidden !important;
}

.bg {
  background-color: #eee !important;
}

#toast {
  position: fixed;
  left: 0;
  right: 0;
  margin: auto;
  bottom: 100px;
  display: flex;
  justify-content: center;
}

#toast > p {
  background-color: #272822;
  color: white;
  padding: 8px 16px;
  border-radius: 6px;
}
</style>