/**
 * console-panel.js
 * Manages console panel toggle and log display.
 */
(function() {
  'use strict';

  let consolePanel, consoleArea, consoleLabel, selfTestBtn, clearBtn, toggleBtn;

  function init() {
    consolePanel = document.getElementById('console-panel');
    consoleArea = document.getElementById('console-area');
    consoleLabel = document.getElementById('console-label');
    selfTestBtn = document.getElementById('self-test-btn');
    clearBtn = document.getElementById('clear-console-btn');
    toggleBtn = document.getElementById('console-toggle');
    
    // Listen for log events
    window.onAppEvent('console-log', (e) => {
      const { message, level } = e.detail;
      appendLog(message, level);
    });
    
    // Toggle console
    toggleBtn.addEventListener('click', toggleConsole);
    
    // Clear console
    clearBtn.addEventListener('click', () => {
      consoleArea.textContent = 'cleared.';
    });
    
    // Self-test button
    selfTestBtn.addEventListener('click', () => {
      window.emitAppEvent('self-test-requested');
    });
  }

  function toggleConsole() {
    const visible = !window.AppState.consoleVisible;
    window.AppState.consoleVisible = visible;
    
    consolePanel.classList.toggle('visible', visible);
    consoleLabel.classList.toggle('visible', visible);
    selfTestBtn.classList.toggle('visible', visible);
    clearBtn.classList.toggle('visible', visible);
    
    if (visible) {
      logToConsole('✔️ Console opened');
    }
  }

  function appendLog(message, level) {
    const now = new Date().toLocaleTimeString();
    const line = document.createElement('div');
    line.style.marginBottom = '4px';
    line.style.fontFamily = 'inherit';
    line.style.whiteSpace = 'pre-wrap';
    
    // Color by level or emoji prefix
    if (level === 'error' || message.startsWith('❌')) {
      line.style.color = '#ef4444';
    } else if (message.startsWith('✅') || message.startsWith('✔')) {
      line.style.color = '#22c55e';
    } else if (message.startsWith('⚠')) {
      line.style.color = '#f59e0b';
    } else if (message.startsWith('👁') || message.startsWith('📋') || message.startsWith('📁')) {
      line.style.color = '#9aa4b2';
    } else {
      line.style.color = '#6b6b6b';
    }
    
    line.textContent = `[${now}] ${message}`;
    consoleArea.appendChild(line);
    consoleArea.scrollTop = consoleArea.scrollHeight;
  }

  // Auto-init
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

  console.log('[console-panel] initialized');
})();