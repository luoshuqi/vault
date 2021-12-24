// 浏览器和 webview 都可使用的方法

import { toast as wToast, copyToClipboard as wCopyToClipboard, getIp as wGetIp } from './webview'
import { toast as bToast, copyToClipboard as bCopyToClipboard } from './browser'

// 获取 ip
export function getIp() {
    return isWebView() ? wGetIp() : location.hostname;
}

/**
 * 判断是否在 webview 中
 * @returns {boolean}
 */
export function isWebView() {
    return "bridge" in window;
}

/**
 * 显示 toast 提示
 * @param {string} message 提示内容
 * @param {boolean} length_long 是否显示更长时间
 */
export function toast(message, length_long = false) {
    isWebView() ? wToast(message, length_long) : bToast(message, length_long);
}

/**
 * 复制到剪切板
 * @param {string} content
 */
export async function copyToClipboard(content) {
    isWebView() ? wCopyToClipboard(content) : await bCopyToClipboard(content);
}