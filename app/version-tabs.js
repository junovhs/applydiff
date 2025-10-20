/**
 * version-tabs.js
 * Manages version history tabs and navigation.
 */
(function() {
  'use strict';

  let tabsContainer, pageInfo, prevBtn, nextBtn;

  function init() {
    tabsContainer = document.getElementById('version-tabs');
    pageInfo = document.getElementById('page-info');
    prevBtn = document.getElementById('prev-page');
    nextBtn = document.getElementById('next-page');
    
    // Create v0 baseline when directory selected
    window.onAppEvent('directory-selected', (e) => {
      const { path } = e.detail;
      window.AppState.versions = [{
        id: 0,
        timestamp: new Date().toLocaleString(),
        patch: '',
        diff: '',
        note: 'Baseline (before any patches)',
        files: []
      }];
      window.AppState.currentVersion = 0;
      renderTabs();
      logToConsole('📌 Created v0 baseline for ' + path);
    });
    
    // Add new version when patch applied
    window.onAppEvent('apply-complete', (e) => {
      const { patch, diff, log } = e.detail;
      
      window.AppState.versions.push({
        id: window.AppState.versions.length,
        timestamp: new Date().toLocaleString(),
        patch,
        diff,
        note: '',
        files: extractFilesFromPatch(patch)
      });
      
      window.AppState.currentVersion = window.AppState.versions.length - 1;
      renderTabs();
      logToConsole(`📌 Created v${window.AppState.currentVersion}`);
    });
    
    // Pagination
    prevBtn.addEventListener('click', () => {
      if (window.AppState.currentVersion > 0) {
        selectVersion(window.AppState.currentVersion - 1);
      }
    });
    
    nextBtn.addEventListener('click', () => {
      const maxIdx = window.AppState.versions.length - 1;
      if (window.AppState.currentVersion < maxIdx) {
        selectVersion(window.AppState.currentVersion + 1);
      }
    });
  }

  function renderTabs() {
    const versions = window.AppState.versions;
    const current = window.AppState.currentVersion;
    
    tabsContainer.innerHTML = '';
    
    versions.forEach((v, i) => {
      const tab = document.createElement('button');
      tab.className = 'version-tab' + (i === current ? ' active' : '');
      
      const displayText = v.note ? `v${v.id} 📝` : `v${v.id}`;
      tab.textContent = displayText;
      
      const tooltip = `${v.timestamp}\nFiles: ${v.files.join(', ') || 'none'}${v.note ? '\nNote: ' + v.note : '\n(Right-click to add note)'}`;
      tab.title = tooltip;
      
      tab.onclick = () => selectVersion(i);
      
      tab.oncontextmenu = (e) => {
        e.preventDefault();
        const newNote = prompt(`Note for v${v.id}:`, v.note || '');
        if (newNote !== null) {
          versions[i].note = newNote.trim();
          renderTabs();
          logToConsole(`📝 Updated note for v${v.id}`);
        }
      };
      
      tabsContainer.appendChild(tab);
    });
    
    pageInfo.textContent = versions.length ? `${current + 1} / ${versions.length}` : '0 / 0';
  }

  function selectVersion(index) {
    if (index < 0 || index >= window.AppState.versions.length) return;
    
    window.AppState.currentVersion = index;
    const v = window.AppState.versions[index];
    
    // Emit event to repopulate patch panel
    window.emitAppEvent('version-selected', {
      index,
      patch: v.patch,
      diff: v.diff
    });
    
    renderTabs();
    logToConsole(`📋 Viewing v${v.id} - ${v.timestamp}`);
  }

  function extractFilesFromPatch(patch) {
    const files = [];
    const classicMatches = patch.matchAll(/^>>> file: (.+?) \|/gm);
    for (const m of classicMatches) files.push(m[1].trim());
    
    const armoredMatches = patch.matchAll(/^Path: (.+)$/gm);
    for (const m of armoredMatches) files.push(m[1].trim());
    
    return [...new Set(files)]; // dedupe
  }

  // Auto-init
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

  console.log('[version-tabs] initialized');
})();