import {toast} from "./util/compat";
import Client from "./util/jsonrpc_client";

const handler = {
    get: function (target, prop) {
        return prop in target ? target[prop] : async (...args) => {
            try {
                return await target.call(prop, ...args);
            } catch (e) {
                if ("response" in e) {
                    showError(e.response);
                } else if ("error" in e) {
                    toast('连接出错');
                } else if ("close" in e) {
                    toast('连接断开');
                }
                throw e;
            }
        }
    }
}

export const rpc = new Proxy(new Client(getUrl()), handler);

function getUrl() {
    let host = location.host;
    if (process.env.NODE_ENV !== 'production') {
        // 端口写死为 8000，方便测试
        host = host.replace(/:\d+/, ":8000")
    }
    let protocol = location.protocol.replace(/^http/, 'ws');
    return protocol + '//' + host + '/ws';
}

const msg = {
    WrongPassword: '密码错误',
    DeserializeFailed: '解析文件失败',
}

/**
 * 显示错误提示
 * @param response
 */
function showError(response) {
    if (response.error.data && response.error.data.kind) {
        let kind = response.error.data.kind;
        toast(kind in msg ? msg[kind] : "出错了(" + kind + ")")
    } else {
        toast("出错了(" + response.error.message + ")")
    }
}