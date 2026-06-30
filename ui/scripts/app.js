const invoke = window.__TAURI__.core.invoke;

const KIND_DOT = {
  node: 'var(--tag-node)', 'next.js': 'var(--tag-node)', vite: 'var(--tag-node)',
  python: 'var(--tag-python)', php: 'var(--tag-php)',
  postgresql: 'var(--tag-db)', mysql: 'var(--tag-db)', redis: 'var(--tag-db)',
  mongodb: 'var(--tag-db)', sqlserver: 'var(--tag-db)', docker: 'var(--tag-docker)',
};
const CATS = [
  { id: 'node',   label: 'Node.js',       dot: 'var(--tag-node)',   kinds: ['node', 'next.js', 'vite'] },
  { id: 'python', label: 'Python',         dot: 'var(--tag-python)', kinds: ['python'] },
  { id: 'php',    label: 'PHP',            dot: 'var(--tag-php)',    kinds: ['php'] },
  { id: 'db',     label: 'Base de datos',  dot: 'var(--tag-db)',     kinds: ['postgresql', 'mysql', 'redis', 'mongodb', 'sqlserver'] },
  { id: 'docker', label: 'Docker',         dot: 'var(--tag-docker)', kinds: ['docker'] },
  { id: 'wsl',    label: 'WSL',            dot: 'var(--fg-muted)',   kinds: ['wsl'] },
];
const dotColor = (k) => KIND_DOT[k] || 'var(--fg-muted)';
const cap = (s) => (s ? s.charAt(0).toUpperCase() + s.slice(1) : s);
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const icons = () => window.lucide && window.lucide.createIcons();

function showToast(msg, type = 'error') {
  const t = document.createElement('div');
  t.setAttribute('role', 'alert');
  t.setAttribute('aria-live', 'polite');
  t.style.cssText = `position:fixed;bottom:16px;right:16px;background:var(--bg-hover);border:1px solid var(--border);
    border-left:3px solid ${type === 'error' ? 'var(--c-danger, #e55)' : 'var(--tag-node)'};
    color:var(--fg);padding:8px 12px;border-radius:6px;font-size:12px;max-width:320px;
    z-index:9999;box-shadow:0 4px 12px rgba(0,0,0,.3)`;
  t.textContent = msg;
  document.body.appendChild(t);
  setTimeout(() => t.remove(), 4000);
}

async function setConfig(cfg) {
  cfgWritesInFlight++;
  try {
    await cmd('set_config', { cfg });
  } finally {
    cfgWritesInFlight--;
  }
}

async function cmd(name, args = {}) {
  try {
    return await invoke(name, args);
  } catch (e) {
    showToast(`${name}: ${esc(String(e))}`);
    throw e;
  }
}

const state = { ports: [], cfg: null, autostart: false, view: 'dashboard', filter: 'all', search: '', selected: null, selectedPids: new Set() };
let cfgWritesInFlight = 0;

function visible() {
  return state.ports.filter((p) => {
    if (!state.cfg.show_system && p.is_system) return false;
    if (!state.cfg.show_unknown && p.kind === 'unknown') return false;
    return true;
  });
}
function inCategory(p, catId) {
  if (catId === 'all') return true;
  const cat = CATS.find((c) => c.id === catId);
  return cat ? cat.kinds.includes(p.kind) : false;
}
function matchSearch(p, q) {
  if (!q) return true;
  q = q.toLowerCase();
  return [p.port, p.app, p.name, p.project, p.framework, p.kind]
    .filter(Boolean).join(' ').toLowerCase().includes(q);
}

async function refresh() {
  const timeout = new Promise((_, r) =>
    setTimeout(() => r(new Error('scan timeout')), 5000)
  );
  let cfg, autostart, ports;
  try {
    [cfg, autostart, ports] = await Promise.race([
      Promise.all([invoke('get_config'), invoke('get_autostart'), invoke('list_ports')]),
      timeout,
    ]);
  } catch (e) {
    showToast(`refresh: ${esc(String(e))}`);
    return;
  }
  // Skip clobbering state.cfg with a stale read while a config write is in flight.
  if (cfgWritesInFlight === 0) state.cfg = cfg;
  state.autostart = autostart;
  state.ports = ports;
  const livePids = new Set(state.ports.map((p) => p.pid));
  for (const pid of state.selectedPids) {
    if (!livePids.has(pid)) state.selectedPids.delete(pid);
  }
  render();
}

function render() {
  renderSidebar();
  document.getElementById('tb-dashboard').classList.toggle('active', state.view === 'dashboard');
  document.getElementById('tb-settings').classList.toggle('active', state.view === 'settings');
  if (state.view === 'settings') renderSettings();
  else if (state.view === 'inspector') renderInspector();
  else renderDashboard();
  icons();
}

function renderSidebar() {
  const v = visible();
  const allCount = v.length;

  const activeCats = CATS
    .map((cat) => ({ ...cat, count: v.filter((p) => cat.kinds.includes(p.kind)).length }))
    .filter((cat) => cat.count > 0);

  if (state.filter !== 'all' && !activeCats.find((c) => c.id === state.filter)) {
    state.filter = 'all';
  }

  const nav = (id, icon, dot, label, count) => `
    <div class="nav ${state.filter === id ? 'active' : ''}" data-filter="${id}" role="button" tabindex="0">
      ${icon ? `<i data-lucide="${icon}"></i>` : `<span class="dot" style="background:${dot}"></span>`}
      <span class="label">${label}</span>
      <span class="badge"><span>${count}</span></span>
    </div>`;

  document.getElementById('sidebar').innerHTML = `
    <div class="sb-hdr"><span>PUERTOS</span></div>
    ${nav('all', 'layout-list', null, 'Todos', allCount)}
    ${activeCats.map((c) => nav(c.id, null, c.dot, c.label, c.count)).join('')}
    <div class="sb-div"></div>
    <div class="sb-spacer"></div>
    <div class="sb-div"></div>
    <div class="sb-toggle ${state.cfg.show_system ? 'on' : ''}" data-toggle="show_system" role="checkbox" aria-checked="${state.cfg.show_system}" tabindex="0">
      <i data-lucide="${state.cfg.show_system ? 'check-square' : 'square'}"></i><span>Mostrar sistema</span>
    </div>
    <div class="sb-toggle ${state.cfg.show_unknown ? 'on' : ''}" data-toggle="show_unknown" role="checkbox" aria-checked="${state.cfg.show_unknown}" tabindex="0">
      <i data-lucide="${state.cfg.show_unknown ? 'check-square' : 'square'}"></i><span>Sin clasificar</span>
    </div>`;
  document.querySelectorAll('.nav').forEach((n) => (n.onclick = () => { state.filter = n.dataset.filter; state.view = 'dashboard'; render(); }));
  document.querySelectorAll('.sb-toggle').forEach((t) => (t.onclick = async () => {
    const key = t.dataset.toggle;
    state.cfg[key] = !state.cfg[key];
    try {
      await setConfig(state.cfg);
    } catch (_) {
      state.cfg[key] = !state.cfg[key]; // rollback optimistic update
    }
    render();
  }));
}

function originText(p) {
  if (p.service) return p.service;
  if (p.parent_name) return `por ${p.parent_name}`;
  return 'ad-hoc';
}

function renderDashboard() {
  const rows = visible().filter((p) => inCategory(p, state.filter) && matchSearch(p, state.search));
  const c = document.getElementById('content');

  if (rows.length === 0) {
    c.innerHTML = `
      <div class="empty">
        <i data-lucide="zap" class="ei"></i>
        <h2>No hay puertos dev activos</h2>
        <p>Arranca tu servidor de desarrollo para verlo aquí.<br>
        <code>npm run dev</code> · <code>python -m uvicorn</code> · <code>php artisan serve</code></p>
        <button class="btn" id="empty-refresh"><i data-lucide="refresh-cw"></i> Refrescar</button>
      </div>`;
    const b = document.getElementById('empty-refresh');
    if (b) b.onclick = refresh;
    return;
  }

  const killSelBtn = state.selectedPids.size > 0
    ? `<button id="kill-sel" style="font-size:11px;padding:2px 8px;background:var(--c-danger,#c53030);color:#fff;border:none;border-radius:4px;cursor:pointer">
        Matar (${state.selectedPids.size})
      </button>`
    : 'ACCIONES';
  const head = `
    <div class="thead">
      <div class="h col-port" style="display:flex;align-items:center;gap:6px">
        <input type="checkbox" id="check-all" aria-label="Seleccionar todos los puertos" style="cursor:pointer" ${state.selectedPids.size === rows.length && rows.length > 0 ? 'checked' : ''}>
        PUERTO
      </div>
      <div class="h col-app">APLICACIÓN</div>
      <div class="h col-fw">FRAMEWORK · PROYECTO</div>
      <div class="h col-origin">ORIGEN</div>
      <div class="h col-actions">${killSelBtn}</div>
    </div>`;
  const ABTN_TITLES = {
    open: 'Abrir en navegador',
    copy: 'Copiar URL',
    folder: 'Abrir carpeta del proyecto',
    restart: 'Reiniciar proceso',
    kill: 'Matar proceso',
  };
  const abtn = (cls, icon) =>
    `<button class="abtn ${cls}" data-act="${cls}" title="${ABTN_TITLES[cls] || cls}"><i data-lucide="${icon}"></i></button>`;
  const body = rows.map((p) => `
    <div class="row" data-port="${p.port}" data-pid="${p.pid}" role="button" tabindex="0">
      <div class="col-port" style="display:flex;align-items:center;gap:6px">
        <input type="checkbox" class="row-check" data-pid="${p.pid}" aria-label="Seleccionar puerto ${p.port}" style="cursor:pointer;flex-shrink:0" ${state.selectedPids.has(p.pid) ? 'checked' : ''} onclick="event.stopPropagation()">
        <span class="dot" style="background:${dotColor(p.kind)}"></span>
        <span class="pn">:${p.port}</span>
      </div>
      <div class="col-app cell-2">
        <span class="a">${esc(p.app || p.name)}</span>
        <span class="b">${esc(p.name)}</span>
      </div>
      <div class="col-fw cell-2">
        <span class="a">${esc(p.framework || '—')}</span>
        <span class="b">${esc(p.project || '—')}</span>
      </div>
      <div class="col-origin">
        <span class="origin"><span class="od"></span><span>${esc(originText(p))}</span></span>
      </div>
      <div class="col-actions">
        <div class="acts">
          ${abtn('open', 'external-link')}
          ${abtn('copy', 'copy')}
          ${abtn('folder', 'folder-open')}
          ${abtn('restart', 'rotate-ccw')}
          ${abtn('kill', 'circle-x')}
        </div>
      </div>
    </div>`).join('');

  c.innerHTML = head + `<div class="tbody">${body}</div>`;
  bindRows();
}

function bindRows() {
  const checkAll = document.getElementById('check-all');
  if (checkAll) {
    checkAll.onclick = (e) => {
      e.stopPropagation();
      const all = document.querySelectorAll('.row-check');
      const check = checkAll.checked;
      all.forEach((cb) => {
        const pid = Number(cb.dataset.pid);
        check ? state.selectedPids.add(pid) : state.selectedPids.delete(pid);
        cb.checked = check;
      });
      renderDashboard();
      icons();
    };
  }
  const killSel = document.getElementById('kill-sel');
  if (killSel) {
    killSel.onclick = async () => {
      const pids = [...state.selectedPids];
      state.selectedPids.clear();
      try { await cmd('kill_ports', { pids }); setTimeout(refresh, 300); } catch (_) {}
    };
  }
  document.querySelectorAll('.row').forEach((row) => {
    const port = Number(row.dataset.port);
    const pid = Number(row.dataset.pid);
    const p = state.ports.find((x) => x.pid === pid && x.port === port);
    const cb = row.querySelector('.row-check');
    if (cb) {
      cb.onchange = () => {
        cb.checked ? state.selectedPids.add(pid) : state.selectedPids.delete(pid);
        renderDashboard();
        icons();
      };
    }
    row.querySelectorAll('.abtn').forEach((btn) => {
      btn.onclick = async (e) => {
        e.stopPropagation();
        const act = btn.dataset.act;
        if (act === 'open') cmd('open_url', { port }).catch(() => {});
        else if (act === 'copy') cmd('copy_url', { port }).catch(() => {});
        else if (act === 'folder') p && p.project_path && cmd('open_folder', { path: p.project_path }).catch(() => {});
        else if (act === 'restart') {
          try { await cmd('restart_port', { pid }); setTimeout(refresh, 600); } catch (_) {}
        } else if (act === 'kill') {
          try { await cmd('kill_port', { pid }); setTimeout(refresh, 300); } catch (_) {}
        }
      };
    });
    row.onclick = (e) => {
      if (e.target.classList.contains('row-check') || e.target.closest('.abtn')) return;
      state.selected = { pid, port }; state.view = 'inspector'; render();
    };
  });
}

function renderInspector() {
  const p = state.selected && state.ports.find((x) => x.pid === state.selected.pid && x.port === state.selected.port);
  const c = document.getElementById('content');
  if (!p) { state.view = 'dashboard'; renderDashboard(); return; }
  const field = (l, v, mono) => `
    <div class="set-row"><div class="meta"><div class="d">${l}</div>
    <div class="l" style="${mono ? 'font-family:var(--font-mono);font-size:12px' : ''}">${esc(v || '—')}</div></div></div>`;
  const parentInfo = p.parent_name
    ? `${esc(p.parent_name)}${p.parent_pid ? ` (pid ${p.parent_pid})` : ''}`
    : null;
  c.innerHTML = `
    <div class="settings">
      <div class="set-section">
        <h3>:${p.port} · ${esc(p.framework || cap(p.kind))}</h3>
        ${field('Aplicación', p.app || p.name)}
        ${field('Proceso', `${p.name} (pid ${p.pid})`, true)}
        ${parentInfo ? field('Proceso padre', parentInfo, true) : ''}
        ${field('Framework · Proyecto', `${p.framework || '—'} · ${p.project || '—'}`)}
        ${field('Origen', originText(p))}
        ${field('Ruta', p.exe, true)}
        ${field('Comando', (p.cmd || []).join(' '), true)}
      </div>
      <div class="set-section" id="probe-section">
        <h3>SONDEO DE PROTOCOLO</h3>
        <button class="btn" id="probe-btn"><i data-lucide="radio"></i> Sondear :${p.port}</button>
        <div id="probe-results" style="margin-top:12px"></div>
      </div>
      <div class="set-section" id="qr-section">
        <h3>QR LAN</h3>
        <button class="btn" id="qr-btn"><i data-lucide="qr-code"></i> Generar QR :${p.port}</button>
        <div id="qr-result" style="margin-top:12px"></div>
      </div>
      <div class="set-section" id="fw-section">
        <h3>FIREWALL</h3>
        <button class="btn" id="fw-btn"><i data-lucide="shield"></i> Verificar firewall</button>
        <div id="fw-result" style="margin-top:12px"></div>
      </div>
      <button class="btn" id="insp-back"><i data-lucide="layout-list"></i> Volver</button>
    </div>`;
  document.getElementById('insp-back').onclick = () => { state.view = 'dashboard'; render(); };
  document.getElementById('probe-btn').onclick = () => runProbe(p.port);
  document.getElementById('qr-btn').onclick = () => runQr(p.port);
  document.getElementById('fw-btn').onclick = () => runFirewallCheck(p.port);
}

async function runProbe(port) {
  const btn = document.getElementById('probe-btn');
  const resultsEl = document.getElementById('probe-results');
  if (!btn || !resultsEl) return;
  btn.disabled = true;
  btn.innerHTML = '<i data-lucide="loader-circle"></i> Sondeando...';
  icons();
  try {
    const result = await invoke('probe_port', { port });
    resultsEl.innerHTML = renderProbeResults(result);
    icons();
  } catch (e) {
    resultsEl.innerHTML = `<p style="color:var(--fg-muted);font-size:12px">Error: ${esc(String(e))}</p>`;
  } finally {
    btn.disabled = false;
    btn.innerHTML = '<i data-lucide="radio"></i> Sondear de nuevo';
    icons();
  }
}

async function runQr(port) {
  const btn = document.getElementById('qr-btn');
  const el = document.getElementById('qr-result');
  if (!btn || !el) return;
  btn.disabled = true;
  btn.innerHTML = '<i data-lucide="loader-circle"></i> Generando...';
  icons();
  try {
    const result = await invoke('get_qr_code', { port });
    if (!result) {
      el.innerHTML = `<p style="color:var(--fg-muted);font-size:12px">No se pudo obtener IP local.</p>`;
    } else {
      el.innerHTML = `
        <div style="font-family:var(--font-mono);font-size:11px;color:var(--fg-muted);margin-bottom:8px">${esc(result.url)}</div>
        ${renderQrSvg(result)}
        <p style="font-size:11px;color:var(--fg-muted);margin-top:6px">Escanea desde tu móvil en la misma red.</p>`;
    }
  } catch (e) {
    el.innerHTML = `<p style="color:var(--fg-muted);font-size:12px">Error: ${esc(String(e))}</p>`;
  } finally {
    btn.disabled = false;
    btn.innerHTML = '<i data-lucide="qr-code"></i> Generar QR';
    icons();
  }
}

function renderQrSvg({ size, cells }) {
  const cell = 8;
  const pad = 4;
  const total = (size + pad * 2) * cell;
  let rects = '';
  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) {
      if (cells[r * size + c]) {
        rects += `<rect x="${(c + pad) * cell}" y="${(r + pad) * cell}" width="${cell}" height="${cell}"/>`;
      }
    }
  }
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${total} ${total}" width="${total}" height="${total}" style="display:block;background:#fff;border-radius:6px;padding:4px">
    <rect width="${total}" height="${total}" fill="#fff"/>
    <g fill="#000">${rects}</g>
  </svg>`;
}

async function runFirewallCheck(port) {
  const btn = document.getElementById('fw-btn');
  const el = document.getElementById('fw-result');
  if (!btn || !el) return;
  btn.disabled = true;
  btn.innerHTML = '<i data-lucide="loader-circle"></i> Verificando...';
  icons();
  try {
    const r = await invoke('check_firewall', { port });
    const badge = (text, ok) =>
      `<span style="display:inline-block;padding:2px 8px;border-radius:4px;font-size:11px;font-weight:600;
        background:${ok ? 'var(--tag-node)' : 'color-mix(in srgb, var(--c-danger,#c53030) 20%, transparent)'};
        color:${ok ? 'var(--bg)' : 'var(--c-danger,#c53030)'}">${esc(text)}</span>`;
    el.innerHTML = `
      <div style="display:flex;flex-direction:column;gap:6px;font-size:12px">
        <div style="display:flex;align-items:center;gap:8px">
          ${badge(r.blocked ? 'BLOQUEADO' : 'ACCESIBLE', !r.blocked)}
          <span style="color:var(--fg-muted)">desde LAN</span>
        </div>
        ${r.has_allow_rule
          ? `<span style="color:var(--fg-muted)">${r.rule_count} regla(s) Allow encontrada(s) en Windows Firewall</span>`
          : `<span style="color:var(--fg-muted)">Sin regla Allow explícita — puerto no accesible desde fuera de localhost</span>`}
      </div>`;
  } catch (e) {
    el.innerHTML = `<p style="color:var(--fg-muted);font-size:12px">Error: ${esc(String(e))}</p>`;
  } finally {
    btn.disabled = false;
    btn.innerHTML = '<i data-lucide="shield"></i> Verificar de nuevo';
    icons();
  }
}

function renderProbeResults(result) {
  const PROTO_ICONS = {
    http: 'globe', https: 'lock', websocket: 'cable', redis: 'database',
    postgresql: 'database', mysql: 'database',
  };
  const latencyColor = (ms) => ms < 50 ? 'var(--tag-node)' : ms < 200 ? 'var(--tag-python)' : 'var(--fg-muted)';

  let html = `<div style="font-size:11px;color:var(--fg-muted);margin-bottom:8px">
    TCP <span style="color:${result.tcp_open ? 'var(--tag-node)' : 'var(--c-danger)'}">
      ${result.tcp_open ? '● abierto' : '● cerrado'}
    </span> · ${result.tcp_latency_ms}ms
  </div>`;

  const successful = result.probes.filter((r) => r.success);
  const failed = result.probes.filter((r) => !r.success);

  if (successful.length === 0 && failed.length === 0) {
    return html + '<p style="color:var(--fg-muted);font-size:12px">Sin resultados.</p>';
  }

  html += '<div style="display:flex;flex-direction:column;gap:6px">';

  for (const probe of successful) {
    const icon = PROTO_ICONS[probe.protocol] || 'zap';
    html += `<div style="background:var(--bg-hover);border-radius:6px;padding:8px 10px">
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:${probe.headers ? '6px' : '0'}">
        <i data-lucide="${icon}" style="width:14px;height:14px;color:var(--tag-node)"></i>
        <span style="font-weight:600;font-size:12px;text-transform:uppercase">${esc(probe.protocol)}</span>
        <span style="color:var(--fg-muted);font-size:11px">${esc(probe.status || '')}</span>
        <span style="margin-left:auto;font-size:11px;color:${latencyColor(probe.latency_ms)}">${probe.latency_ms}ms</span>
      </div>
      ${probe.headers ? renderHeaders(probe.headers) : ''}
    </div>`;
  }

  if (failed.length > 0) {
    html += `<details style="margin-top:4px">
      <summary style="font-size:11px;color:var(--fg-muted);cursor:pointer;user-select:none">
        ${failed.length} protocolo(s) no detectado(s)
      </summary>
      <div style="display:flex;flex-direction:column;gap:4px;margin-top:6px">`;
    for (const probe of failed) {
      const icon = PROTO_ICONS[probe.protocol] || 'zap';
      html += `<div style="display:flex;align-items:center;gap:8px;padding:4px 0">
        <i data-lucide="${icon}" style="width:12px;height:12px;color:var(--fg-muted)"></i>
        <span style="font-size:11px;color:var(--fg-muted);text-transform:uppercase">${esc(probe.protocol)}</span>
        <span style="font-size:11px;color:var(--fg-muted)">${esc(probe.error || '—')}</span>
        <span style="margin-left:auto;font-size:11px;color:var(--fg-muted)">${probe.latency_ms}ms</span>
      </div>`;
    }
    html += '</div></details>';
  }

  html += '</div>';
  return html;
}

function renderHeaders(headers) {
  const important = ['content-type', 'server', 'x-powered-by', 'cache-control', 'access-control-allow-origin'];
  const shown = Object.entries(headers)
    .sort(([a], [b]) => {
      const ai = important.indexOf(a.toLowerCase());
      const bi = important.indexOf(b.toLowerCase());
      if (ai !== -1 && bi !== -1) return ai - bi;
      if (ai !== -1) return -1;
      if (bi !== -1) return 1;
      return a.localeCompare(b);
    })
    .slice(0, 12);

  if (shown.length === 0) return '';
  return `<div style="display:flex;flex-direction:column;gap:2px;border-top:1px solid var(--border);padding-top:6px">
    ${shown.map(([k, v]) => `<div style="display:flex;gap:8px;font-size:11px;font-family:var(--font-mono)">
      <span style="color:var(--fg-muted);flex-shrink:0;min-width:160px">${esc(k)}</span>
      <span style="color:var(--fg);overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${esc(v)}</span>
    </div>`).join('')}
  </div>`;
}

function renderSettings() {
  const c = document.getElementById('content');
  const cfg = state.cfg;
  const sw = (key, on) => `<div class="switch ${on ? 'on' : ''}" data-sw="${key}" role="switch" aria-checked="${on}" tabindex="0"></div>`;
  const row = (label, desc, control) => `
    <div class="set-row"><div class="meta"><div class="l">${label}</div><div class="d">${desc}</div></div>${control}</div>`;
  c.innerHTML = `
    <div class="settings">
      <div class="set-section"><h3>GENERAL</h3>
        ${row('Intervalo de sondeo', 'Cada cuántos segundos se refrescan los puertos', `<input class="set-input" id="poll" type="number" min="1" value="${cfg.poll_interval_secs}">`)}
        ${row('Arrancar al inicio', 'Iniciar Killport con Windows', sw('autostart', state.autostart))}
      </div>
      <div class="set-section"><h3>FILTROS</h3>
        ${row('Mostrar procesos del sistema', 'Incluir procesos del SO', sw('show_system', cfg.show_system))}
        ${row('Mostrar sin clasificar', 'Incluir procesos no-dev', sw('show_unknown', cfg.show_unknown))}
        ${row('Puertos ignorados', 'Separados por coma', `<input class="set-input" id="ignore" value="${esc((cfg.ignore_ports || []).join(', '))}">`)}
        ${row('Puertos reservados', 'Notificar si otro proceso los ocupa', `<input class="set-input" id="reserved" value="${esc((cfg.reserved_ports || []).join(', '))}">`)}
      </div>
      <div class="set-section"><h3>NOTIFICACIONES</h3>
        ${row('Notificaciones', 'Avisar al abrir o cerrar un puerto', sw('notifications', cfg.notifications))}
      </div>
    </div>`;

  document.querySelectorAll('.switch').forEach((s) => (s.onclick = async () => {
    const key = s.dataset.sw;
    if (key === 'autostart') {
      state.autostart = !state.autostart;
      try {
        await cmd('set_autostart', { enabled: state.autostart });
      } catch (_) {
        state.autostart = !state.autostart; // rollback
      }
    } else {
      cfg[key] = !cfg[key];
      try {
        await setConfig(cfg);
        if (key === 'show_system' || key === 'show_unknown') {
          state.view = 'dashboard';
        }
      } catch (_) {
        cfg[key] = !cfg[key]; // rollback
      }
    }
    render();
  }));
  const save = async () => {
    try { await setConfig(state.cfg); } catch (_) {}
  };
  const poll = document.getElementById('poll');
  poll.onchange = () => { cfg.poll_interval_secs = Math.max(1, Number(poll.value) || 3); save(); };
  const parsePorts = (v) => v.split(',').map((x) => parseInt(x.trim(), 10)).filter((n) => !isNaN(n) && n >= 1 && n <= 65535);
  const ignore = document.getElementById('ignore');
  ignore.onchange = () => { cfg.ignore_ports = parsePorts(ignore.value); save(); };
  const reserved = document.getElementById('reserved');
  reserved.onchange = () => { cfg.reserved_ports = parsePorts(reserved.value); save(); };
}

// Make role="button"/role="checkbox"/role="switch" divs keyboard-operable (Enter/Space).
document.addEventListener('keydown', (e) => {
  if (e.key !== 'Enter' && e.key !== ' ') return;
  const el = e.target.closest('[role="button"], [role="checkbox"], [role="switch"]');
  if (!el) return;
  e.preventDefault();
  el.click();
});

document.querySelector('.logo').onclick = document.querySelector('.app-name').onclick = () => { state.view = 'dashboard'; state.filter = 'all'; render(); };
document.getElementById('search').oninput = (e) => { state.search = e.target.value; if (state.view === 'dashboard') renderDashboard(), icons(); };
document.getElementById('tb-refresh').onclick = refresh;
document.getElementById('tb-dashboard').onclick = () => { state.view = 'dashboard'; render(); };
document.getElementById('tb-settings').onclick = () => { state.view = 'settings'; render(); };

refresh();
let _inFlight = false;
setInterval(async () => {
  if (_inFlight || state.view !== 'dashboard') return;
  _inFlight = true;
  try { await refresh(); } finally { _inFlight = false; }
}, 3000);
