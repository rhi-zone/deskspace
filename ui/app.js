/* deskspace – tiling file browser */

import { marked } from 'marked';
import hljs from 'highlight.js/lib/common';

// ─── State ───────────────────────────────────────────────

let tileTree = { type: 'pane', id: genId(), path: '', projection: null };
let panes = new Map(); // id -> DOM element

function genId() {
  return Math.random().toString(36).slice(2, 9);
}

// ─── API ─────────────────────────────────────────────────

async function fetchResource(path, projection) {
  const base = path ? `/api/files/${encodeURI(path)}` : '/api/files/';
  const url = projection ? `${base}?projection=${encodeURIComponent(projection)}` : base;
  const res = await fetch(url);
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error || res.statusText);
  }
  return res.json();
}

// ─── Tile tree operations ────────────────────────────────

function findNode(tree, id) {
  if (tree.id === id) return tree;
  if (tree.type === 'split') {
    for (const child of tree.children) {
      const found = findNode(child, id);
      if (found) return found;
    }
  }
  return null;
}

function findParent(tree, id) {
  if (tree.type !== 'split') return null;
  for (let i = 0; i < tree.children.length; i++) {
    if (tree.children[i].id === id) return { parent: tree, index: i };
    const found = findParent(tree.children[i], id);
    if (found) return found;
  }
  return null;
}

function splitPane(paneId, direction) {
  const node = findNode(tileTree, paneId);
  if (!node || node.type !== 'pane') return;

  const newPaneId = genId();
  const splitId = genId();

  const original = { ...node };
  // Mutate in place
  node.type = 'split';
  node.id = splitId;
  node.direction = direction;
  node.children = [
    { type: 'pane', id: original.id, path: original.path, projection: original.projection },
    { type: 'pane', id: newPaneId, path: original.path, projection: null },
  ];
  node.sizes = [50, 50];
  delete node.path;
  delete node.projection;

  render();
}

function closePane(paneId) {
  // Can't close the last pane
  if (tileTree.type === 'pane') return;

  const result = findParent(tileTree, paneId);
  if (!result) return;
  const { parent, index } = result;

  const sibling = parent.children[1 - index];

  // Replace parent with sibling
  Object.keys(parent).forEach(k => delete parent[k]);
  Object.assign(parent, sibling);

  render();
}

function navigatePane(paneId, path) {
  const node = findNode(tileTree, paneId);
  if (!node || node.type !== 'pane') return;
  node.path = path;
  node.projection = null;
  renderPane(paneId);
}

function switchProjection(paneId, projectionId) {
  const node = findNode(tileTree, paneId);
  if (!node || node.type !== 'pane') return;
  node.projection = projectionId;
  renderPane(paneId);
}

// ─── Rendering: tile tree → DOM ──────────────────────────

function render() {
  const root = document.getElementById('root');
  root.innerHTML = '';
  panes.clear();
  root.appendChild(buildDom(tileTree));
}

function buildDom(node) {
  if (node.type === 'pane') return buildPaneDom(node);
  return buildSplitDom(node);
}

function buildSplitDom(node) {
  const el = document.createElement('div');
  el.className = `split ${node.direction}`;

  for (let i = 0; i < node.children.length; i++) {
    if (i > 0) {
      const handle = document.createElement('div');
      handle.className = 'split-handle';
      handle.addEventListener('mousedown', (e) => startResize(e, node, i - 1, el));
      el.appendChild(handle);
    }

    const child = buildDom(node.children[i]);
    const sizeProp = node.direction === 'horizontal' ? 'width' : 'height';
    child.style.flex = `0 0 ${node.sizes[i]}%`;
    el.appendChild(child);
  }

  return el;
}

function buildPaneDom(node) {
  const el = document.createElement('div');
  el.className = 'pane';
  el.dataset.paneId = node.id;

  // Header
  const header = document.createElement('div');
  header.className = 'pane-header';

  const breadcrumb = document.createElement('div');
  breadcrumb.className = 'pane-breadcrumb';
  header.appendChild(breadcrumb);

  const projBar = document.createElement('div');
  projBar.className = 'pane-projections';
  header.appendChild(projBar);

  // Split buttons
  const splitH = document.createElement('button');
  splitH.className = 'pane-action';
  splitH.textContent = '│';
  splitH.title = 'Split horizontally (Ctrl+\\)';
  splitH.onclick = () => splitPane(node.id, 'horizontal');
  header.appendChild(splitH);

  const splitV = document.createElement('button');
  splitV.className = 'pane-action';
  splitV.textContent = '─';
  splitV.title = 'Split vertically (Ctrl+Shift+\\)';
  splitV.onclick = () => splitPane(node.id, 'vertical');
  header.appendChild(splitV);

  const closeBtn = document.createElement('button');
  closeBtn.className = 'pane-action';
  closeBtn.textContent = '×';
  closeBtn.title = 'Close pane';
  closeBtn.onclick = () => closePane(node.id);
  header.appendChild(closeBtn);

  el.appendChild(header);

  const content = document.createElement('div');
  content.className = 'pane-content';
  el.appendChild(content);

  panes.set(node.id, el);
  renderPane(node.id);

  return el;
}

// ─── Rendering: pane content ─────────────────────────────

async function renderPane(paneId) {
  const node = findNode(tileTree, paneId);
  if (!node || node.type !== 'pane') return;

  const el = panes.get(paneId);
  if (!el) return;

  const content = el.querySelector('.pane-content');
  const breadcrumb = el.querySelector('.pane-breadcrumb');
  const projBar = el.querySelector('.pane-projections');

  content.innerHTML = '<div class="loading">Loading…</div>';
  breadcrumb.textContent = '';
  projBar.innerHTML = '';

  try {
    const data = await fetchResource(node.path, node.projection);

    // Breadcrumb
    buildBreadcrumb(breadcrumb, paneId, data.path);

    // Projection buttons
    for (const proj of data.projections) {
      const btn = document.createElement('button');
      btn.className = 'pane-proj-btn';
      if (proj.id === data.active_projection) btn.classList.add('active');
      btn.textContent = proj.name;
      btn.onclick = () => switchProjection(paneId, proj.id);
      projBar.appendChild(btn);
    }

    // Content
    content.innerHTML = '';
    renderOutput(content, data.output, paneId);
  } catch (err) {
    content.innerHTML = `<div class="error-msg">${escHtml(err.message)}</div>`;
  }
}

function buildBreadcrumb(container, paneId, path) {
  const parts = path ? path.split('/') : [];

  const rootLink = document.createElement('a');
  rootLink.href = '#';
  rootLink.textContent = '~';
  rootLink.onclick = (e) => { e.preventDefault(); navigatePane(paneId, ''); };
  container.appendChild(rootLink);

  let accumulated = '';
  for (const part of parts) {
    container.appendChild(document.createTextNode(' / '));
    accumulated += (accumulated ? '/' : '') + part;
    const link = document.createElement('a');
    link.href = '#';
    link.textContent = part;
    const navPath = accumulated;
    link.onclick = (e) => { e.preventDefault(); navigatePane(paneId, navPath); };
    container.appendChild(link);
  }
}

// ─── Output renderers ────────────────────────────────────

function renderOutput(container, output, paneId) {
  switch (output.type) {
    case 'DirectoryList': return renderDirList(container, output, paneId);
    case 'Text': return renderText(container, output);
    case 'Markdown': return renderMarkdown(container, output);
    case 'Image': return renderImage(container, output);
    default:
      container.innerHTML = `<div class="error-msg">Unknown output type: ${escHtml(output.type)}</div>`;
  }
}

function renderDirList(container, output, paneId) {
  const grid = document.createElement('div');
  grid.className = 'dir-list';

  const currentNode = findNode(tileTree, paneId);
  const basePath = currentNode ? currentNode.path : '';

  for (const entry of output.entries) {
    const item = document.createElement('a');
    item.className = 'dir-entry' + (entry.is_dir ? ' is-dir' : '');
    item.href = '#';
    item.onclick = (e) => {
      e.preventDefault();
      const entryPath = basePath ? `${basePath}/${entry.name}` : entry.name;
      navigatePane(paneId, entryPath);
    };

    const icon = document.createElement('span');
    icon.className = 'dir-entry-icon';
    icon.textContent = entry.is_dir ? '📁' : fileIcon(entry.extension);
    item.appendChild(icon);

    const name = document.createElement('span');
    name.className = 'dir-entry-name';
    name.textContent = entry.name;
    item.appendChild(name);

    if (!entry.is_dir) {
      const size = document.createElement('span');
      size.className = 'dir-entry-size';
      size.textContent = formatSize(entry.size);
      item.appendChild(size);
    }

    grid.appendChild(item);
  }

  container.appendChild(grid);
}

function renderText(container, output) {
  const wrapper = document.createElement('div');
  wrapper.className = 'text-content';
  const pre = document.createElement('pre');
  const code = document.createElement('code');

  if (output.language) {
    code.className = `language-${output.language}`;
  }
  code.textContent = output.content;

  if (output.language) {
    try {
      const result = hljs.highlight(output.content, { language: output.language, ignoreIllegals: true });
      code.innerHTML = result.value;
    } catch (_) {
      // fallback to plain text
    }
  }

  pre.appendChild(code);
  wrapper.appendChild(pre);
  container.appendChild(wrapper);
}

function renderMarkdown(container, output) {
  const view = document.createElement('div');
  view.className = 'markdown-view';

  const body = document.createElement('div');
  body.className = 'markdown-body';

  marked.setOptions({
    highlight: function(code, lang) {
      if (lang) {
        try {
          return hljs.highlight(code, { language: lang, ignoreIllegals: true }).value;
        } catch (_) {}
      }
      return code;
    }
  });
  body.innerHTML = marked.parse(output.raw);

  view.appendChild(body);
  container.appendChild(view);

  // Floating TOC
  if (output.toc && output.toc.length > 0) {
    const toc = document.createElement('nav');
    toc.className = 'toc-float';

    const title = document.createElement('div');
    title.className = 'toc-float-title';
    title.textContent = 'Contents';
    toc.appendChild(title);

    for (const entry of output.toc) {
      const link = document.createElement('a');
      link.href = `#${entry.slug}`;
      link.className = `toc-h${entry.level}`;
      link.textContent = entry.text;
      link.onclick = (e) => {
        e.preventDefault();
        const target = body.querySelector(`#${CSS.escape(entry.slug)}`);
        if (target) target.scrollIntoView({ behavior: 'smooth' });
      };
      toc.appendChild(link);
    }

    container.appendChild(toc);
  }

  // Add IDs to headings for TOC links
  body.querySelectorAll('h1, h2, h3, h4, h5, h6').forEach(el => {
    const slug = el.textContent.trim().toLowerCase()
      .replace(/[^\w\s-]/g, '')
      .replace(/\s+/g, '-');
    el.id = slug;
  });
}

function renderImage(container, output) {
  const wrapper = document.createElement('div');
  wrapper.className = 'image-preview';
  const img = document.createElement('img');
  img.src = output.url;
  img.alt = 'Preview';
  wrapper.appendChild(img);
  container.appendChild(wrapper);
}

// ─── Drag resize ─────────────────────────────────────────

function startResize(e, splitNode, handleIndex, splitEl) {
  e.preventDefault();
  const handle = splitEl.querySelectorAll('.split-handle')[handleIndex];
  handle.classList.add('dragging');

  const isHorizontal = splitNode.direction === 'horizontal';
  const startPos = isHorizontal ? e.clientX : e.clientY;
  const totalSize = isHorizontal ? splitEl.offsetWidth : splitEl.offsetHeight;
  const startSizes = [...splitNode.sizes];

  function onMove(e) {
    const delta = (isHorizontal ? e.clientX : e.clientY) - startPos;
    const deltaPercent = (delta / totalSize) * 100;

    const newA = Math.max(10, startSizes[handleIndex] + deltaPercent);
    const newB = Math.max(10, startSizes[handleIndex + 1] - deltaPercent);

    splitNode.sizes[handleIndex] = newA;
    splitNode.sizes[handleIndex + 1] = newB;

    // Update flex directly without full re-render
    const children = Array.from(splitEl.children).filter(c => !c.classList.contains('split-handle'));
    if (children[handleIndex]) children[handleIndex].style.flex = `0 0 ${newA}%`;
    if (children[handleIndex + 1]) children[handleIndex + 1].style.flex = `0 0 ${newB}%`;
  }

  function onUp() {
    handle.classList.remove('dragging');
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
  }

  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', onUp);
}

// ─── Keyboard shortcuts ──────────────────────────────────

document.addEventListener('keydown', (e) => {
  if (e.key === '\\' && e.ctrlKey && !e.shiftKey) {
    e.preventDefault();
    const activePaneId = getActivePaneId();
    if (activePaneId) splitPane(activePaneId, 'horizontal');
  }
  if (e.key === '\\' && e.ctrlKey && e.shiftKey) {
    e.preventDefault();
    const activePaneId = getActivePaneId();
    if (activePaneId) splitPane(activePaneId, 'vertical');
  }
});

function getActivePaneId() {
  // Use the first pane, or the pane that has focus
  const focused = document.activeElement?.closest('.pane');
  if (focused) return focused.dataset.paneId;
  const first = document.querySelector('.pane');
  return first ? first.dataset.paneId : null;
}

// ─── Helpers ─────────────────────────────────────────────

function escHtml(s) {
  const div = document.createElement('div');
  div.textContent = s;
  return div.innerHTML;
}

function formatSize(bytes) {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const val = bytes / Math.pow(1024, i);
  return `${val < 10 ? val.toFixed(1) : Math.round(val)} ${units[i]}`;
}

function fileIcon(ext) {
  if (!ext) return '📄';
  const icons = {
    md: '📝', markdown: '📝',
    rs: '🦀', toml: '⚙️',
    js: '📜', ts: '📜', jsx: '📜', tsx: '📜',
    py: '🐍',
    json: '📋', yaml: '📋', yml: '📋',
    html: '🌐', css: '🎨',
    png: '🖼️', jpg: '🖼️', jpeg: '🖼️', gif: '🖼️', webp: '🖼️', svg: '🖼️',
    sh: '🔧', bash: '🔧',
  };
  return icons[ext] || '📄';
}

// ─── Boot ────────────────────────────────────────────────

render();
