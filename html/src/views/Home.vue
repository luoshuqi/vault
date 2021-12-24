<template>
  <v-app :class="{bg: !$vuetify.breakpoint.xs}">
    <v-app-bar v-if="$vuetify.breakpoint.xs" app color="#ffffff" flat>
      <div id="search" class="mr-3">
        <v-text-field v-model="search" dense filled placeholder="搜索" rounded/>
      </div>

      <v-icon color="#323233" left size="28" @click="toAddPage">{{ icon.plus }}</v-icon>

      <v-menu offset-y origin="center center" transition="scale-transition">
        <template v-slot:activator="{ on, attrs }">
          <v-icon color="#323233" size="28" v-bind="attrs" v-on="on">{{ icon.dotsVertical }}</v-icon>
        </template>
        <v-list>
          <v-list-item :disabled="!list || !list.length" @click="exportPassword">
            <v-list-item-title>导出</v-list-item-title>
          </v-list-item>
          <v-list-item @click="importPassword">
            <v-list-item-title>导入</v-list-item-title>
          </v-list-item>
          <v-list-item @click="toChangePasswordPage">
            <v-list-item-title>修改密码</v-list-item-title>
          </v-list-item>
          <v-list-item @click="$router.push({name: 'NetworkAccess'})">
            <v-list-item-title>从电脑访问</v-list-item-title>
          </v-list-item>
          <v-list-item @click="$router.push({name: 'About'})">
            <v-list-item-title>关于</v-list-item-title>
          </v-list-item>
        </v-list>
      </v-menu>
    </v-app-bar>
    <v-main style="height: calc(100% - 64px);">
      <v-card :flat="$vuetify.breakpoint.xs" :style="style">
        <div v-if="!$vuetify.breakpoint.xs" class="pl-2 pr-2" style="display: flex">
          <v-text-field v-model="search" dense placeholder="搜索"/>
          <div class="ml-6">
            <v-btn color="primary" @click="toAddPage">新建</v-btn>
            <v-menu offset-y origin="center center" transition="scale-transition">
              <template v-slot:activator="{ on, attrs }">
                <v-btn class="ml-4" color="success" v-bind="attrs" v-on="on">菜单</v-btn>
              </template>
              <v-list>
                <v-list-item :disabled="!list || !list.length" @click="exportPassword">
                  <v-list-item-title>导出</v-list-item-title>
                </v-list-item>
                <v-list-item @click="importPassword">
                  <v-list-item-title>导入</v-list-item-title>
                </v-list-item>
                <v-list-item @click="toChangePasswordPage">
                  <v-list-item-title>修改密码</v-list-item-title>
                </v-list-item>
                <v-list-item @click="$router.push({name: 'About'})">
                  <v-list-item-title>关于</v-list-item-title>
                </v-list-item>
              </v-list>
            </v-menu>
          </div>
        </div>
        <div v-if="searchEmpty" class="ml-4 mt-4 text-body-1">
          未找到任何结果
        </div>
        <v-list v-else-if="list && list.length" :style="{height: listHeight, 'overflow-y': 'auto'}">
          <v-list-item-group v-model="selected">
            <template v-for="item in list">
              <template v-if="!search || item.name.toLowerCase().indexOf(search.toLowerCase()) !== -1">
                <v-list-item :key="item.id" :value="item.id">
                  <v-list-item-content>
                    <v-list-item-title v-text="item.name"/>
                  </v-list-item-content>
                  <v-list-item-action v-if="selected === item.id" class="d-flex flex-row">
                    <v-icon color="primary" @click.stop="copy(item.id)">{{ icon.contentCopy }}</v-icon>
                    <v-icon color="success" right @click.stop="toEditPage(item.id)">{{
                        icon.pencilOutline
                      }}
                    </v-icon>
                    <v-icon color="warning" right @click.stop="show(item.id)">{{ icon.eyeOutline }}</v-icon>
                    <v-icon color="error" right @click.stop="erase(item.id, item.name)">{{
                        icon.deleteOutline
                      }}
                    </v-icon>
                  </v-list-item-action>
                </v-list-item>
              </template>
            </template>
          </v-list-item-group>
        </v-list>
        <div v-else-if="list !== null" class="d-flex align-center justify-center"
             style="height: 100%;">
          <v-btn depressed @click="toAddPage">新建</v-btn>
          <v-btn class="ml-2" depressed @click="importPassword">导入</v-btn>
        </div>
      </v-card>
      <ConfirmDialog ref="confirm"/>
      <ImportPasswordDialog ref="import_password"/>
    </v-main>
  </v-app>
</template>

<script>

import {rpc} from "../lib/rpc";
import {exportPassword, importPassword, store} from "../lib/controller";
import {mdiContentCopy, mdiDeleteOutline, mdiDotsVertical, mdiEyeOutline, mdiPencilOutline, mdiPlus} from "@mdi/js";
import ConfirmDialog from "../components/ConfirmDialog";
import ImportPasswordDialog from "../components/ImportPasswordDialog";
import {copyToClipboard, toast} from "../lib/util/compat";

export default {
  name: 'Home',
  components: {
    ImportPasswordDialog,
    ConfirmDialog
  },
  data() {
    return {
      list: null,
      selected: 0,
      search: '',
      icon: {
        plus: mdiPlus,
        dotsVertical: mdiDotsVertical,
        contentCopy: mdiContentCopy,
        pencilOutline: mdiPencilOutline,
        eyeOutline: mdiEyeOutline,
        deleteOutline: mdiDeleteOutline,
      }
    }
  },
  async beforeMount() {
    await this.listPassword();
  },
  computed: {
    style() {
      return this.$vuetify.breakpoint.xs ? {height: '100%'} :
          {width: '500px', margin: '16px auto 0 auto', padding: '16px', height: 'calc(100% - 32px)'};
    },
    listHeight() {
      return this.$vuetify.breakpoint.xs ? '100%' : 'calc(100% - 48px)'
    },
    searchEmpty() {
      if (!this.search) return false;
      if (!this.list || !this.list.length) return true;

      for (const item of this.list) {
        if (item.name.toLowerCase().indexOf(this.search.toLowerCase()) !== -1) {
          return false;
        }
      }
      return true;
    }
  },
  methods: {
    toAddPage() {
      this.$router.push({name: 'Add'});
    },
    async exportPassword() {
      await exportPassword()
    },
    async importPassword() {
      await importPassword(this.$refs.import_password.open) && await this.listPassword();
    },
    async toChangePasswordPage() {
      await this.$router.push('/change-password');
    },
    async erase(id, name) {
      if (await this.$refs.confirm.open("确认", "将删除 " + name)) {
        await rpc.delete_password(store.masterPassword, id);
        toast('已删除');
        await this.listPassword();
      }
    },
    toEditPage(id) {
      this.$router.push('/edit/' + id);
    },
    async show(id) {
      toast((await rpc.get_password(store.masterPassword, id)).password);
    },
    async copy(id) {
      try {
        await copyToClipboard((await rpc.get_password(store.masterPassword, id)).password);
        toast('已复制');
      } catch (e) {
        toast('复制失败');
        console.error(e);
      }
    },
    async listPassword() {
      this.list = await rpc.list_password(store.masterPassword);
    }
  }
}

</script>

<style>
#search {
  height: 32px;
  min-height: 32px;
  flex-grow: 1;
}

#search .v-input__slot {
  height: 32px !important;
  min-height: 32px !important;
  font-size: 13px;
}
</style>