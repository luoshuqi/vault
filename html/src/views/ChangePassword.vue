<template>
  <v-app :class="{'bg': !$vuetify.breakpoint.xs}">
    <v-app-bar v-if="$vuetify.breakpoint.xs" app color="#ffffff" dense flat hide-on-scroll>
      <v-icon color="#323233" size="24" @click="$router.back()">{{ icon.close }}</v-icon>
      <v-spacer/>
      <v-app-bar-title>修改密码</v-app-bar-title>
      <v-spacer/>
      <v-icon :disabled="!current_password || !new_password || !password_confirm" color="#323233" size="28"
              @click="submit">{{
          icon.check
        }}
      </v-icon>
    </v-app-bar>
    <v-main :class="{'d-flex': !$vuetify.breakpoint.xs, 'align-center': !$vuetify.breakpoint.xs}">
      <v-card :flat="$vuetify.breakpoint.xs" :style="style">
        <v-card-title v-if="!$vuetify.breakpoint.xs">修改密码</v-card-title>
        <v-card-subtitle class="orange--text">修改密码后，导入以前导出的数据需要提供导出时使用的密码</v-card-subtitle>
        <v-card-text class="text-body-1">
          <v-text-field v-model="current_password" label="当前密码" type="password"></v-text-field>
          <v-text-field v-model="new_password" label="新密码" type="password"></v-text-field>
          <v-text-field v-model="password_confirm" label="确认新密码" type="password"></v-text-field>
        </v-card-text>
        <v-card-actions v-if="!$vuetify.breakpoint.xs">
          <v-btn @click="$router.back()">取消</v-btn>
          <v-spacer/>
          <v-btn :disabled="!current_password || !new_password || !password_confirm" color="primary"
                 @click="submit">确认
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-main>
  </v-app>
</template>

<script>

import {rpc} from "../lib/rpc";
import {store} from "../lib/controller";
import {mdiCheck, mdiClose} from '@mdi/js';
import {toast} from "../lib/util/compat";

export default {
  name: 'ChangePassword',
  data() {
    return {
      current_password: '',
      new_password: '',
      password_confirm: '',
      icon: {
        close: mdiClose,
        check: mdiCheck,
      }
    }
  },
  computed: {
    style() {
      return this.$vuetify.breakpoint.xs ? {} :
          {width: '460px', margin: '0 auto', padding: '16px'};
    },
  },
  methods: {
    async submit() {
      if (this.current_password !== store.masterPassword) {
        toast('当前密码错误');
        return;
      }

      if (this.new_password !== this.password_confirm) {
        toast('两次输入的密码不一致');
        return;
      }

      await rpc.change_password(this.current_password, this.new_password);
      store.masterPassword = this.new_password;
      toast('密码已修改');
      await this.$router.back();
    },
  }
}
</script>
