const invoke = window.__TAURI__.core.invoke;

const KIND_DOT = {
  node: 'var(--tag-node)',
  'next.js': 'var(--tag-node)',
  vite: 'var(--tag-node)',
  python: 'var(--tag-python)',
  php: 'var(--tag-php)',
  postgresql: 'var(--tag-db)',
  mysql: 'var(--tag-db)',
  redis: 'var(--tag-db)',
  mongodb: 'var(--tag-db)',
  sqlserver: 'var(--tag-db)',
  docker: 'var(--tag-docker)',
};

const dotColor = (kind) => KIND_DOT[kind] || 'var(--fg-muted)';
const cap = (s) => (s ? s.charAt(0).toUpperCase() + s.slice(1) : s);

let cfg = { show_system: false };

function devFilter(p) {
  return cfg.show_system ? true : !p.is_system && p.kind !== 'unknown';
}

async function load() {
  const list = document.getElementById('list');
  let cfgResult, autostart, ports;
  try {
    [cfgResult, autostart, ports] = await Promise.all([
      invoke('get_config'),
      invoke('get_autostart'),
      invoke('list_ports'),
    ]);
  } catch (e) {
    list.innerHTML = `<div class="empty-mini">Error: ${esc(String(e))}</div>`;
    return;
  }
  cfg = cfgResult;
  ports = ports.filter(devFilter);

  document.getElementById('count').textContent =
    `${ports.length} puerto${ports.length === 1 ? '' : 's'} activo${ports.length === 1 ? '' : 's'}`;

  if (ports.length === 0) {
    list.innerHTML = '<div class="empty-mini">No hay puertos dev activos</div>';
  } else {
    list.innerHTML = ports
      .map((p) => {
        const label = `:${p.port} — ${p.framework || cap(p.kind)}`;
        const proj = p.project || p.app || '—';
        return `
        <div class="port-row" data-pid="${p.pid}" data-port="${p.port}">
          <span class="dot" style="background:${dotColor(p.kind)}"></span>
          <div class="info">
            <span class="pp">${esc(label)}</span>
            <span class="pj">${esc(proj)}</span>
          </div>
          <div class="acts">
            <span class="a kill" title="Matar proceso"><i data-lucide="x"></i></span>
            <span class="a open" title="Ver en panel"><i data-lucide="chevron-right"></i></span>
          </div>
        </div>`;
      })
      .join('');
  }

  setToggle('tog-autostart', autostart);
  setToggle('tog-system', cfg.show_system);

  window.lucide && window.lucide.createIcons();
  bindRows();
}

function esc(s) {
  return String(s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
}

function setToggle(id, on) {
  document.getElementById(id).classList.toggle('on', !!on);
}

function bindRows() {
  document.querySelectorAll('.port-row').forEach((row) => {
    const pid = Number(row.dataset.pid);
    row.querySelector('.kill').onclick = async (e) => {
      e.stopPropagation();
      try { await invoke('kill_port', { pid }); } catch (_) {}
      load();
    };
    row.querySelector('.open').onclick = (e) => { e.stopPropagation(); invoke('open_main').catch(() => {}); };
    row.onclick = () => invoke('open_main').catch(() => {});
  });
}

document.getElementById('refresh').onclick = load;
document.getElementById('act-open').onclick = () => invoke('open_main').catch(() => {});
document.getElementById('exit').onclick = () => invoke('quit_app').catch(() => {});

document.getElementById('tog-autostart').onclick = async () => {
  const next = !document.getElementById('tog-autostart').classList.contains('on');
  try { await invoke('set_autostart', { enabled: next }); } catch (_) {}
  load();
};
document.getElementById('tog-system').onclick = async () => {
  cfg.show_system = !cfg.show_system;
  try { await invoke('set_config', { cfg }); } catch (_) { cfg.show_system = !cfg.show_system; }
  load();
};

load();
let _loading = false;
setInterval(async () => {
  if (_loading) return;
  _loading = true;
  try { await load(); } finally { _loading = false; }
}, 2500);
