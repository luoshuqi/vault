import {isWebView, toast} from "./util/compat";
import {rpc} from "./rpc";
import {download, read} from "./util/browser";
import {chooseImportFile, getCacheDir, saveExportFile} from "./util/webview";

export const store = {
    // 是否设置了主密码
    isMasterPasswordSet: null,

    // 主密码
    masterPassword: undefined,
}

/**
 * @typedef DecryptPassword
 * @property {string|null} password
 *
 * @callback GetDecryptPassword
 * @return {Promise<DecryptPassword|null>}
 */

/**
 * 导入密码
 * @param {GetDecryptPassword} getDecryptPassword
 * @returns {Promise<boolean>} 如果没有数据被导入，返回 false
 */
export async function importPassword(getDecryptPassword) {
    let data;
    if (isWebView()) {
        data = await chooseImportFile();
        if (data === null) return false;
    } else {
        data = await read();
        if (data === null) return false;
        data = decodeImportData(data);
        if (data === null) {
            toast('解析文件失败');
            return false;
        }
    }

    let decryptPassword = await getDecryptPassword();
    if (decryptPassword === null) return false;

    let count = await rpc.import_password(store.masterPassword, decryptPassword.password, data);
    let msg = "导入了 " + count.insert + " 个密码";
    if (count.ignore > 0) {
        msg += ", 忽略了 " + count.ignore + " 个重复密码";
    }
    toast(msg);

    return count.insert > 0;
}

/**
 * 导出密码
 * @returns {Promise<void>}
 */
export async function exportPassword() {
    if (isWebView()) {
        let file = getCacheDir() + "/" + (new Date()).getTime();
        await rpc.export_password(store.masterPassword, file);
        saveExportFile(file);
    } else {
        let data = await rpc.export_password(store.masterPassword, null);
        download(makeExportFilename(), JSON.stringify(data));
    }
}

/**
 * 生成带日期时间的导出文件名
 * @returns {string}
 */
function makeExportFilename() {
    const pad = v => (v < 10 ? "0" : "") + v;
    const date = new Date();
    return '密码_' + date.getFullYear() + pad(date.getMonth() + 1) + pad(date.getDate())
        + "_" + pad(date.getHours()) + pad(date.getMinutes()) + pad(date.getSeconds()) + ".txt";
}

/**
 * json decode 要导入的数据
 * @param {string} data
 * @returns {string|null} 如果格式不正确，返回 null
 */
function decodeImportData(data) {
    try {
        data = JSON.parse(data);
    } catch (e) {
        return null;
    }

    const isArray = v => v.constructor.name === 'Array';
    const isString = v => v.constructor.name === 'String';

    if (!isArray(data)) {
        return null;
    }

    for (const item of data) {
        if (!isArray(item) || item.length !== 2 || !isString(item[0]) || !isString(item[1])) {
            return null;
        }
    }

    return data;
}