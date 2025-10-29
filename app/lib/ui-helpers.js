/**
 * ui-helpers.js
 * Shared UI utility functions. No state. Interacts with the original DOM.
 */
(function () {
  'use strict';

  const statusChip = document.getElementById('status');
  const healthErrorsEl = document.getElementById('health-errors');
  const healthExchangesEl = document.getElementById('health-exchanges');

  /**
   * Escapes HTML special characters to prevent XSS.
   * @param {string} str - The string to escape.
   * @returns {string} The escaped string.
   */
  window.escapeHtml = function (str) {
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  };

  /**
   * Updates the status chip in the patch panel header.
   * @param {string} text - The text to display.
   * @param {string} [tone=''] - The color tone for the chip (e.g., 'ok', 'warn', 'err').
   */
  window.setStatus = function (text, tone) {
    if (!statusChip) return;
    statusChip.textContent = text;
    // The CSS uses border color to show status
    const colors = { ok: 'var(--ok)', warn: 'var(--warn)', err: 'var(--err)' };
    const color = colors[tone] || 'var(--border)';
    statusChip.style.borderColor = color;
    statusChip.style.color = color === 'var(--border)' ? 'var(--text-muted)' : color;
  };

  /**
   * Updates the Conversation Health Monitor display.
   * @param {object} sessionMetrics - The session object from AppState.
   */
  window.updateHealthDisplay = function (sessionMetrics) {
    if (healthErrorsEl) {
      healthErrorsEl.textContent = `Errors: ${sessionMetrics.total_errors}/3`;
      healthErrorsEl.style.color = 'var(--text-muted)';
      if (sessionMetrics.total_errors >= 2) healthErrorsEl.style.color = 'var(--warn)';
      if (sessionMetrics.total_errors >= 3) healthErrorsEl.style.color = 'var(--err)';
    }
    if (healthExchangesEl) {
      healthExchangesEl.textContent = `Exchanges: ${sessionMetrics.exchange_count}/10`;
      healthExchangesEl.style.color = 'var(--text-muted)';
      if (sessionMetrics.exchange_count >= 7) healthExchangesEl.style.color = 'var(--warn)';
    }
  };

  /**
   * Emits a console-log event to be handled by the console panel.
   * @param {string} message - The message to log.
   * @param {'info' | 'error' | 'warn' | 'success'} [level='info'] - The log level.
   */
  window.logToConsole = function (message, level) {
    window.emitAppEvent('console-log', { message, level: level || 'info' });
  };

  console.log('[ui-helpers] Initialized.');
})();