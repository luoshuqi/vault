// 在 webview 上使用的方法

/**
 * 显示 toast 提示
 * @param {string} message 提示内容
 * @param {boolean} length_long 是否显示更长时间
 */
export function toast(message, length_long = false) {
    window.bridge.toast(message, length_long);
}

/**
 * 复制到剪切板
 * @param {string} content
 */
export function copyToClipboard(content) {
    window.bridge.copyToClipboard(content);
}

/**
 * 获取 app 缓存目录
 * @returns {string}
 */
export function getCacheDir() {
    return window.bridge.getCacheDir();
}

/**
 * 复制导出的文件到用户选择的路径
 * @param {string} file
 */
export function saveExportFile(file) {
    window.bridge.saveExportFile(file);
}

/**
 * 获取手机 ip 地址
 * @returns {string}
 */
export function getIp() {
    return window.bridge.getIp();
}

/**
 * 选择导入文件
 * @returns {Promise<string|null>}
 */
export function chooseImportFile() {
    let complete;
    window.completeChooseImportFile = file => complete(file);
    window.bridge.chooseImportFile();
    return new Promise(resolve => complete = resolve);
}

/**
 * 设置 app 返回事件回调
 * @param {function|null} listener
 */
export function setBackPressedListener(listener = null) {
    window.backPressedListener = () => {
        if (listener) {
            listener();
        } else {
            return true;
        }
    }
}