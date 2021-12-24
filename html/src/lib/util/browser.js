// 在浏览器上使用的方法

/**
 * 读取文件
 * @returns {Promise<string|null>} 如果没有选择文件，返回 null,
 */
export async function read() {
    const input = document.createElement('input');
    input.style.display = 'none';
    input.type = 'file';
    input.value = '';

    let ok, error;
    let choose = false;
    input.addEventListener('change', () => {
        choose = true;
        const reader = new FileReader();
        reader.readAsText(input.files[0]);
        reader.onload = () => ok(reader.result);
        reader.onerror = ev => error(ev);
    })

    window.addEventListener('focus', () => {
        document.body.removeChild(input);
        choose || ok(null);
    }, {once: true});

    document.body.appendChild(input);
    input.click();

    return new Promise((resolve, reject) => {
        ok = resolve;
        error = reject;
    });
}

/**
 * 下载
 * @param {string} name 文件名
 * @param {BlobPart} content 内容
 */
export function download(name, content) {
    const a = document.createElement('a');
    a.download = name;
    a.href = URL.createObjectURL(new Blob([content]));
    a.style.display = 'none';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
}

/**
 * 复制到剪切板
 * @param {string} content
 */
export async function copyToClipboard(content) {
    await navigator.clipboard.writeText(content);
}


let toastTimer;

/**
 * 显示 toast 提示
 * @param {string} message 提示内容
 * @param {boolean} length_long 是否显示更长时间
 */
export function toast(message, length_long = false) {
    let dom = document.getElementById('toast');
    if (!dom) {
        dom = document.createElement('div');
        dom.id = "toast";
        dom.appendChild(document.createElement('p'));
        document.body.appendChild(dom);
    } else {
        toastTimer && clearTimeout(toastTimer);
    }
    dom.children[0].innerText = message;
    dom.style.display = 'flex';
    toastTimer = setTimeout(() => {
        dom.style.display = 'none'
        toastTimer = null;
    }, length_long ? 3500 : 2000);
}