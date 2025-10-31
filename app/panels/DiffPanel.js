/**
 * DiffPanel.js
 * Manages rendering the diff preview and the "Apply" button.
 */
(function () {
  'use strict';

  const diffPre = document.getElementById('diff-pre');
  const applyBtn = document.getElementById('btn-apply');

  function init() {
    applyBtn.addEventListener('click', () => {
      if (window.AppState.currentPatch) {
        emitAppEvent('apply-requested', { patch: window.AppState.currentPatch });
      }
    });

    onAppEvent('preview-ready', (e) => {
      renderDiff(e.detail.diff);
      updateApplyButton(e.detail.hasDiff, e.detail.hasError);
      logToConsole(e.detail.log);
    });

    onAppEvent('apply-successful', () => {
      diffPre.innerHTML = 'No preview yet.';
      diffPre.style.color = '#777';
      applyBtn.style.display = 'none';
    });
    
    onAppEvent('project-loaded', () => {
        diffPre.innerHTML = 'No preview yet.';
        diffPre.style.color = '#777';
        applyBtn.style.display = 'none';
    });
  }

  function renderDiff(udiff) {
    if (!udiff || !udiff.trim()) {
      diffPre.innerHTML = 'No changes detected in preview.';
      diffPre.style.color = '#777';
      return;
    }

    diffPre.style.color = ''; // Reset color
    const lines = udiff.replace(/\r\n/g, '\n').split('\n');
    const html = lines.map(line => {
        let cls = 'd-ctx';
        if (line.startsWith('+++') || line.startsWith('---') || line.startsWith('@@')) cls = 'd-hdr';
        else if (line.startsWith('+')) cls = 'd-add';
        else if (line.startsWith('-')) cls = 'd-del';
        return `<span class="${cls}">${window.escapeHtml(line)}</span>`;
      })
      .join('\n');
    diffPre.innerHTML = html;
  }

  function updateApplyButton(hasDiff, hasError) {
    if (!hasDiff) {
      applyBtn.style.display = 'none';
      return;
    }
    applyBtn.style.display = 'block';
    applyBtn.classList.toggle('warn', hasError);
    applyBtn.textContent = hasError ? 'Apply Partial' : 'Apply Patch';
  }

  // Auto-init
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();