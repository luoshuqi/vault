<template>
  <v-dialog v-model="dialog" :width="$vuetify.breakpoint.xs ? '90%' : '460px'" @keydown.esc="cancel">
    <v-card>
      <v-card-title>导入</v-card-title>
      <v-card-text>
        <div>请输入导出时使用的密码。如果与当前密码一样，可不填。</div>
        <v-text-field v-model="password" placeholder="密码" type="password"></v-text-field>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn color="primary" text @click="cancel">取消</v-btn>
        <v-btn color="primary" text @click="ok">确定</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script>
export default {
  name: "ImportPasswordDialog",
  data() {
    return {
      dialog: false,
      resolve: null,
      reject: null,
      password: null,
    };
  },

  methods: {
    open() {
      this.dialog = true;
      return new Promise((resolve, reject) => {
        this.resolve = resolve;
        this.reject = reject;
      });
    },
    ok() {
      let password = this.password ? this.password : null;
      this.password = null;
      this.resolve({password});
      this.dialog = false;
    },
    cancel() {
      this.resolve(null);
      this.dialog = false;
    },
  },
};
</script>
