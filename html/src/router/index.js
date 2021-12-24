import Vue from 'vue'
import VueRouter from 'vue-router'
import Home from '../views/Home.vue'
import Setup from '../views/Setup'
import Unlock from "../views/Unlock";
import {store} from "../lib/controller";
import Add from "../views/Add";
import ChangePassword from "../views/ChangePassword";
import About from "../views/About";
import NetworkAccess from "../views/NetworkAccess";
import {setBackPressedListener} from "../lib/util/webview";
import {rpc} from "../lib/rpc";


Vue.use(VueRouter)

const routes = [
    {path: '/', name: 'Home', component: Home, meta: {level: 10}},
    {path: '/setup', name: 'Setup', component: Setup, meta: {level: 1}},
    {path: '/unlock', name: 'Unlock', component: Unlock, meta: {level: 1}},
    {path: '/add', name: 'Add', component: Add, meta: {back: true, level: 20}},
    {path: '/edit/:id', name: 'Edit', component: Add, meta: {back: true, level: 20}},
    {path: '/change-password', name: 'ChangePassword', component: ChangePassword, meta: {back: true, level: 20}},
    {path: '/network-access', name: 'NetworkAccess', component: NetworkAccess, meta: {back: true, level: 20}},
    {path: '/about', name: 'About', component: About, meta: {back: true, level: 20}},
]

const router = new VueRouter({
    routes
})

router.beforeEach(async (to, from, next) => {
    setBackPressedListener(to.meta.back ? () => router.back() : null);

    if (store.isMasterPasswordSet === null) {
        store.isMasterPasswordSet = await rpc.is_master_password_set();
    }

    if (!store.isMasterPasswordSet) {
        if (to.name !== 'Setup') {
            next({name: 'Setup'})
        } else {
            next()
        }
        return;
    }

    if (!store.masterPassword) {
        if (to.name !== 'Unlock') {
            next({name: 'Unlock'})
        } else {
            next()
        }
        return;
    }

    if (to.name === 'Setup' || to.name === 'Unlock') {
        next({name: 'Home'})
    } else {
        next()
    }
})

export default router
