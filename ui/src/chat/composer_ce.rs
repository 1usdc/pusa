//! 单 contenteditable 输入区：打字留在 DOM，Rust 只在插/删 chip 与提交时同步。

use dioxus::document;
use serde::{Deserialize, Serialize};

use super::attachments::ChatPendingAttachment;

pub const COMPOSER_ROOT_MAIN: &str = "main";
pub const COMPOSER_ROOT_EDIT: &str = "edit";

/// 长期挂起的 JS 桥：事件委托 + MutationObserver，经 `dioxus.send` 回传。
pub const INSTALL_BRIDGE_JS: &str = r#"(async () => {
  const esc = (s) => String(s ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');

  const FOLDER_SVG = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>';
  const FILE_SVG = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/></svg>';
  const X_SVG = '<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>';

  let suppress = 0;
  const withSuppress = (fn) => {
    suppress++;
    try { return fn(); } finally { suppress--; }
  };

  const rootEl = (id) => document.querySelector('[data-ac-composer="' + id + '"]');

  const isSendable = (el) => {
    if (!el) return false;
    if (el.querySelector('[data-ac-attach-id]')) return true;
    const t = (el.innerText || '').replace(/\u200b/g, '').replace(/\u00a0/g, ' ').trim();
    return t.length > 0;
  };

  const syncEmptyClass = (el) => {
    if (!el) return;
    const empty = !isSendable(el) && !(el.innerText || '').replace(/\u200b/g, '').trim();
    el.classList.toggle('is-empty', empty || el.childNodes.length === 0);
  };

  const emit = (msg) => {
    try { dioxus.send(msg); } catch (_) {}
  };

  const serialize = (id) => {
    const el = rootEl(id);
    if (!el) return { text: '', attach_ids: [], parts: [] };
    const parts = [];
    let textBuf = '';
    const flushText = () => {
      const t = (textBuf || '').replace(/\u200b/g, '');
      textBuf = '';
      if (t) parts.push({ kind: 'text', text: t, id: 0 });
    };
    const walk = (node, inRoot) => {
      if (node.nodeType === 3) {
        textBuf += node.nodeValue || '';
        return;
      }
      if (node.nodeType !== 1) return;
      const tag = node.tagName;
      if (node.hasAttribute && node.hasAttribute('data-ac-attach-id')) {
        const aid = Number(node.getAttribute('data-ac-attach-id'));
        flushText();
        if (!Number.isNaN(aid)) parts.push({ kind: 'attach', text: '', id: aid });
        return;
      }
      if (tag === 'BR') {
        textBuf += '\n';
        return;
      }
      if ((tag === 'DIV' || tag === 'P') && !inRoot) {
        if (textBuf.length > 0 && !textBuf.endsWith('\n')) textBuf += '\n';
      }
      const kids = node.childNodes;
      for (let i = 0; i < kids.length; i++) walk(kids[i], false);
      if ((tag === 'DIV' || tag === 'P') && !inRoot) {
        if (textBuf.length > 0 && !textBuf.endsWith('\n')) textBuf += '\n';
      }
    };
    walk(el, true);
    flushText();
    const attach_ids = parts.filter((p) => p.kind === 'attach').map((p) => p.id);
    const text = parts.filter((p) => p.kind === 'text').map((p) => p.text).join('');
    return { text, attach_ids, parts };
  };

  const closestChip = (node, root) => {
    if (!node || !root) return null;
    const el = node.nodeType === 1 ? node : node.parentElement;
    if (!el || !el.closest) return null;
    const chip = el.closest('[data-ac-attach-id]');
    return chip && root.contains(chip) ? chip : null;
  };

  const placeCaretInText = (textNode, offset) => {
    const sel = window.getSelection();
    if (!sel || !textNode) return;
    const range = document.createRange();
    const len = (textNode.nodeValue || '').length;
    const o = Math.max(0, Math.min(offset, len));
    range.setStart(textNode, o);
    range.collapse(true);
    sel.removeAllRanges();
    sel.addRange(range);
  };

  const isZwsOnlyText = (n) =>
    !!(n && n.nodeType === 3 && !(n.nodeValue || '').replace(/\u200b/g, '').length);

  /** 若 caret 紧贴某 chip 右侧（中间只有 ZWS），返回该 chip。 */
  const chipLeftOfCaret = (node, offset) => {
    if (!node) return null;
    if (node.nodeType === 3) {
      const visibleBefore = (node.nodeValue || '').slice(0, offset).replace(/\u200b/g, '');
      if (visibleBefore.length > 0) return null;
      let n = node.previousSibling;
      while (isZwsOnlyText(n)) n = n.previousSibling;
      if (n && n.nodeType === 1 && n.hasAttribute && n.hasAttribute('data-ac-attach-id')) return n;
      return null;
    }
    if (node.nodeType === 1) {
      let i = offset - 1;
      while (i >= 0 && isZwsOnlyText(node.childNodes[i])) i--;
      const n = i >= 0 ? node.childNodes[i] : null;
      if (n && n.nodeType === 1 && n.hasAttribute && n.hasAttribute('data-ac-attach-id')) return n;
    }
    return null;
  };

  /** 若 caret 紧贴某 chip 左侧（中间只有 ZWS），返回该 chip。 */
  const chipRightOfCaret = (node, offset) => {
    if (!node) return null;
    if (node.nodeType === 3) {
      const visibleAfter = (node.nodeValue || '').slice(offset).replace(/\u200b/g, '');
      if (visibleAfter.length > 0) return null;
      let n = node.nextSibling;
      while (isZwsOnlyText(n)) n = n.nextSibling;
      if (n && n.nodeType === 1 && n.hasAttribute && n.hasAttribute('data-ac-attach-id')) return n;
      return null;
    }
    if (node.nodeType === 1) {
      let i = offset;
      while (i < node.childNodes.length && isZwsOnlyText(node.childNodes[i])) i++;
      const n = node.childNodes[i];
      if (n && n.nodeType === 1 && n.hasAttribute && n.hasAttribute('data-ac-attach-id')) return n;
    }
    return null;
  };

  const placeCaretAfter = (node) => {
    if (!node) return;
    let next = node.nextSibling;
    if (!next || next.nodeType !== 3) {
      next = document.createTextNode('\u200b');
      if (node.parentNode) node.parentNode.insertBefore(next, node.nextSibling);
    }
    // 落到锚点文本末尾，避免停在 ZWS 中间造成「多按一次」
    placeCaretInText(next, (next.nodeValue || '').length);
  };

  const placeCaretBefore = (node) => {
    if (!node) return;
    let prev = node.previousSibling;
    if (!prev || prev.nodeType !== 3) {
      prev = document.createTextNode('\u200b');
      if (node.parentNode) node.parentNode.insertBefore(prev, node);
    }
    // 落到锚点文本开头，光标在芯片左侧，再按左键可继续在正文里移动
    placeCaretInText(prev, 0);
  };

  /** 保证 chip 两侧与相邻 chip 之间都有可落点的零宽文本节点。 */
  const ensureChipAnchors = (el) => {
    if (!el) return;
    const direct = [];
    for (let i = 0; i < el.childNodes.length; i++) {
      const n = el.childNodes[i];
      if (n.nodeType === 1 && n.hasAttribute && n.hasAttribute('data-ac-attach-id')) direct.push(n);
    }
    if (direct.length === 0) {
      if (!el.firstChild) el.appendChild(document.createTextNode(''));
      return;
    }
    if (!direct[0].previousSibling || direct[0].previousSibling.nodeType !== 3) {
      el.insertBefore(document.createTextNode('\u200b'), direct[0]);
    }
    for (let i = 0; i < direct.length; i++) {
      const chip = direct[i];
      const next = chip.nextSibling;
      if (!next) {
        el.appendChild(document.createTextNode('\u200b'));
      } else if (next.nodeType === 1 && next.hasAttribute && next.hasAttribute('data-ac-attach-id')) {
        el.insertBefore(document.createTextNode('\u200b'), next);
      } else if (next.nodeType !== 3) {
        el.insertBefore(document.createTextNode('\u200b'), next);
      }
    }
  };

  /** 把误嵌套进其它 chip 的 chip 提升为兄弟节点。 */
  const flattenNestedChips = (el) => {
    if (!el) return;
    let guard = 0;
    while (guard++ < 32) {
      const nested = el.querySelector('[data-ac-attach-id] [data-ac-attach-id]');
      if (!nested) break;
      let outer = nested.parentElement;
      while (outer && outer !== el && !(outer.hasAttribute && outer.hasAttribute('data-ac-attach-id'))) {
        outer = outer.parentElement;
      }
      if (!outer || outer === el) break;
      const zws = document.createTextNode('\u200b');
      if (outer.nextSibling) {
        outer.parentNode.insertBefore(nested, outer.nextSibling);
        outer.parentNode.insertBefore(zws, nested.nextSibling);
      } else {
        outer.parentNode.appendChild(nested);
        outer.parentNode.appendChild(zws);
      }
    }
  };

  const sanitizeCaret = (el) => {
    if (!el) return;
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0 || !sel.isCollapsed) return;
    if (!el.contains(sel.anchorNode)) return;
    const chip = closestChip(sel.anchorNode, el);
    if (!chip) return;
    // 选区落在 chip 内：挪到 chip 后的锚点文本
    placeCaretAfter(chip);
  };

  /** 各 composer 上次落在内部的选区；失焦后「添加到聊天」仍插到原 caret。挂 window 以免桥重装丢记录。 */
  if (!window.__acComposerLastCaret) window.__acComposerLastCaret = Object.create(null);
  const lastCaret = window.__acComposerLastCaret;

  const saveCaret = (el) => {
    if (!el) return;
    const id = el.getAttribute('data-ac-composer');
    if (!id) return;
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0 || !sel.anchorNode) return;
    if (sel.anchorNode !== el && !el.contains(sel.anchorNode)) return;
    try {
      lastCaret[id] = sel.getRangeAt(0).cloneRange();
    } catch (_) {}
  };

  const rangeForInsert = (el) => {
    if (!el) return null;
    const sel = window.getSelection();
    if (sel && sel.rangeCount > 0 && sel.anchorNode &&
        (sel.anchorNode === el || el.contains(sel.anchorNode))) {
      try { return sel.getRangeAt(0).cloneRange(); } catch (_) {}
    }
    const id = el.getAttribute('data-ac-composer');
    const saved = id ? lastCaret[id] : null;
    if (!saved) return null;
    try {
      const ca = saved.commonAncestorContainer;
      if (ca && (ca === el || el.contains(ca))) return saved.cloneRange();
    } catch (_) {}
    return null;
  };

  const insertChip = (id, html) => {
    const el = rootEl(id);
    if (!el) return false;
    // 先记下 caret：focus() 可能把选区挪走；失焦时用 lastCaret。
    const range = rangeForInsert(el);
    return withSuppress(() => {
      el.focus();
      const wrap = document.createElement('div');
      wrap.innerHTML = html;
      const chip = wrap.firstElementChild;
      if (!chip) return false;
      chip.setAttribute('contenteditable', 'false');
      chip.setAttribute('draggable', 'false');

      flattenNestedChips(el);
      ensureChipAnchors(el);

      let placed = false;
      if (range) {
        try {
          const sel = window.getSelection();
          if (sel) {
            sel.removeAllRanges();
            sel.addRange(range);
          }
          const live = (sel && sel.rangeCount > 0) ? sel.getRangeAt(0) : range;
          const inside = closestChip(live.startContainer, el);
          if (inside) {
            if (inside.nextSibling) inside.parentNode.insertBefore(chip, inside.nextSibling);
            else inside.parentNode.appendChild(chip);
          } else {
            live.collapse(true);
            live.insertNode(chip);
          }
          placed = !!chip.parentNode;
        } catch (_) {
          placed = false;
        }
      }
      if (!placed) {
        el.appendChild(chip);
      }

      flattenNestedChips(el);
      ensureChipAnchors(el);
      // 紧挨 chip 后插入 ZWS 锚点再落 caret，避免落到后续正文末尾。
      const zws = document.createTextNode('\u200b');
      if (chip.parentNode) chip.parentNode.insertBefore(zws, chip.nextSibling);
      placeCaretInText(zws, (zws.nodeValue || '').length);
      saveCaret(el);
      syncEmptyClass(el);
      return isSendable(el);
    });
  };

  const insertText = (id, text) => {
    const el = rootEl(id);
    if (!el || !text) return false;
    el.focus();
    return withSuppress(() => {
      let ok = false;
      try { ok = document.execCommand('insertText', false, text); } catch (_) {}
      if (!ok) {
        const sel = window.getSelection();
        if (sel && sel.rangeCount > 0 && el.contains(sel.anchorNode)) {
          const range = sel.getRangeAt(0);
          range.deleteContents();
          range.insertNode(document.createTextNode(text));
          range.collapse(false);
          sel.removeAllRanges();
          sel.addRange(range);
        } else {
          el.appendChild(document.createTextNode(text));
        }
      }
      syncEmptyClass(el);
      return isSendable(el);
    });
  };

  const clear = (id) => {
    const el = rootEl(id);
    if (!el) return;
    delete lastCaret[id];
    withSuppress(() => {
      el.innerHTML = '';
      syncEmptyClass(el);
    });
  };

  const setHtml = (id, html) => {
    const el = rootEl(id);
    if (!el) return false;
    delete lastCaret[id];
    return withSuppress(() => {
      el.innerHTML = html || '';
      flattenNestedChips(el);
      ensureChipAnchors(el);
      syncEmptyClass(el);
      return isSendable(el);
    });
  };

  const focusEnd = (id) => {
    const el = rootEl(id);
    if (!el) return;
    el.focus();
    ensureChipAnchors(el);
    const last = el.lastChild;
    if (last && last.nodeType === 3) {
      placeCaretInText(last, (last.nodeValue || '').length);
      return;
    }
    if (last && last.nodeType === 1 && last.hasAttribute && last.hasAttribute('data-ac-attach-id')) {
      placeCaretAfter(last);
      return;
    }
    const range = document.createRange();
    range.selectNodeContents(el);
    range.collapse(false);
    const sel = window.getSelection();
    if (!sel) return;
    sel.removeAllRanges();
    sel.addRange(range);
  };

  const focusStart = (id) => {
    const el = rootEl(id);
    if (!el) return;
    el.focus();
    ensureChipAnchors(el);
    const first = el.firstChild;
    if (first && first.nodeType === 3) {
      placeCaretInText(first, 0);
      return;
    }
    if (first && first.nodeType === 1 && first.hasAttribute && first.hasAttribute('data-ac-attach-id')) {
      placeCaretBefore(first);
    }
  };

  const collectRemovedIds = (node, out) => {
    if (!node) return;
    if (node.nodeType === 1) {
      if (node.hasAttribute && node.hasAttribute('data-ac-attach-id')) {
        const aid = Number(node.getAttribute('data-ac-attach-id'));
        if (!Number.isNaN(aid)) out.push(aid);
      }
      const kids = node.querySelectorAll ? node.querySelectorAll('[data-ac-attach-id]') : [];
      for (let i = 0; i < kids.length; i++) {
        const aid = Number(kids[i].getAttribute('data-ac-attach-id'));
        if (!Number.isNaN(aid)) out.push(aid);
      }
    }
  };

  const observeRoot = (el) => {
    if (!el || el.__acCeObserved) return;
    el.__acCeObserved = true;
    const obs = new MutationObserver((mutations) => {
      if (suppress > 0) return;
      const removed = [];
      for (const m of mutations) {
        for (const n of m.removedNodes) collectRemovedIds(n, removed);
      }
      if (removed.length === 0) {
        syncEmptyClass(el);
        return;
      }
      const root = el.getAttribute('data-ac-composer');
      emit({
        kind: 'remove',
        root,
        ids: removed,
        sendable: isSendable(el),
      });
      syncEmptyClass(el);
    });
    obs.observe(el, { childList: true, subtree: true });
    el.__acCeObserver = obs;
  };

  const ensureObserved = () => {
    document.querySelectorAll('[data-ac-composer]').forEach(observeRoot);
  };

  if (window.__acComposerDocBound !== 7) {
    window.__acComposerDocBound = 7;
    document.addEventListener('input', (e) => {
      const t = e.target;
      const el = t && t.closest ? t.closest('[data-ac-composer]') : null;
      if (!el) return;
      syncEmptyClass(el);
      emit({
        kind: 'input',
        root: el.getAttribute('data-ac-composer'),
        sendable: isSendable(el),
      });
    }, true);

    document.addEventListener('click', (e) => {
      const t = e.target;
      if (!t || !t.closest) return;
      const removeBtn = t.closest('[data-ac-chip-remove]');
      if (removeBtn) {
        const el = removeBtn.closest('[data-ac-composer]');
        if (!el) return;
        e.preventDefault();
        e.stopPropagation();
        const id = Number(removeBtn.getAttribute('data-ac-chip-remove'));
        const chip = el.querySelector('[data-ac-attach-id="' + id + '"]');
        withSuppress(() => { if (chip) chip.remove(); });
        ensureChipAnchors(el);
        emit({
          kind: 'remove',
          root: el.getAttribute('data-ac-composer'),
          ids: Number.isNaN(id) ? [] : [id],
          sendable: isSendable(el),
        });
        syncEmptyClass(el);
        return;
      }
      const chip = t.closest('[data-ac-attach-id]');
      if (chip) {
        const el = chip.closest('[data-ac-composer]');
        if (!el) return;
        // 点击 chip：激活或把光标放到 chip 后，绝不让 caret 进内部
        e.preventDefault();
        e.stopPropagation();
        if (chip.getAttribute('data-ac-activate') === '1') {
          emit({
            kind: 'activate',
            root: el.getAttribute('data-ac-composer'),
            id: Number(chip.getAttribute('data-ac-attach-id')),
          });
        }
        placeCaretAfter(chip);
        return;
      }
      const el = t.closest('[data-ac-composer]');
      if (el) {
        ensureChipAnchors(el);
        // 点在编辑器空白：若点在首 chip 左侧区域，落到前导锚点
        requestAnimationFrame(() => sanitizeCaret(el));
      }
    }, true);

    document.addEventListener('selectionchange', () => {
      if (suppress > 0) return;
      const sel = window.getSelection();
      if (!sel || !sel.anchorNode) return;
      const el = (sel.anchorNode.nodeType === 1 ? sel.anchorNode : sel.anchorNode.parentElement)?.closest?.('[data-ac-composer]');
      if (!el) return;
      sanitizeCaret(el);
      saveCaret(el);
    });

    document.addEventListener('focusout', (e) => {
      const t = e.target;
      const el = t && t.closest ? t.closest('[data-ac-composer]') : null;
      if (!el) return;
      saveCaret(el);
    }, true);

    document.addEventListener('mousedown', (e) => {
      const t = e.target;
      if (!t || !t.closest) return;
      if (t.closest('[data-ac-attach-id]') || t.closest('[data-ac-chip-remove]')) return;
      const el = t.closest('[data-ac-composer]');
      if (!el) return;
      ensureChipAnchors(el);
      const firstChip = Array.from(el.childNodes).find(
        (n) => n.nodeType === 1 && n.hasAttribute && n.hasAttribute('data-ac-attach-id')
      );
      if (!firstChip) return;
      const rect = firstChip.getBoundingClientRect();
      // 点在第一个 chip 左侧 → 落到前导锚点，允许在芯片前输入
      if (e.clientX < rect.left - 1) {
        e.preventDefault();
        el.focus();
        placeCaretBefore(firstChip);
      }
    }, true);

    document.addEventListener('keydown', (e) => {
      const el = e.target && e.target.closest ? e.target.closest('[data-ac-composer]') : null;
      if (!el) return;
      if (e.key === 'Home') {
        e.preventDefault();
        ensureChipAnchors(el);
        const first = el.firstChild;
        if (first && first.nodeType === 3) placeCaretInText(first, 0);
        else if (first) placeCaretBefore(first);
        return;
      }

      // 左右方向键：紧贴 chip 时一次跳过（不在中间 ZWS 上多停一拍）
      if ((e.key === 'ArrowLeft' || e.key === 'ArrowRight') && !e.shiftKey && !e.metaKey && !e.ctrlKey && !e.altKey) {
        ensureChipAnchors(el);
        const sel = window.getSelection();
        if (!sel || !sel.isCollapsed || !el.contains(sel.anchorNode)) return;

        const inside = closestChip(sel.anchorNode, el);
        if (inside) {
          e.preventDefault();
          if (e.key === 'ArrowLeft') placeCaretBefore(inside);
          else placeCaretAfter(inside);
          return;
        }

        if (e.key === 'ArrowLeft') {
          const chip = chipLeftOfCaret(sel.anchorNode, sel.anchorOffset);
          if (chip) {
            e.preventDefault();
            placeCaretBefore(chip);
            return;
          }
        }

        if (e.key === 'ArrowRight') {
          const chip = chipRightOfCaret(sel.anchorNode, sel.anchorOffset);
          if (chip) {
            e.preventDefault();
            placeCaretAfter(chip);
            return;
          }
        }
        return;
      }

      // Backspace / Delete：仅当光标紧贴 chip 右侧时删除该 chip
      if ((e.key === 'Backspace' || e.key === 'Delete') && !e.metaKey && !e.ctrlKey && !e.altKey) {
        ensureChipAnchors(el);
        const sel = window.getSelection();
        if (!sel || !sel.isCollapsed || !el.contains(sel.anchorNode)) return;
        if (closestChip(sel.anchorNode, el)) {
          sanitizeCaret(el);
          return;
        }

        const chip = chipLeftOfCaret(sel.anchorNode, sel.anchorOffset);
        if (!chip) return;

        e.preventDefault();
        const id = Number(chip.getAttribute('data-ac-attach-id'));
        const after = chip.nextSibling;
        withSuppress(() => chip.remove());
        ensureChipAnchors(el);
        if (after && el.contains(after)) {
          if (after.nodeType === 3) placeCaretInText(after, 0);
          else placeCaretBefore(after);
        } else {
          focusEnd(el.getAttribute('data-ac-composer'));
        }
        emit({
          kind: 'remove',
          root: el.getAttribute('data-ac-composer'),
          ids: [id],
          sendable: isSendable(el),
        });
        syncEmptyClass(el);
        return;
      }

      if (e.key === 'End') {
        requestAnimationFrame(() => sanitizeCaret(el));
      }
    }, true);

    const mo = new MutationObserver(() => ensureObserved());
    mo.observe(document.documentElement, { childList: true, subtree: true });
    ensureObserved();
  }

  if (!window.__acImageLightboxBound) {
    window.__acImageLightboxBound = 1;
    const LB = 'ac-chat-image-lightbox';
    const closeImageLightbox = () => {
      const el = document.querySelector('.' + LB);
      if (el) el.remove();
    };
    const openImageLightbox = (src) => {
      if (!src) return;
      closeImageLightbox();
      const overlay = document.createElement('div');
      overlay.className = LB;
      overlay.setAttribute('role', 'dialog');
      overlay.setAttribute('aria-modal', 'true');
      overlay.setAttribute('aria-label', '图片预览');
      const img = document.createElement('img');
      img.src = src;
      img.alt = '';
      overlay.appendChild(img);
      overlay.addEventListener('click', (ev) => {
        ev.preventDefault();
        ev.stopPropagation();
        closeImageLightbox();
      });
      document.body.appendChild(overlay);
    };
    window.__acOpenImageLightbox = openImageLightbox;
    document.addEventListener('dblclick', (e) => {
      const t = e.target;
      if (!t || !t.closest) return;
      if (t.closest('[data-ac-chip-remove]')) return;
      const thumb = t.closest('.ac-chat-attach-thumb');
      if (!thumb) return;
      const img = thumb.querySelector('.ac-chat-attach-thumb-img');
      if (!img) return;
      e.preventDefault();
      e.stopPropagation();
      openImageLightbox(img.getAttribute('src') || img.src);
    }, true);
    document.addEventListener('keydown', (e) => {
      if (e.key !== 'Escape') return;
      if (!document.querySelector('.' + LB)) return;
      e.preventDefault();
      e.stopPropagation();
      closeImageLightbox();
    }, true);
  }

  window.__acComposer = {
    serialize,
    insertChip,
    insertText,
    clear,
    setHtml,
    focusEnd,
    focusStart,
    isSendable: (id) => isSendable(rootEl(id)),
    ensureObserved,
    ensureChipAnchors: (id) => ensureChipAnchors(rootEl(id)),
    esc,
    FOLDER_SVG,
    FILE_SVG,
    X_SVG,
  };

  await new Promise(() => {});
})()"#;

#[derive(Debug, Clone, Deserialize)]
pub struct ComposerBridgeEvent {
    pub kind: String,
    pub root: Option<String>,
    #[serde(default)]
    pub sendable: Option<bool>,
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub ids: Vec<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposerDomSnapshot {
    pub text: String,
    #[serde(default)]
    pub attach_ids: Vec<u64>,
    #[serde(default)]
    pub parts: Vec<ComposerDomPart>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposerDomPart {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub id: u64,
}

/// 按输入区 DOM 顺序生成发给模型的正文、附件列表与气泡片段。
pub fn compose_user_payload(
    snap: &ComposerDomSnapshot,
    registry: &[ChatPendingAttachment],
) -> (
    String,
    Vec<ChatPendingAttachment>,
    Vec<super::attachments::ChatUserSeg>,
) {
    use super::attachments::ChatUserSeg;

    if snap.parts.is_empty() {
        let pending = pending_resolve(registry, &snap.attach_ids);
        let mut send_text = snap.text.trim().to_string();
        let non_image: Vec<String> = pending
            .iter()
            .filter(|a| a.image_data_url.is_none())
            .map(|a| a.send_label().to_string())
            .collect();
        if !non_image.is_empty() {
            let names = non_image.join(", ");
            if send_text.is_empty() {
                send_text = format!("[附件: {names}]");
            } else {
                send_text.push_str(&format!("\n\n[附件: {names}]"));
            }
        }
        let segs = super::attachments::parse_user_message_segments(&send_text, &pending);
        return (send_text, pending, segs);
    }

    let mut send_text = String::new();
    let mut pending = Vec::new();
    let mut segs = Vec::new();
    for part in &snap.parts {
        if part.kind == "attach" {
            let Some(att) = registry.iter().find(|a| a.id == part.id).cloned() else {
                continue;
            };
            pending.push(att.clone());
            segs.push(ChatUserSeg::Attachment(att.clone()));
            if att.image_data_url.is_none() {
                send_text.push_str("[附件: ");
                send_text.push_str(att.send_label());
                send_text.push(']');
            }
        } else {
            let t = part.text.replace('\u{200b}', "");
            if t.is_empty() {
                continue;
            }
            send_text.push_str(&t);
            segs.push(ChatUserSeg::Text(t));
        }
    }
    (send_text.trim().to_string(), pending, segs)
}

fn js_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

/// 生成 chip HTML（插入 contenteditable）。
pub fn chip_html(att: &ChatPendingAttachment) -> String {
    let id = att.id;
    let name = html_escape(&att.name);
    let title = html_escape(
        att.source_path
            .as_deref()
            .filter(|p| !p.trim().is_empty())
            .unwrap_or(att.name.as_str()),
    );
    if let Some(url) = att.preview_url.as_deref() {
        let url = html_escape(url);
        return format!(
            concat!(
                r#"<span class="ac-chat-attach-thumb" contenteditable="false" "#,
                r#"data-ac-attach-id="{id}" title="{title}">"#,
                r#"<img class="ac-chat-attach-thumb-img" src="{url}" alt="{name}" draggable="false" />"#,
                r#"<button type="button" tabindex="-1" class="ac-chat-attach-thumb-remove" "#,
                r#"data-ac-chip-remove="{id}" title="移除附件" aria-label="移除附件">"#,
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>"#,
                r#"</button></span>"#
            ),
            id = id,
            title = title,
            url = url,
            name = name,
        );
    }
    let is_dir = att.is_dir;
    let activatable = att.source_path.is_some();
    let mut class = if is_dir {
        "ac-chat-attach-chip is-dir".to_string()
    } else {
        "ac-chat-attach-chip is-file".to_string()
    };
    if activatable {
        class.push_str(" is-activatable");
    }
    let activate = if activatable { "1" } else { "0" };
    let icon = if is_dir {
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>"#
        )
    } else {
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/></svg>"#
        )
    };
    format!(
        concat!(
            r#"<span class="{class}" contenteditable="false" data-ac-attach-id="{id}" "#,
            r#"data-ac-activate="{activate}" title="{title}">"#,
            r#"<span class="ac-chat-attach-chip-icon">"#,
            r#"<span class="ac-chat-attach-chip-type-icon">{icon}</span>"#,
            r#"<button type="button" tabindex="-1" class="ac-chat-attach-chip-remove" "#,
            r#"data-ac-chip-remove="{id}" title="移除附件" aria-label="移除附件">{x}</button>"#,
            r#"</span>"#,
            r#"<span class="ac-chat-attach-chip-name">{name}</span>"#,
            r#"</span>"#
        ),
        class = class,
        id = id,
        activate = activate,
        title = title,
        icon = icon,
        name = name,
        x = concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>"#
        ),
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 编辑恢复：按片段顺序插入 chip 与正文。
pub fn composer_seed_html_from_segs(segs: &[super::attachments::ChatUserSeg]) -> String {
    use super::attachments::ChatUserSeg;
    let mut out = String::new();
    for seg in segs {
        match seg {
            ChatUserSeg::Attachment(att) => {
                out.push_str(&chip_html(att));
                out.push('\u{200b}');
            }
            ChatUserSeg::Text(text) => {
                let visible = super::attachments::strip_attachment_suffix(text);
                if !visible.is_empty() {
                    out.push_str(&html_escape(&visible).replace('\n', "<br>"));
                }
            }
        }
    }
    out
}

/// 编辑恢复：芯片与正文按 `[附件: …]` 在原文中的位置交错。
pub fn composer_seed_html(text: &str, attachments: &[ChatPendingAttachment]) -> String {
    let segs = super::attachments::parse_user_message_segments(text, attachments);
    if segs.is_empty() {
        let visible = super::attachments::strip_attachment_suffix(text);
        if visible.is_empty() {
            String::new()
        } else {
            html_escape(&visible).replace('\n', "<br>")
        }
    } else {
        composer_seed_html_from_segs(&segs)
    }
}

pub fn pending_has_path(pending: &[ChatPendingAttachment], path: &str) -> bool {
    pending
        .iter()
        .any(|att| att.source_path.as_deref() == Some(path))
}

pub fn pending_resolve(
    pending: &[ChatPendingAttachment],
    ids: &[u64],
) -> Vec<ChatPendingAttachment> {
    ids.iter()
        .filter_map(|id| pending.iter().find(|a| a.id == *id).cloned())
        .collect()
}

pub fn pending_remove_ids(pending: &mut Vec<ChatPendingAttachment>, ids: &[u64]) {
    if ids.is_empty() {
        return;
    }
    pending.retain(|a| !ids.contains(&a.id));
}

pub fn ce_insert_chip(root: &str, html: &str) {
    let script = format!(
        "window.__acComposer && window.__acComposer.insertChip({}, {});",
        js_str(root),
        js_str(html)
    );
    let _ = document::eval(&script);
}

pub fn ce_insert_text(root: &str, text: &str) {
    let script = format!(
        "window.__acComposer && window.__acComposer.insertText({}, {});",
        js_str(root),
        js_str(text)
    );
    let _ = document::eval(&script);
}

pub fn ce_clear(root: &str) {
    let script = format!(
        "window.__acComposer && window.__acComposer.clear({});",
        js_str(root)
    );
    let _ = document::eval(&script);
}

pub fn ce_set_html(root: &str, html: &str) {
    let script = format!(
        "window.__acComposer && window.__acComposer.setHtml({}, {});",
        js_str(root),
        js_str(html)
    );
    let _ = document::eval(&script);
}

pub fn ce_focus_end(root: &str) {
    let script = format!(
        "window.__acComposer && window.__acComposer.focusEnd({});",
        js_str(root)
    );
    let _ = document::eval(&script);
}

pub async fn ce_serialize(root: &str) -> ComposerDomSnapshot {
    let script = format!(
        "return (window.__acComposer && window.__acComposer.serialize({})) || {{ text: '', attach_ids: [], parts: [] }};",
        js_str(root)
    );
    match document::eval(&script).join::<ComposerDomSnapshot>().await {
        Ok(snap) => snap,
        Err(_) => ComposerDomSnapshot::default(),
    }
}

#[allow(dead_code)]
pub async fn ce_is_sendable(root: &str) -> bool {
    let script = format!(
        "return !!(window.__acComposer && window.__acComposer.isSendable({}));",
        js_str(root)
    );
    document::eval(&script)
        .join::<bool>()
        .await
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{compose_user_payload, ComposerDomPart, ComposerDomSnapshot};
    use crate::chat::attachments::{attachment_from_path, ChatUserSeg};

    #[test]
    fn payload_keeps_dom_chip_order_even_if_registry_differs() {
        let registry = vec![
            attachment_from_path(1, "/x/app"),
            attachment_from_path(2, "/x/desktop"),
        ];
        let snap = ComposerDomSnapshot {
            text: "需要参考".into(),
            attach_ids: vec![1, 2],
            parts: vec![
                ComposerDomPart {
                    kind: "attach".into(),
                    text: String::new(),
                    id: 2,
                },
                ComposerDomPart {
                    kind: "attach".into(),
                    text: String::new(),
                    id: 1,
                },
                ComposerDomPart {
                    kind: "text".into(),
                    text: "需要参考".into(),
                    id: 0,
                },
            ],
        };
        let (send, pending, segs) = compose_user_payload(&snap, &registry);
        assert_eq!(
            pending.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            vec!["desktop", "app"]
        );
        assert_eq!(send, "[附件: /x/desktop][附件: /x/app]需要参考");
        match &segs[..] {
            [ChatUserSeg::Attachment(a), ChatUserSeg::Attachment(b), ChatUserSeg::Text(t)] => {
                assert_eq!(a.name, "desktop");
                assert_eq!(b.name, "app");
                assert_eq!(t, "需要参考");
            }
            other => panic!("unexpected segs: {other:?}"),
        }
    }

    #[test]
    fn payload_keeps_text_between_chips() {
        let registry = vec![
            attachment_from_path(1, "/x/app"),
            attachment_from_path(2, "/x/desktop"),
        ];
        let snap = ComposerDomSnapshot {
            text: String::new(),
            attach_ids: vec![1, 2],
            parts: vec![
                ComposerDomPart {
                    kind: "attach".into(),
                    text: String::new(),
                    id: 1,
                },
                ComposerDomPart {
                    kind: "text".into(),
                    text: "参考".into(),
                    id: 0,
                },
                ComposerDomPart {
                    kind: "attach".into(),
                    text: String::new(),
                    id: 2,
                },
            ],
        };
        let (send, _, segs) = compose_user_payload(&snap, &registry);
        assert_eq!(send, "[附件: /x/app]参考[附件: /x/desktop]");
        match &segs[..] {
            [ChatUserSeg::Attachment(a), ChatUserSeg::Text(t), ChatUserSeg::Attachment(b)] => {
                assert_eq!(a.name, "app");
                assert_eq!(t, "参考");
                assert_eq!(b.name, "desktop");
            }
            other => panic!("unexpected segs: {other:?}"),
        }
    }
}
