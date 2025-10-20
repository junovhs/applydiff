/**
 * diff-panel.js
 * Manages diff preview rendering and Apply button.
 */
(function() {
  'use strict';

  let diffPre, applyBtn;

  function init() {
    diffPre = document.getElementById('diff-pre');
    applyBtn = document.getElementById('btn-apply');
    
    // Listen for preview results
    window.onAppEvent('preview-ready', (e) => {
      const { diff, hasError, hasDiff } = e.detail;
      renderDiff(diff);
      updateApplyButton(hasDiff, hasError);
    });
    
    // Apply button click
    applyBtn.addEventListener('click', () => {
      const patch = window.AppState.currentPatch;
      const diff = diffPre.innerHTML;
      
      if (!patch) {
        logToConsole('⚠️ No patch to apply.');
        return;
      }
      
      logToConsole('🔧 Apply button clicked');
      window.emitAppEvent('apply-requested', { patch, diff });
    });
  }

  function renderDiff(udiff) {
    if (!udiff || !udiff.trim()) {
      diffPre.style.color = '#777';
      diffPre.innerHTML = 'No preview yet.';
      return;
    }
    
    diffPre.style.color = '';
    const lines = udiff.replace(/\r\n/g, '\n').split('\n');
    
    diffPre.innerHTML = lines.map(line => {
      let cls = 'd-ctx';
      if (line.startsWith('+++') || line.startsWith('---') || line.startsWith('@@')) {
        cls = 'd-hdr';
      } else if (line.startsWith('+')) {
        cls = 'd-add';
      } else if (line.startsWith('-')) {
        cls = 'd-del';
      }
      return `<span class="${cls}">${window.escapeHtml(line)}</span>`;
    }).join('\n');
  }

  function updateApplyButton(hasDiff, hasError) {
    if (!hasDiff) {
      applyBtn.style.display = 'none';
      return;
    }
    
    applyBtn.style.display = 'inline-block';
    applyBtn.classList.toggle('warn', hasError);
    applyBtn.textContent = hasError ? 'Apply Valid Changes' : 'Apply Patch';
  }

  // Auto-init
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

  console.log('[diff-panel] initialized');
})();