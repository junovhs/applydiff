/**
 * ConsolePanel.js
 * Manages the slide-out console panel.
 */
(function () {
  'use strict';

  const consolePanel = document.getElementById('console-panel');
  const consoleArea = document.getElementById('console-area');
  const consoleLabel = document.getElementById('console-label');
  const selfTestBtn = document.getElementById('self-test-btn');
  const clearBtn = document.getElementById('clear-console-btn');
  const toggleBtn = document.getElementById('console-toggle');

  function init() {
    onAppEvent('console-log', (e) => {
      const { message, level } = e.detail;
      appendLog(message, level);
    });

    toggleBtn.addEventListener('click', toggleConsole);
    clearBtn.addEventListener('click', () => {
      consoleArea.textContent = 'cleared.';
    });
  }

  function toggleConsole() {
    const isVisible = !window.AppState.ui.isConsoleVisible;
    window.AppState.ui.isConsoleVisible = isVisible;

    consolePanel.classList.toggle('visible', isVisible);
    consoleLabel.classList.toggle('visible', isVisible);
    selfTestBtn.classList.toggle('visible', isVisible);
    clearBtn.classList.toggle('visible', isVisible);

    if (isVisible) {
      logToConsole('Console opened.');
    }
  }

  function appendLog(message, level) {
    const now = new Date().toLocaleTimeString();
    const line = document.createElement('div');
    line.style.fontFamily = 'inherit';
    line.style.whiteSpace = 'pre-wrap';
    line.style.marginBottom = '4px';

    const colors = { error: 'var(--err)', warn: 'var(--warn)', success: 'var(--ok)' };
    line.style.color = colors[level] || 'var(--text-muted)';
    
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
})();