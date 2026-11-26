// The launcher's window. All the work (shader step, starting the game, reading its log) is done on
// the Rust side: this only shows what `poll` reports and passes clicks on.
(() => {
  const tauri = window.__TAURI__;
  const invoke = tauri ? tauri.core.invoke : preview;
  const $ = (id) => document.getElementById(id);
  const root = document.documentElement;
  const calm = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const EASE = 'cubic-bezier(0.2, 0.8, 0.2, 1)';

  let settings = { GamePath: '', LitShade: true, Level: 0.6, Fast: true, Rays: false, RaysFull: false, WaitPipelines: true };
  let pictures = [];
  let pictureAt = 0;
  let front = $('artA');
  let back = $('artB');
  let phase = 'idle';

  // Outside the app (a plain browser, for looking at the layout) there is nothing to call:
  // show a fixed state, picked with #idle, #prep or #game.
  function preview(name) {
    const at = (location.hash || '#idle').slice(1);
    if (name === 'load') {
      return Promise.resolve({ settings, art: ['/art/art1.png', '/art/art2.png'], build: Date.now() / 1000 });
    }
    if (name === 'poll') {
      const views = {
        idle: { step: 'Ready', hint: '', progress: 0, detail: '', thirdLabel: 'Last run', thirdValue: '12 min' },
        prep: { step: 'Preparing shaders', hint: 'Done once for each build. Seconds if they were translated before, about 15 minutes if not.', progress: 35, detail: '3,000 of 8,634', thirdLabel: 'Last run', thirdValue: '–' },
        game: { step: 'Running', hint: '', progress: 100, detail: '', thirdLabel: 'fps', thirdValue: '5.8' },
      };
      return Promise.resolve({ phase: at, shadersReady: at !== 'prep', shaderFiles: true, freeGb: 9.1, totalGb: 16, lowMemory: false, ...views[at] });
    }
    return Promise.resolve(null);
  }

  // ---- motion helpers

  // Words that change slide out and the new ones slide in.
  function swapText(el, text) {
    if (el.dataset.text === text) return;
    const first = el.dataset.text === undefined;
    el.dataset.text = text;
    if (first || calm || phase === 'game' && el.id !== 'step' && el.id !== 'playText') { el.textContent = text; return; }
    el.animate([{ opacity: 1, transform: 'none' }, { opacity: 0, transform: 'translateY(-5px)' }], { duration: 130, easing: 'ease-in', fill: 'forwards' })
      .finished.then(() => {
        el.textContent = el.dataset.text;
        el.animate([{ opacity: 0, transform: 'translateY(6px)' }, { opacity: 1, transform: 'none' }], { duration: 320, easing: EASE, fill: 'forwards' });
      });
  }

  // The percentage counts up instead of jumping.
  let shownPct = 0, wantPct = 0, counting = false;
  function countTo(pct) {
    wantPct = pct;
    if (counting) return;
    counting = true;
    const step = () => {
      shownPct += (wantPct - shownPct) * 0.12;
      if (Math.abs(wantPct - shownPct) < 0.4) shownPct = wantPct;
      $('playPct').textContent = phase === 'prep' ? `${Math.round(shownPct)}%` : '';
      if (shownPct !== wantPct && phase === 'prep') requestAnimationFrame(step); else counting = false;
    };
    requestAnimationFrame(step);
  }

  // The picture leans a little away from the mouse. The loop sleeps when the mouse rests, and
  // nothing moves at all while the game runs: the game needs every bit of the processor.
  let aimX = 0, aimY = 0, atX = 0, atY = 0, leaning = false;
  function lean() {
    atX += (aimX - atX) * 0.07;
    atY += (aimY - atY) * 0.07;
    root.style.setProperty('--px', (atX * -16).toFixed(2) + 'px');
    root.style.setProperty('--py', (atY * -10).toFixed(2) + 'px');
    if (phase !== 'game' && (Math.abs(aimX - atX) > 0.002 || Math.abs(aimY - atY) > 0.002)) requestAnimationFrame(lean);
    else leaning = false;
  }
  if (!calm) {
    window.addEventListener('mousemove', (e) => {
      aimX = e.clientX / window.innerWidth - 0.5;
      aimY = e.clientY / window.innerHeight - 0.5;
      if (!leaning && phase !== 'game') { leaning = true; requestAnimationFrame(lean); }
    });
  }

  function showPicture(src) {
    const img = back.querySelector('img');
    img.onload = () => {
      back.classList.add('on');
      front.classList.remove('on');
      [front, back] = [back, front];
      document.body.classList.add('ready');
    };
    img.src = src;
  }

  function render(v) {
    phase = v.phase;
    document.body.dataset.phase = v.phase;
    swapText($('step'), v.step);
    swapText($('hint'), v.hint);

    // one quiet line of facts under the step
    const facts = [];
    if (v.phase === 'prep') {
      if (v.detail) facts.push(v.detail);
    } else {
      if (v.phase === 'idle' && settings.Rays) facts.push('Ray tracing on (experimental)');
      else if (v.phase === 'idle') facts.push(v.shadersReady ? 'Shaders ready' : v.shaderFiles ? 'Every shader is prepared on first play' : 'Shaders are made as you play');
      facts.push(`${v.freeGb.toFixed(1)} of ${v.totalGb} GB free`);
      if (v.thirdValue && v.thirdValue !== '–') {
        facts.push(v.thirdLabel === 'fps' ? `${v.thirdValue} fps` : `${v.thirdLabel} ${v.thirdValue}`);
      }
    }
    $('meta').textContent = facts.join('  ·  ');

    swapText($('playText'), v.phase === 'idle' ? 'Play' : v.phase === 'prep' ? 'Preparing' : 'Running');
    if (v.phase === 'prep') countTo(v.progress); else { shownPct = 0; $('playPct').textContent = ''; }
    $('prog').style.width = `${v.progress}%`;
  }

  let pollTimer = 0;
  async function tick() {
    try { render(await invoke('poll')); } catch (e) { console.error(e); }
    // while the game runs, look less often
    clearTimeout(pollTimer);
    pollTimer = setTimeout(tick, phase === 'game' ? 2000 : 700);
  }

  function showSettings() {
    for (const [id, on] of [['tglLit', settings.LitShade], ['tglFast', settings.Fast], ['tglRays', settings.Rays], ['tglFull', settings.RaysFull], ['tglWait', settings.WaitPipelines]]) {
      $(id).classList.toggle('on', !!on);
      $(id).setAttribute('aria-checked', String(!!on));
    }
    $('rowFull').classList.toggle('off', !settings.Rays);
    $('level').value = settings.Level;
    $('level').disabled = !settings.LitShade;
    $('level').style.setProperty('--at', `${((settings.Level - 0.2) / 1.0) * 100}%`);
    $('levelOut').textContent = Number(settings.Level).toFixed(1);
    const path = settings.GamePath || '';
    $('game').textContent = path.split(/[\\/]/).pop() || 'Not picked';
    $('game').title = path;
  }
  const save = () => invoke('save', { settings });
  const card = (open) => document.body.classList.toggle('setup-open', open);

  async function start() {
    const init = await invoke('load');
    settings = Object.assign(settings, init.settings);
    showSettings();
    $('build').textContent = init.build
      ? 'NVIDIA build, ' + new Date(init.build * 1000).toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' })
      : 'No build found';

    pictures = init.art || [];
    if (pictures.length) {
      showPicture(pictures[0]);
      if (pictures.length > 1) {
        setInterval(() => {
          if (phase === 'game' || document.hidden) return;
          pictureAt = (pictureAt + 1) % pictures.length;
          showPicture(pictures[pictureAt]);
        }, 16000);
      }
    }
    // with no picture (or a slow one) the words still come in
    setTimeout(() => document.body.classList.add('ready'), pictures.length ? 900 : 60);
    // after the intro its rules let go (see style.css)
    setTimeout(() => document.body.classList.add('settled'), 4200);

    tick();
  }

  // ---- clicks
  $('bar').addEventListener('mousedown', (e) => { if (e.button === 0 && !e.target.closest('button')) invoke('win_drag'); });
  $('btnMin').addEventListener('click', () => invoke('win_min'));
  $('btnClose').addEventListener('click', () => {
    document.body.classList.add('leaving');
    setTimeout(() => invoke('win_close'), calm ? 0 : 190);
  });
  $('play').addEventListener('click', async () => { card(false); await invoke('play'); tick(); });
  $('tglLit').addEventListener('click', () => { settings.LitShade = !settings.LitShade; showSettings(); save(); });
  $('tglFast').addEventListener('click', () => { settings.Fast = !settings.Fast; showSettings(); save(); });
  $('tglWait').addEventListener('click', () => { settings.WaitPipelines = !settings.WaitPipelines; showSettings(); save(); });
  $('tglRays').addEventListener('click', () => { settings.Rays = !settings.Rays; showSettings(); save(); });
  $('tglFull').addEventListener('click', () => { settings.RaysFull = !settings.RaysFull; showSettings(); save(); });
  $('level').addEventListener('input', (e) => { settings.Level = Math.round(e.target.value * 10) / 10; showSettings(); });
  $('level').addEventListener('change', save);
  $('btnSetup').addEventListener('click', () => card(!document.body.classList.contains('setup-open')));
  $('scrim').addEventListener('click', () => card(false));
  $('btnPick').addEventListener('click', async () => {
    const picked = await invoke('pick_game');
    if (picked) { settings.GamePath = picked; showSettings(); }
  });
  $('btnPrep').addEventListener('click', async () => { card(false); await invoke('prepare'); tick(); });
  $('btnKeys').addEventListener('click', () => invoke('open_controls'));
  $('btnLog').addEventListener('click', () => invoke('open_log'));
  for (const id of ['tglLit', 'tglFast', 'tglRays', 'tglFull', 'tglWait', 'btnPrep', 'btnPick', 'level']) $(id).classList.add('busy-off');

  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') card(false);
    if (e.key === 'Enter' && phase === 'idle' && !document.body.classList.contains('setup-open')) $('play').click();
    // no reload, no page zoom, no find bar: this is an app window
    if (e.key === 'F5' || (e.ctrlKey && ['r', 'f', 'p', '+', '-', '='].includes(e.key.toLowerCase()))) e.preventDefault();
  });
  window.addEventListener('contextmenu', (e) => e.preventDefault());
  window.addEventListener('hashchange', tick);

  start();
})();
