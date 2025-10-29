/**
 * app-state.js
 * Global application state. Centralized and reactive.
 */
(function () {
  'use strict';

  window.AppState = {
    projectRoot: '',
    currentPatch: '',
    
    // Session health metrics, mirrored from the backend
    session: {
      exchange_count: 0,
      total_errors: 0,
    },

    // UI-specific state
    ui: {
      isPatchAreaActive: false,
      isPreviewInFlight: false,
      isConsoleVisible: false,
    },
  };

  /**
   * Emits a custom event on the window object.
   * @param {string} name - The name of the event (e.g., 'session-loaded').
   * @param {object} [detail={}] - The data payload for the event.
   */
  window.emitAppEvent = function (name, detail) {
    window.dispatchEvent(new CustomEvent('app:' + name, { detail: detail || {} }));
  };

  /**
   * Listens for a custom app event on the window object.
   * @param {string} name - The name of the event to listen for.
   * @param {Function} handler - The callback function to execute.
   */
  window.onAppEvent = function (name, handler) {
    window.addEventListener('app:' + name, handler);
  };

  console.log('[app-state] Initialized.');
})();