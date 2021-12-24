<template>
  <v-app>
    <v-main class="d-flex align-center">
      <v-card :flat="$vuetify.breakpoint.xs" :style="style">
        <v-card-title class="text-h5">解锁</v-card-title>
        <v-card-text>
          <v-text-field v-model="password" label="密码" type="password" @keydown.enter="submit"/>
        </v-card-text>
        <v-card-actions>
          <v-btn :disabled="!password" block class="text-body-1" color="primary" large rounded @click="submit">确定
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-main>
  </v-app>
</template>

<script>
import {store} from "../lib/controller";
import {toast} from "../lib/util/compat";
import {rpc} from "../lib/rpc";

export default {
  name: 'Unlock',
  data() {
    return {
      password: '',
    }
  },
  computed: {
    style() {
      return this.$vuetify.breakpoint.xs ? {} :
          {width: '460px', margin: '0 auto', padding: '16px'};
    }
  },
  methods: {
    async submit() {
      if (!this.password) return;
      if (await rpc.verify_master_password(this.password)) {
        store.masterPassword = this.password
        await this.$router.push({name: 'Home'})
      } else {
        toast('密码错误');
      }
    }
  }
}
</script>