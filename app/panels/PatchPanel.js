/**
 * PatchPanel.js
 * Manages the patch input area and associated header buttons.
 */
(function () {
  'use strict';

  const patchArea = document.getElementById('patch-area');
  const placeholder = document.getElementById('patch-placeholder');
  const selectDirBtn = document.getElementById('btn-select-dir');
  const copyBriefingBtn = document.getElementById('btn-copy-briefing');
  const refreshBtn = document.getElementById('btn-refresh-session'); // Assuming an ID for the refresh button

  function init() {
    // Wire up header buttons
    selectDirBtn.addEventListener('click', () => {
      emitAppEvent('select-project-requested');
    });

    copyBriefingBtn.addEventListener('click', () => {
      emitAppEvent('copy-briefing-requested');
    });
    
    // Add event listener for the refresh button if it exists
    if(refreshBtn) {
        refreshBtn.addEventListener('click', () => {
            emitAppEvent('refresh-session-requested');
        });
    }


    // Handle clicks on the patch area (for pasting)
    patchArea.addEventListener('click', onPatchAreaClick);

    // Listen for session state changes
    onAppEvent('session-loaded', (e) => {
      placeholder.textContent = 'Click to Paste Patch';
      patchArea.classList.remove('disabled');
      copyBriefingBtn.disabled = false;
      if(refreshBtn) refreshBtn.disabled = false;
      
      logToConsole(`Project loaded: ${e.detail.path}`);
      setStatus('ready');
      
      // THIS IS THE NEW, CRITICAL PIECE
      emitAppEvent('session-state-sync-requested');
    });
    
    onAppEvent('session-load-failed', () => {
        placeholder.textContent = 'Select a project to begin';
        patchArea.classList.add('disabled');
        copyBriefingBtn.disabled = true;
        if(refreshBtn) refreshBtn.disabled = true;
    });

    onAppEvent('apply-successful', () => {
      if (editorEl) {
        editorEl.value = '';
        window.AppState.currentPatch = '';
        placeholder.style.display = 'block';
      }
      setStatus('idle');
    });
  }

  async function onPatchAreaClick() {
    if (patchArea.classList.contains('disabled')) return;

    ensureEditorExists();
    const textFromClipboard = await window.__TAURI__.clipboardManager.readText();

    if (textFromClipboard && textFromClipboard.trim()) {
      logToConsole(`📋 Pasted ${textFromClipboard.length} chars from clipboard.`);
      editorEl.value = textFromClipboard;
      window.AppState.currentPatch = textFromClipboard;
      placeholder.style.display = 'none';
      emitAppEvent('preview-requested', { patch: textFromClipboard });
      editorEl.focus();
    } else {
      logToConsole('⌨️ Clipboard empty. Enter patch manually.');
      editorEl.focus();
    }
  }

  function ensureEditorExists() {
    if (editorEl) return;

    editorEl = document.createElement('textarea');
    editorEl.className = 'patch-editor';
    patchArea.appendChild(editorEl);

    editorEl.addEventListener('input', () => {
      window.setStatus('typing…', 'warn');
      clearTimeout(typingTimer);
      typingTimer = setTimeout(() => {
        const patch = editorEl.value.trim();
        if (patch) {
          window.AppState.currentPatch = patch;
          emitAppEvent('preview-requested', { patch });
        }
      }, 1500); // 1.5 second debounce
    });
  }

  // Auto-init
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();