<template>
  <v-app>
    <v-main class="d-flex align-center">
      <v-card :flat="$vuetify.breakpoint.xs" :style="style">
        <v-card-title class="text-h5">设置密码</v-card-title>
        <v-card-text>
          <v-text-field v-model="password" label="输入密码" type="password"/>
          <v-text-field v-model="password_confirm" label="确认密码" type="password" @keyup.enter="submit"/>
        </v-card-text>
        <v-card-actions>
          <v-btn :disabled="!password || !password_confirm" block class="text-body-1"
                 color="primary" large rounded @click="submit">确定
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-main>
  </v-app>
</template>

<script>
import {toast} from "../lib/util/compat";
import {rpc} from "../lib/rpc";
import {store} from "../lib/controller";

export default {
  name: 'Setup',
  data() {
    return {
      password: '',
      password_confirm: '',
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
      if (!this.password || !this.password_confirm) return;
      if (this.password === this.password_confirm) {
        await rpc.set_master_password(this.password);
        store.isMasterPasswordSet = true;
        store.masterPassword = this.password;
        await this.$router.push({name: 'Home'});
      } else {
        toast('两次输入的密码不一致');
      }
    }
  }
}
</script>