// Depot Navigation Bar - injected into all proxied apps
(function() {
  // Don't inject twice
  if (document.getElementById('depot-nav')) return;

  // Determine current app
  const path = window.location.pathname;
  const current = path.startsWith('/operator/') ? 'operator'
                : path.startsWith('/grafana/') ? 'grafana'
                : path.startsWith('/influx/') ? 'influx'
                : 'portal';

  // Create nav element
  const nav = document.createElement('div');
  nav.id = 'depot-nav';
  nav.innerHTML = `
    <a href="/" class="${current === 'portal' ? 'active' : ''}" title="Portal">⌂</a>
    <a href="/operator/" class="${current === 'operator' ? 'active' : ''}" title="Operator">Operator</a>
    <a href="/grafana/" class="${current === 'grafana' ? 'active' : ''}" title="Dashboards">Dashboards</a>
    <a href="/influx/" class="${current === 'influx' ? 'active' : ''}" title="Database">Database</a>
  `;

  // Inject styles
  const style = document.createElement('style');
  style.textContent = `
    #depot-nav {
      position: fixed;
      bottom: 12px;
      left: 50%;
      transform: translateX(-50%);
      display: flex;
      gap: 2px;
      background: rgba(0, 0, 0, 0.9);
      border: 1px solid rgba(255, 255, 255, 0.2);
      padding: 4px;
      font-family: "Berkeley Mono", "SF Mono", monospace;
      font-size: 11px;
      z-index: 2147483647;
      backdrop-filter: blur(8px);
    }
    html:has(#depot-nav), body:has(#depot-nav) {
      overflow: visible !important;
    }
    #depot-nav a {
      color: rgba(255, 255, 255, 0.7);
      text-decoration: none;
      padding: 4px 10px;
      transition: all 0.15s;
    }
    #depot-nav a:hover {
      background: #ff6600;
      color: #000;
    }
    #depot-nav a.active {
      background: rgba(255, 255, 255, 0.15);
      color: #fff;
    }
  `;

  document.head.appendChild(style);
  document.body.appendChild(nav);
})();
