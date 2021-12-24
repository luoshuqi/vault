import Vue from 'vue'
import App from './App.vue'
import router from './router'
import vuetify from './plugins/vuetify'
import {setBackPressedListener} from "./lib/util/webview";

Vue.config.productionTip = false

setBackPressedListener(null);

new Vue({
    router,
    vuetify,
    render: h => h(App)
}).$mount('#app')