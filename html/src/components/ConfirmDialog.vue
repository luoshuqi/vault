<template>
  <v-dialog v-model="dialog" :width="$vuetify.breakpoint.xs ? '90%' : '460px'" @keydown.esc="cancel">
    <v-card>
      <v-card-title>{{ title }}</v-card-title>
      <v-card-text v-text="message"></v-card-text>
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
  name: "ConfirmDialog",
  data() {
    return {
      dialog: false,
      resolve: null,
      reject: null,
      message: null,
      title: null,
    };
  },

  methods: {
    open(title, message) {
      this.dialog = true;
      this.title = title;
      this.message = message;
      return new Promise((resolve, reject) => {
        this.resolve = resolve;
        this.reject = reject;
      });
    },
    ok() {
      this.resolve(true);
      this.dialog = false;
    },
    cancel() {
      this.resolve(false);
      this.dialog = false;
    },
  },
};
</script>
