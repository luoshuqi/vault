<template>
  <v-app :class="{'bg': !$vuetify.breakpoint.xs}">
    <v-app-bar v-if="$vuetify.breakpoint.xs" app color="#ffffff" dense flat hide-on-scroll>
      <v-icon color="#323233" size="24" @click="$router.back()">{{ icon.close }}</v-icon>
      <v-spacer/>
      <v-app-bar-title>{{ this.id ? '修改密码' : '新建密码' }}</v-app-bar-title>
      <v-spacer/>
      <v-icon :disabled="!submitReady" color="#323233" size="28" @click="submit">{{ icon.check }}
      </v-icon>
    </v-app-bar>
    <v-main :class="{'d-flex': !$vuetify.breakpoint.xs, 'align-center': !$vuetify.breakpoint.xs}">
      <v-card :flat="$vuetify.breakpoint.xs" :style="style">
        <v-card-title v-if="!$vuetify.breakpoint.xs">
          {{ this.id ? '修改密码' : '新建密码' }}
        </v-card-title>
        <v-card-text class="text-body-1">
          <div>
            <v-text-field v-model="form.name" label="名称"></v-text-field>
            <v-text-field v-model="form.password" label="密码"></v-text-field>
          </div>
          <div>
            <div class="d-flex align-center mt-2" style="height: 48px;">
              <div>长度</div>
              <v-spacer/>
              <div>
                <v-icon :disabled="options.len < 2" color="primary" @click="decreaseLen">{{ icon.minus }}</v-icon>
                <input v-model.number="options.len"
                       style="text-align: center; border: none; outline: none; width: 40px;">
                <v-icon :disabled="options.len >= 2048" color="primary" @click="increaseLen">{{ icon.plus }}</v-icon>
              </div>
            </div>
            <div class="d-flex align-center" style="height: 48px">
              <div>大写字母</div>
              <v-spacer/>
              <div>
                <v-switch v-model="options.uppercase"/>
              </div>
            </div>
            <div class="d-flex align-center" style="height: 48px">
              <div>小写字母</div>
              <v-spacer/>
              <div>
                <v-switch v-model="options.lowercase"/>
              </div>
            </div>
            <div class="d-flex align-center" style="height: 48px">
              <div>数字</div>
              <v-spacer/>
              <div>
                <v-switch v-model="options.digit"/>
              </div>
            </div>
            <div class="d-flex align-center" style="height: 48px">
              <div>特殊字符</div>
              <v-spacer/>
              <div>
                <v-switch v-model="options.special"/>
              </div>
            </div>
          </div>
        </v-card-text>
        <v-card-actions v-if="!$vuetify.breakpoint.xs">
          <v-btn @click="$router.back()">取消</v-btn>
          <v-spacer/>
          <v-btn :disabled="!submitReady" color="primary" @click="submit">确认</v-btn>
        </v-card-actions>
      </v-card>
    </v-main>
  </v-app>
</template>

<script>

import {rpc} from "../lib/rpc";
import {store} from "../lib/controller";
import {mdiCheck, mdiClose, mdiMinus, mdiPlus} from '@mdi/js';
import {toast} from '../lib/util/compat';

export default {
  name: 'Add',
  data() {
    return {
      options: {
        len: 16,
        uppercase: true,
        lowercase: true,
        digit: true,
        special: true,
      },
      form: {
        name: '',
        password: '',
      },
      id: 0,
      edit: {
        name: '',
        password: '',
      },
      icon: {
        close: mdiClose,
        check: mdiCheck,
        minus: mdiMinus,
        plus: mdiPlus,
      }
    }
  },
  computed: {
    style() {
      return this.$vuetify.breakpoint.xs ? {} :
          {width: '460px', margin: '0 auto', padding: '16px'};
    },
    submitReady() {
      return this.form.name && this.form.password
          && !(this.form.name === this.edit.name && this.form.password === this.edit.password)
    },
  },
  watch: {
    options: {
      async handler() {
        await this.makePassword();
      },
      deep: true,
    }
  },
  async mounted() {
    this.id = 'id' in this.$route.params ? parseInt(this.$route.params.id) : 0;
    isNaN(this.id) && (this.id = 0);
    if (this.id) {
      this.edit = await rpc.get_password(store.masterPassword, this.id)
      this.form.name = this.edit.name;
      this.form.password = this.edit.password;
    } else {
      await this.makePassword();
    }
  },
  methods: {
    async makePassword() {
      if (/[1-9]\d*/.test(this.options.len + "")) {
        this.options.len = parseInt(this.options.len);
        this.form.password = await rpc.make_password(this.options);
      } else {
        this.form.password = '';
      }
    },
    async submit() {
      if (this.id) {
        await rpc.update_password(store.masterPassword, this.id, this.form.name, this.form.password);
        toast('已更新');
      } else {
        await rpc.add_password(store.masterPassword, this.form.name, this.form.password);
        toast('已创建');
      }
      await this.$router.back();
    },
    increaseLen() {
      let len = normalize_len(this.options.len);
      this.options.len = Math.min(2048, ++len);
    },
    decreaseLen() {
      let len = normalize_len(this.options.len);
      this.options.len = Math.max(1, --len);
    },
  }
}

function normalize_len(len) {
  len = parseInt(len);
  if (isNaN(len) || len < 1) {
    len = 16;
  }
  return len;
}
</script>
