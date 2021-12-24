# Vault

#### 介绍
一个密码存储 app，密码使用 `AES-256-GCM` 加密存储在本地。

#### 软件架构

使用 Rust + Vue 开发，Rust 实现主要逻辑，处理前端请求。

前端使用 Vue，通过 WebSocket JSON-RPC 调用后端服务。

app 作为一个启动器，启动后端服务，调用 WebView 显示界面。

#### 截图

![unlock](https://gitee.com/luoshuqi/vault/raw/master/screenshots/unlock.jpg)

![home](https://gitee.com/luoshuqi/vault/raw/master/screenshots/home.jpg)

![add](https://gitee.com/luoshuqi/vault/raw/master/screenshots/add.jpg)