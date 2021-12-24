const DISCONNECTED = 0; // 未连接
const CONNECTING = 1; // 正在连接
const CONNECTED = 2; // 已连接

/**
 * 连接 websocket
 * @callback messageCallback
 * @param {MessageEvent} ev
 */
class Connector {
    /**
     * constructor
     * @param {string} url 地址
     * @param {messageCallback} onmessage 消息回调
     */
    constructor(url, onmessage) {
        this.url = url;
        this.onmessage = onmessage;
        this.state = DISCONNECTED;
        this.ws = null;
    }

    /**
     * 获取已有的连接，或者新建连接
     * @returns {Promise<WebSocket>}
     */
    getConnection() {
        return new Promise((resolve, reject) => {
            if (this.state === CONNECTED) {
                resolve(this.ws);
            } else if (this.state === DISCONNECTED) {
                this.connect().then(ws => resolve(ws), err => reject(err));
            } else {
                setTimeout(() => this.connect().then(ws => resolve(ws), err => reject(err)), 100);
            }
        })
    }

    /**
     * 连接 websocket
     * @returns {Promise<WebSocket>}
     */
    connect() {
        return new Promise((resolve, reject) => {
            this.state = CONNECTING;
            this.ws = new WebSocket(this.url);
            this.ws.onmessage = e => this.onmessage(e);
            this.ws.onopen = _ => {
                this.state = CONNECTED;
                resolve(this.ws);
            }
            this.ws.onerror = e => {
                this.state = DISCONNECTED;
                reject({error: e});
            }
            this.ws.onclose = e => {
                this.state = DISCONNECTED;
                reject({close: e});
            }
        })
    }
}

/**
 * JsonRpc 客户端
 */
export default class Client {
    constructor(url) {
        this.id = 1;
        this.completer = {};
        this.connector = new Connector(url, e => {
            let response = JSON.parse(e.data);
            if (response.id) {
                let {resolve, reject} = this.completer[response.id];
                if (!response.error) {
                    resolve(response.result);
                } else {
                    reject({response});
                }
                delete this.completer[response.id];
            } else {
                console.error("response without id: ", response)
            }
        });
    }

    /**
     * 调用 rpc 方法
     * @param {string} method 方法名
     * @param {unknown} args 参数
     * @returns {Promise<unknown>}
     */
    call(method, ...args) {
        return new Promise((resolve, reject) => {
            this.connector.getConnection().then(ws => {
                let request = {jsonrpc: "2.0", id: this.id++, method, params: args};
                this.completer[request.id] = {resolve, reject};
                ws.send(JSON.stringify(request));
            }, err => reject(err));
        })
    }
}