<template>
  <v-app>
    <v-app-bar app color="#ffffff" dense flat hide-on-scroll>
      <v-icon color="#323233" size="24" @click="$router.back()">{{ icon.back }}</v-icon>
      <v-app-bar-title class="ml-4">从电脑访问</v-app-bar-title>
    </v-app-bar>
    <v-main>
      <div class="pa-4">
        <div class="d-flex align-center mb-2" style="height: 48px">
          <div>允许从电脑访问</div>
          <v-spacer/>
          <div>
            <v-switch v-model="enabled" @click="toggle"/>
          </div>
        </div>
        <div v-if="enabled" class="mt-4">
          地址：<a class="blue--text" target="_blank" :href="url">{{ url }}</a>
          <div class="mt-2 text-body-2">因为使用了自签名证书，浏览器会提示页面不安全，忽略即可</div>
        </div>
        <div v-else class="mt-4 text-body-2">
          <div>开启后，可以从同一网络的电脑上通过浏览器访问</div>
        </div>
      </div>
    </v-main>
  </v-app>
</template>

<script>

import {rpc} from "../lib/rpc";
import {mdiArrowLeft} from '@mdi/js';
import {toast, getIp} from "../lib/util/compat";

export default {
  name: 'NetworkAccess',
  data() {
    return {
      enabled: false,
      port: null,
      ip: null,
      icon: {
        back: mdiArrowLeft,
      }
    }
  },
  async beforeMount() {
    this.port = await rpc.get_network_port();
    this.enabled = this.port !== null;
    this.ip = getIp();
    if (!this.ip) {
      toast('获取地址失败');
    }
  },
  methods: {
    async toggle() {
      if (this.enabled) {
        try {
          this.port = await rpc.enable_network_access();
        } catch (e) {
          this.enabled = false;
        }
      } else {
        await rpc.disable_network_access();
        this.port = null;
      }
    }
  },
  computed: {
    url() {
      return "https://" + this.ip + ':' + this.port;
    }
  }
}
</script>
