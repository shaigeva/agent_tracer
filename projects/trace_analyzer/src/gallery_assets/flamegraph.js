/**
 * Minimal SVG flame graph renderer - pure vanilla JS, no dependencies.
 *
 * Usage:
 *   import { renderFlameGraph } from './flamegraph.js';
 *   renderFlameGraph(container, events, { height: 400 });
 *
 * Input: events = array of { event, file, function, line, depth, timestamp_ns }
 *   produced by pytest-tracer (or any equivalent CALL/RETURN event stream).
 *
 * Features:
 *   - Click to zoom into a subtree
 *   - Right-click to zoom out one level
 *   - Hover for tooltip with full function name and file path
 *   - Search/highlight matching frames
 *   - Deterministic color per frame (hash-based, orange/red palette like classic flame graphs)
 */
(function (root) {
  'use strict';

  const ROW_HEIGHT = 18;
  const MIN_BAR_WIDTH = 0.3; // pixels; smaller frames won't be rendered

  /**
   * Build a tree of frames from a stream of CALL/RETURN events.
   * Returns the root node. Each node has:
   *   { name, file, function, children: [], value: number, depth: number, start: number, end: number }
   */
  function buildTree(events) {
    const root = { name: '<root>', file: '', function: '<root>', children: [], value: 0, depth: -1, start: 0, end: 0 };
    const stack = [root];
    let counter = 0;

    for (const e of events) {
      if (e.event === 'call') {
        const module = fileStem(e.file);
        const name = module + '.' + e.function;
        const node = {
          name: name,
          file: e.file,
          function: e.function,
          line: e.line,
          children: [],
          value: 0,
          depth: stack.length - 1,
          start: counter++,
          end: -1,
        };
        stack[stack.length - 1].children.push(node);
        stack.push(node);
      } else if (e.event === 'return') {
        const node = stack.pop();
        if (node && node !== root) {
          node.end = counter++;
          node.value = Math.max(1, node.end - node.start);
        }
      }
    }

    // Close any unclosed stacks (e.g., due to exceptions)
    while (stack.length > 1) {
      const node = stack.pop();
      node.end = counter++;
      node.value = Math.max(1, node.end - node.start);
    }

    // Compute values for all nodes: sum of own duration or children
    computeValues(root);
    return root;
  }

  function computeValues(node) {
    if (node.children.length === 0) {
      if (node.value === 0) node.value = 1;
      return node.value;
    }
    let sum = 0;
    for (const child of node.children) {
      sum += computeValues(child);
    }
    if (node.value === 0) {
      node.value = sum;
    } else {
      node.value = Math.max(node.value, sum);
    }
    return node.value;
  }

  function fileStem(path) {
    const slash = path.lastIndexOf('/');
    const filename = slash >= 0 ? path.substring(slash + 1) : path;
    const dot = filename.lastIndexOf('.');
    return dot >= 0 ? filename.substring(0, dot) : filename;
  }

  /**
   * Deterministic hash-based color for a frame name.
   * Uses classic flame graph palette (warm orange-red).
   */
  function colorFor(name) {
    let hash = 0;
    for (let i = 0; i < name.length; i++) {
      hash = ((hash << 5) - hash) + name.charCodeAt(i);
      hash |= 0;
    }
    const r = 205 + (Math.abs(hash) % 50);
    const g = 40 + (Math.abs(hash >> 8) % 120);
    const b = 20 + (Math.abs(hash >> 16) % 40);
    return 'rgb(' + r + ',' + g + ',' + b + ')';
  }

  /**
   * Render a flame graph into the given container element.
   * Returns an object with a `setSearch(query)` method for highlighting.
   */
  function renderFlameGraph(container, events, options) {
    options = options || {};
    container.innerHTML = '';
    container.style.position = 'relative';

    if (!events || events.length === 0) {
      container.textContent = 'No call trace data';
      return { setSearch: function () {} };
    }

    const root = buildTree(events);

    // Controls
    const controls = document.createElement('div');
    controls.style.cssText = 'display:flex; gap:0.5rem; align-items:center; margin-bottom:0.5rem; font-size:0.85rem;';
    const resetBtn = document.createElement('button');
    resetBtn.textContent = 'Reset zoom';
    resetBtn.style.cssText = 'padding:4px 10px; border-radius:4px; border:1px solid #ccc; background:white; cursor:pointer;';
    const searchInput = document.createElement('input');
    searchInput.type = 'text';
    searchInput.placeholder = 'Highlight frames matching...';
    searchInput.style.cssText = 'flex:1; padding:4px 8px; border-radius:4px; border:1px solid #ccc;';
    const info = document.createElement('span');
    info.style.cssText = 'color:#6e6e73; font-size:0.8rem;';
    info.textContent = events.length + ' events';
    controls.appendChild(resetBtn);
    controls.appendChild(searchInput);
    controls.appendChild(info);
    container.appendChild(controls);

    // SVG canvas
    const svgNS = 'http://www.w3.org/2000/svg';
    const maxDepth = computeMaxDepth(root);
    const viewW = 1200;
    const viewH = Math.max(100, (maxDepth + 1) * ROW_HEIGHT + 20);

    const svg = document.createElementNS(svgNS, 'svg');
    svg.setAttribute('viewBox', '0 0 ' + viewW + ' ' + viewH);
    svg.setAttribute('preserveAspectRatio', 'none');
    svg.style.cssText = 'width:100%; height:' + viewH + 'px; border:1px solid #e0e0e0; border-radius:6px; background:#fafafa; font-family: ui-monospace, Menlo, monospace;';
    container.appendChild(svg);

    // Tooltip
    const tooltip = document.createElement('div');
    tooltip.style.cssText = 'position:absolute; background:rgba(0,0,0,0.85); color:white; padding:6px 10px; border-radius:4px; font-size:0.8rem; font-family:ui-monospace,Menlo,monospace; pointer-events:none; display:none; z-index:100; max-width:500px; word-break:break-all;';
    container.appendChild(tooltip);

    let currentRoot = root;
    let searchQuery = '';

    function render(rootNode) {
      // Clear SVG
      while (svg.firstChild) svg.removeChild(svg.firstChild);

      const total = rootNode.value || 1;
      const baseDepth = rootNode.depth + 1;

      function drawNode(node, x, width) {
        if (width < MIN_BAR_WIDTH) return;

        if (node !== rootNode || node === root) {
          const y = (node.depth - baseDepth + (rootNode === root ? 0 : 0)) * ROW_HEIGHT;
          if (y >= 0 && node !== root) {
            const rect = document.createElementNS(svgNS, 'rect');
            rect.setAttribute('x', x);
            rect.setAttribute('y', y);
            rect.setAttribute('width', width);
            rect.setAttribute('height', ROW_HEIGHT - 1);
            const matches = searchQuery && node.name.toLowerCase().includes(searchQuery.toLowerCase());
            rect.setAttribute('fill', matches ? '#ffd700' : colorFor(node.name));
            rect.setAttribute('stroke', matches ? '#b8860b' : 'rgba(0,0,0,0.1)');
            rect.style.cursor = 'pointer';
            svg.appendChild(rect);

            // Text label (only if width is enough)
            if (width > 20) {
              const text = document.createElementNS(svgNS, 'text');
              text.setAttribute('x', x + 3);
              text.setAttribute('y', y + ROW_HEIGHT - 5);
              text.setAttribute('font-size', '11');
              text.setAttribute('fill', matches ? 'black' : 'white');
              text.setAttribute('pointer-events', 'none');
              // Truncate text to fit (rough estimate: 7px per char)
              const maxChars = Math.floor((width - 6) / 7);
              text.textContent = node.name.length > maxChars
                ? node.name.substring(0, maxChars - 1) + '\u2026'
                : node.name;
              svg.appendChild(text);
            }

            // Interactions
            rect.addEventListener('mouseenter', function (evt) {
              const rectBounds = container.getBoundingClientRect();
              const pctValue = total > 0 ? ((node.value / total) * 100).toFixed(1) : '0';
              tooltip.innerHTML = '<div style="font-weight:bold">' + escapeHtml(node.name) + '</div>' +
                '<div style="color:#ccc; margin-top:2px">' + escapeHtml(node.file) + ':' + (node.line || '?') + '</div>' +
                '<div style="color:#aaa; margin-top:2px">' + node.value + ' samples (' + pctValue + '%)</div>';
              tooltip.style.display = 'block';
              tooltip.style.left = (evt.clientX - rectBounds.left + 10) + 'px';
              tooltip.style.top = (evt.clientY - rectBounds.top + 10) + 'px';
            });
            rect.addEventListener('mousemove', function (evt) {
              const rectBounds = container.getBoundingClientRect();
              tooltip.style.left = (evt.clientX - rectBounds.left + 10) + 'px';
              tooltip.style.top = (evt.clientY - rectBounds.top + 10) + 'px';
            });
            rect.addEventListener('mouseleave', function () {
              tooltip.style.display = 'none';
            });
            rect.addEventListener('click', function (evt) {
              evt.preventDefault();
              currentRoot = node;
              render(node);
            });
            rect.addEventListener('contextmenu', function (evt) {
              evt.preventDefault();
              // Find parent of current root
              const parent = findParent(root, rootNode);
              currentRoot = parent || root;
              render(currentRoot);
            });
          }
        }

        // Recurse into children
        let offset = 0;
        const childrenTotal = node.children.reduce(function (s, c) { return s + c.value; }, 0) || 1;
        const scale = width / (node === rootNode ? childrenTotal : node.value);
        for (const child of node.children) {
          const childW = child.value * scale;
          drawNode(child, x + offset, childW);
          offset += childW;
        }
      }

      drawNode(rootNode, 0, viewW);
    }

    function findParent(node, target) {
      if (!node || node === target) return null;
      for (const child of node.children) {
        if (child === target) return node;
        const p = findParent(child, target);
        if (p) return p;
      }
      return null;
    }

    function computeMaxDepth(node, d) {
      d = d || 0;
      let max = d;
      for (const c of node.children) {
        max = Math.max(max, computeMaxDepth(c, d + 1));
      }
      return max;
    }

    function escapeHtml(s) {
      return String(s)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
    }

    resetBtn.addEventListener('click', function () {
      currentRoot = root;
      render(root);
    });
    searchInput.addEventListener('input', function () {
      searchQuery = searchInput.value.trim();
      render(currentRoot);
    });

    render(root);

    return {
      setSearch: function (q) {
        searchInput.value = q;
        searchQuery = q;
        render(currentRoot);
      },
      reset: function () {
        currentRoot = root;
        render(root);
      },
    };
  }

  // Expose to window for use from HTML
  root.flamegraph = {
    render: renderFlameGraph,
    buildTree: buildTree,
  };

  // Also expose as ES module if used that way
  if (typeof module !== 'undefined' && module.exports) {
    module.exports = { renderFlameGraph: renderFlameGraph, buildTree: buildTree };
  }
})(typeof window !== 'undefined' ? window : this);
