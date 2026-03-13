var $ = function(id) { return document.getElementById(id); };
var cpuHistory = [];
var CPU_HISTORY_LEN = 60;

function formatBytes(b) {
  if (b === 0) return '0 B';
  var units = ['B', 'KB', 'MB', 'GB', 'TB'];
  var i = Math.floor(Math.log(b) / Math.log(1024));
  return (b / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
}

function formatUptime(s) {
  var d = Math.floor(s / 86400);
  var h = Math.floor((s % 86400) / 3600);
  var m = Math.floor((s % 3600) / 60);
  if (d > 0) return d + 'd ' + h + 'h ' + m + 'm';
  if (h > 0) return h + 'h ' + m + 'm';
  return m + 'm ' + Math.floor(s % 60) + 's';
}

function barColorClass(pct) {
  if (pct > 90) return 'bar-red';
  if (pct > 70) return 'bar-amber';
  if (pct > 40) return 'bar-teal';
  return 'bar-green';
}

function valColor(pct) {
  if (pct > 90) return '#f87171';
  if (pct > 70) return '#fbbf24';
  return '#40e0d0';
}

// ── CPU sparkline graph ──
function drawCpuGraph(canvas, history) {
  var ctx = canvas.getContext('2d');
  var w = canvas.width = canvas.clientWidth;
  var h = canvas.height;
  ctx.clearRect(0, 0, w, h);

  if (history.length < 2) return;

  var step = w / (CPU_HISTORY_LEN - 1);
  var startX = w - (history.length - 1) * step;

  // Fill gradient
  var grad = ctx.createLinearGradient(0, 0, 0, h);
  grad.addColorStop(0, 'rgba(64, 224, 208, 0.25)');
  grad.addColorStop(1, 'rgba(64, 224, 208, 0.01)');

  ctx.beginPath();
  ctx.moveTo(startX, h);
  for (var i = 0; i < history.length; i++) {
    var x = startX + i * step;
    var y = h - (history[i] / 100) * h;
    if (i === 0) ctx.lineTo(x, y);
    else {
      var px = startX + (i - 1) * step;
      var py = h - (history[i - 1] / 100) * h;
      var cx = (px + x) / 2;
      ctx.bezierCurveTo(cx, py, cx, y, x, y);
    }
  }
  ctx.lineTo(startX + (history.length - 1) * step, h);
  ctx.closePath();
  ctx.fillStyle = grad;
  ctx.fill();

  // Line
  ctx.beginPath();
  for (var i = 0; i < history.length; i++) {
    var x = startX + i * step;
    var y = h - (history[i] / 100) * h;
    if (i === 0) ctx.moveTo(x, y);
    else {
      var px = startX + (i - 1) * step;
      var py = h - (history[i - 1] / 100) * h;
      var cx = (px + x) / 2;
      ctx.bezierCurveTo(cx, py, cx, y, x, y);
    }
  }
  ctx.strokeStyle = '#40e0d0';
  ctx.lineWidth = 1.5;
  ctx.stroke();

  // Latest point glow
  if (history.length > 0) {
    var lx = startX + (history.length - 1) * step;
    var ly = h - (history[history.length - 1] / 100) * h;
    ctx.beginPath();
    ctx.arc(lx, ly, 3, 0, Math.PI * 2);
    ctx.fillStyle = '#40e0d0';
    ctx.fill();
    ctx.beginPath();
    ctx.arc(lx, ly, 6, 0, Math.PI * 2);
    ctx.fillStyle = 'rgba(64, 224, 208, 0.2)';
    ctx.fill();
  }
}

// ── Memory ring ──
function drawMemRing(canvas, pct) {
  var ctx = canvas.getContext('2d');
  var s = canvas.width;
  ctx.clearRect(0, 0, s, s);

  var cx = s / 2, cy = s / 2, r = s / 2 - 8, lw = 6;
  var start = -Math.PI / 2;
  var end = start + (pct / 100) * Math.PI * 2;

  // Track
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.strokeStyle = '#143040';
  ctx.lineWidth = lw;
  ctx.stroke();

  // Fill arc
  var arcGrad = ctx.createLinearGradient(0, 0, s, s);
  arcGrad.addColorStop(0, '#40e0d0');
  arcGrad.addColorStop(1, '#00c9a7');
  ctx.beginPath();
  ctx.arc(cx, cy, r, start, end);
  ctx.strokeStyle = arcGrad;
  ctx.lineWidth = lw;
  ctx.lineCap = 'round';
  ctx.stroke();

  // Center text
  ctx.fillStyle = '#e0f0f0';
  ctx.font = '700 14px Orbitron, sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(pct.toFixed(0) + '%', cx, cy);
}

// ── Render functions ──

function renderCpu(cpu) {
  var used = parseFloat((100 - cpu.total.idle_percent).toFixed(1));
  cpuHistory.push(used);
  if (cpuHistory.length > CPU_HISTORY_LEN) cpuHistory.shift();

  $('cpu-big').textContent = used.toFixed(1) + '%';
  $('cpu-big').style.color = valColor(used);

  drawCpuGraph($('cpu-graph'), cpuHistory);

  $('cpu-modes').innerHTML =
    modeBadge('#40e0d0', 'usr', cpu.total.user_percent + '%') +
    modeBadge('#0d9488', 'sys', cpu.total.system_percent + '%') +
    modeBadge('#fbbf24', 'iow', cpu.total.iowait_percent + '%') +
    modeBadge('#1a5c6a', 'idl', cpu.total.idle_percent + '%');

  var cores = '';
  cpu.per_core.forEach(function(c) {
    var u = parseFloat((100 - c.idle_percent).toFixed(1));
    cores += '<div class="core-box">' +
      '<div class="core-fill" style="height:' + u + '%"></div>' +
      '<div class="core-name">' + c.cpu + '</div>' +
      '<div class="core-value" style="color:' + valColor(u) + '">' + u + '%</div>' +
      '</div>';
  });
  $('cpu-cores').innerHTML = cores;
}

function modeBadge(color, label, val) {
  return '<div class="cpu-mode-item">' +
    '<span class="cpu-mode-dot" style="background:' + color + '"></span>' +
    label + ' <span class="cpu-mode-val">' + val + '</span></div>';
}

function renderMemory(mem) {
  var pct = (mem.used_bytes / mem.total_bytes) * 100;
  $('mem-big').textContent = pct.toFixed(1) + '%';
  $('mem-big').style.color = valColor(pct);

  drawMemRing($('mem-ring'), pct);

  $('mem-details').innerHTML =
    memRow('used', formatBytes(mem.used_bytes)) +
    memRow('total', formatBytes(mem.total_bytes)) +
    memRow('avail', formatBytes(mem.available_bytes));
}

function memRow(label, val) {
  return '<div class="mem-row-item"><span class="mem-label">' + label +
    '</span><span class="mem-val">' + val + '</span></div>';
}

function renderSystem(sys) {
  $('uptime-badge').textContent = '↑ ' + formatUptime(sys.uptime_seconds);

  var maxLoad = Math.max(sys.load_1, sys.load_5, sys.load_15, 1);
  var scale = Math.max(4, Math.ceil(maxLoad));

  $('system').innerHTML =
    '<div class="load-bars">' +
      loadBar('1m', sys.load_1, scale) +
      loadBar('5m', sys.load_5, scale) +
      loadBar('15m', sys.load_15, scale) +
    '</div>';
}

function sysItem(label, val) {
  return '<div class="sys-item"><span class="sys-label">' + label +
    '</span><span class="sys-val">' + val + '</span></div>';
}

function loadBar(label, val, max) {
  var pct = Math.min((val / max) * 100, 100);
  var color = val > 4 ? '#f87171' : val > 2 ? '#fbbf24' : val > 1 ? '#0d9488' : '#34d399';
  return '<div class="load-bar-row">' +
    '<span class="load-bar-label">' + label + '</span>' +
    '<div class="load-bar-track"><div class="load-bar-fill" style="width:' + pct + '%;background:' + color + '"></div></div>' +
    '<span class="load-bar-val">' + val.toFixed(2) + '</span>' +
    '</div>';
}

function renderNetwork(ifaces) {
  var html = '';
  ifaces.forEach(function(iface) {
    html += '<div class="iface-block">' +
      '<div class="iface-name">' + iface.name + '</div>' +
      '<div class="iface-stats">' +
        ifStat('rx', formatBytes(iface.rx_bytes)) +
        ifStat('tx', formatBytes(iface.tx_bytes)) +
      '</div></div>';
  });
  $('network').innerHTML = html;
}

function ifStat(label, val) {
  return '<div class="iface-stat"><span class="iface-stat-label">' + label +
    '</span><span class="iface-stat-val">' + val + '</span></div>';
}

function renderDisk(disks) {
  var html = '';
  disks.forEach(function(d) {
    if (d.total_bytes === 0) return;
    var pct = (d.used_bytes / d.total_bytes) * 100;
    var mount = d.mount_point;
    if (mount.length > 20) mount = '...' + mount.slice(-18);

    html += '<div class="disk-item">' +
      '<div class="disk-info">' +
        '<span class="disk-mount" title="' + d.mount_point + '">' + mount + '</span>' +
        '<span class="disk-info-pct" style="color:' + valColor(pct) + '">' + pct.toFixed(1) + '%</span>' +
      '</div>' +
      '<div class="disk-bar"><div class="disk-bar-fill ' + barColorClass(pct) + '" style="width:' + pct + '%"></div></div>' +
      '</div>';
  });
  $('disk').innerHTML = html;
}

// ── Update loop ──
var fails = 0;

function update() {
  fetch('/json')
    .then(function(r) { return r.json(); })
    .then(function(data) {
      fails = 0;
      $('status').className = 'status';
      $('status-text').textContent = 'live';
      $('error-banner').style.display = 'none';

      if (data.cpu) { latestData.cpu = data.cpu; renderCpu(data.cpu); }
      if (data.memory) { latestData.memory = data.memory; renderMemory(data.memory); }
      if (data.system) { latestData.system = data.system; renderSystem(data.system); }
      if (data.network) { latestData.network = data.network; renderNetwork(data.network); }
      if (data.disk) { latestData.disk = data.disk; renderDisk(data.disk); }
    })
    .catch(function(e) {
      fails++;
      $('status').className = 'status error';
      $('status-text').textContent = 'disconnected';
      if (fails > 2) {
        $('error-banner').style.display = 'block';
        $('error-banner').textContent = 'Connection lost - retrying...';
      }
    });
}

// ── Modal system ──
var latestData = {};

function openModal(title, contentFn) {
  $('modal-title').textContent = title;
  $('modal-body').innerHTML = contentFn();
  $('modal-backdrop').classList.add('open');
}

function closeModal() {
  $('modal-backdrop').classList.remove('open');
}

$('modal-backdrop').addEventListener('click', function(e) {
  if (e.target === $('modal-backdrop')) closeModal();
});

$('modal-close').addEventListener('click', closeModal);

document.addEventListener('keydown', function(e) {
  if (e.key === 'Escape') closeModal();
});

// ── Expanded card renderers ──

function mRow(label, val) {
  return '<div class="m-row"><span class="m-label">' + label + '</span><span class="m-val">' + val + '</span></div>';
}

function mBar(pct) {
  var cls = pct > 90 ? 'bar-red' : pct > 70 ? 'bar-amber' : pct > 40 ? 'bar-teal' : 'bar-green';
  return '<div class="m-bar-track"><div class="m-bar-fill ' + cls + '" style="width:' + pct + '%"></div></div>';
}

function expandCpu() {
  var cpu = latestData.cpu;
  if (!cpu) return '';
  var used = (100 - cpu.total.idle_percent).toFixed(1);

  var html = '<span class="big-value" style="color:' + valColor(parseFloat(used)) + '">' + used + '% used</span>';

  html += '<div class="modal-section"><div class="modal-section-title">Total Breakdown</div>' +
    mRow('user', cpu.total.user_percent + '%') +
    mRow('system', cpu.total.system_percent + '%') +
    mRow('iowait', cpu.total.iowait_percent + '%') +
    mRow('idle', cpu.total.idle_percent + '%') +
    mBar(parseFloat(used)) +
    '</div>';

  html += '<div class="modal-section"><div class="modal-section-title">Per Core</div><div class="m-cores-grid">';
  cpu.per_core.forEach(function(c) {
    var u = (100 - c.idle_percent).toFixed(1);
    html += '<div class="m-core">' +
      '<div class="m-core-fill" style="height:' + u + '%"></div>' +
      '<div class="m-core-name">' + c.cpu + '</div>' +
      '<div class="m-core-val" style="color:' + valColor(parseFloat(u)) + '">' + u + '%</div>' +
      '<div class="m-core-details">usr ' + c.user_percent + '% / sys ' + c.system_percent + '%</div>' +
      '</div>';
  });
  html += '</div></div>';

  html += '<div class="modal-section"><div class="modal-section-title">History (last 2 min)</div>' +
    '<canvas id="modal-cpu-graph" width="650" height="120"></canvas></div>';

  setTimeout(function() {
    var c = document.getElementById('modal-cpu-graph');
    if (c) drawCpuGraph(c, cpuHistory);
  }, 50);

  return html;
}

function expandMemory() {
  var mem = latestData.memory;
  if (!mem) return '';
  var pct = ((mem.used_bytes / mem.total_bytes) * 100).toFixed(1);

  var html = '<span class="big-value" style="color:' + valColor(parseFloat(pct)) + '">' + pct + '% used</span>';
  html += mBar(parseFloat(pct));

  html += '<div class="modal-section"><div class="modal-section-title">Breakdown</div>' +
    mRow('total', formatBytes(mem.total_bytes)) +
    mRow('used', formatBytes(mem.used_bytes)) +
    mRow('free', formatBytes(mem.free_bytes)) +
    mRow('available', formatBytes(mem.available_bytes)) +
    mRow('buffers', formatBytes(mem.buffers_bytes)) +
    mRow('cached', formatBytes(mem.cached_bytes)) +
    '</div>';

  var swapUsed = mem.swap_total_bytes - mem.swap_free_bytes;
  var swapPct = mem.swap_total_bytes > 0 ? ((swapUsed / mem.swap_total_bytes) * 100).toFixed(1) : '0.0';

  html += '<div class="modal-section"><div class="modal-section-title">Swap</div>' +
    mRow('total', formatBytes(mem.swap_total_bytes)) +
    mRow('used', formatBytes(swapUsed)) +
    mRow('free', formatBytes(mem.swap_free_bytes)) +
    mBar(parseFloat(swapPct)) +
    '</div>';

  return html;
}

function expandSystem() {
  var sys = latestData.system;
  if (!sys) return '';

  var html = '<span class="big-value" style="color:var(--aqua)">' + formatUptime(sys.uptime_seconds) + '</span>';

  html += '<div class="modal-section"><div class="modal-section-title">Load Average</div>' +
    mRow('1 minute', sys.load_1.toFixed(2)) +
    mRow('5 minutes', sys.load_5.toFixed(2)) +
    mRow('15 minutes', sys.load_15.toFixed(2)) +
    '</div>';

  html += '<div class="modal-section"><div class="modal-section-title">Processes</div>' +
    mRow('running', sys.processes_running) +
    mRow('total', sys.processes_total) +
    '</div>';

  return html;
}

function expandNetwork() {
  var ifaces = latestData.network;
  if (!ifaces) return '';

  var html = '';
  ifaces.forEach(function(iface) {
    html += '<div class="m-iface">' +
      '<div class="m-iface-name">' + iface.name + '</div>' +
      '<div class="m-iface-grid">' +
        mRow('rx bytes', formatBytes(iface.rx_bytes)) +
        mRow('tx bytes', formatBytes(iface.tx_bytes)) +
        mRow('rx packets', iface.rx_packets.toLocaleString()) +
        mRow('tx packets', iface.tx_packets.toLocaleString()) +
        mRow('rx errors', iface.rx_errors) +
        mRow('tx errors', iface.tx_errors) +
        mRow('rx dropped', iface.rx_dropped) +
        mRow('tx dropped', iface.tx_dropped) +
      '</div></div>';
  });

  return html;
}

function expandDisk() {
  var disks = latestData.disk;
  if (!disks) return '';

  var html = '';
  disks.forEach(function(d) {
    if (d.total_bytes === 0) return;
    var pct = ((d.used_bytes / d.total_bytes) * 100).toFixed(1);

    html += '<div class="m-disk-item">' +
      '<div class="m-disk-header">' +
        '<span class="m-disk-mount">' + d.mount_point + '</span>' +
        '<span class="m-disk-pct" style="color:' + valColor(parseFloat(pct)) + '">' + pct + '%</span>' +
      '</div>' +
      '<div class="m-disk-device">' + d.device + '</div>' +
      '<div class="m-disk-sizes">used ' + formatBytes(d.used_bytes) + ' / total ' + formatBytes(d.total_bytes) + ' / free ' + formatBytes(d.free_bytes) + '</div>' +
      mBar(parseFloat(pct)) +
      '</div>';
  });

  return html;
}

// ── Card click handlers ──
document.querySelector('.cpu-card').addEventListener('click', function() {
  openModal('◇ CPU Details', expandCpu);
});

document.querySelector('.mem-card').addEventListener('click', function() {
  openModal('◇ Memory Details', expandMemory);
});

document.querySelector('.sys-card').addEventListener('click', function() {
  openModal('◇ System Details', expandSystem);
});

document.querySelector('.net-card').addEventListener('click', function() {
  openModal('◇ Network Details', expandNetwork);
});

document.querySelector('.disk-card').addEventListener('click', function() {
  openModal('◇ Disk Details', expandDisk);
});

update();
setInterval(update, 2000);