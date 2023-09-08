/**
 * 
 * @param {number} totalMs 
 * @param {number} precision
 * @returns {string}
 */
export function formatDuration(totalMs, precision = 1) {
    if (totalMs < 0) {
        return "-" + formatDuration(Math.abs(totalMs));
    }

    var total_sec = totalMs / 1000;
    var h = Math.floor(total_sec / 3600);
    total_sec = total_sec % 3600;

    var m = Math.floor(total_sec / 60);
    var s = total_sec % 60;
    
    var ret = "";
    if (h >= 1) {
        ret = ((h < 10) ? "0" : "") + h.toFixed(0) + ":";
    }

    ret += ((m < 10) ? "0" : "") + m.toFixed(0) + ":";
    ret += ((s < 10) ? "0" : "") + s.toFixed(precision);

    return ret;
}

/**
 * 
 * @param {string} tag 
 * @param {{ className: string?, id: string?, text: string?, parent: HTMLElement?, children: HTMLElement[]? }?} opts
 * @returns {HTMLElement}
 */
export function newElement(tag, opts = null) {
    var ret = document.createElement(tag);

    if (opts) {
        if (opts.className) {
            ret.className = opts.className;
        }

        if (opts.id) {
            ret.id = opts.id;
        }

        if (opts.text) {
            ret.innerText = opts.text;
        }

        if (opts.parent) {
            opts.parent.appendChild(ret);
        }

        if (opts.children) {
            Element.prototype.replaceChildren.apply(ret, opts.children);
        }
    }

    return ret;
}
