import init, { send_command, demo_status } from './pkg/bevy_flash_demo.js';

const $ = id => document.getElementById(id);
let page = 'animation', index = 0, started = false, signature = '', selectedClip = '', failed = false, seeking = false;
const send = action => { if (started) send_command(JSON.stringify(action)); };
const options = (node, values, selected) => {
  node.replaceChildren(...values.map((value, i) => {
    const option = document.createElement('option');
    option.textContent = value; option.value = value;
    if (selected === value || selected === i) option.selected = true;
    return option;
  }));
};
function navigate(nextPage, nextIndex = 0) {
  page = nextPage; index = nextIndex; signature = ''; selectedClip = '';
  $('disabled').checked = false;
  $('loading').hidden = false; $('loading-text').textContent = '正在加载资源与依赖…'; $('retry').hidden = true;
  send({action:'navigate',page,index});
}
document.querySelectorAll('[data-page]').forEach(button => button.addEventListener('click', () => navigate(button.dataset.page)));
$('asset').addEventListener('change', () => navigate(page, $('asset').selectedIndex));
$('clip').addEventListener('change', () => selectedClip = $('clip').value);
$('play').addEventListener('click', () => send({action:'play', name:$('clip').value, mode:$('mode').value}));
$('fallback').addEventListener('change', () => send({action:'fallback', name:$('fallback').value}));
$('pause').addEventListener('click', () => send({action:'pause'}));
$('reset').addEventListener('click', () => send({action:'reset'}));
$('speed').addEventListener('change', () => send({action:'speed', value:Number($('speed').value)}));
$('zoom').addEventListener('change', () => send({action:'zoom', value:Number($('zoom').value)}));
$('zoom-reset').addEventListener('click', () => send({action:'zoom', value:1}));
$('seek').addEventListener('input', () => send({action:'seek', frame:Number($('seek').value)}));
$('seek').addEventListener('pointerdown', () => seeking = true);
window.addEventListener('pointerup', () => seeking = false);
$('seek').addEventListener('blur', () => seeking = false);
$('disabled').addEventListener('change', () => send({action:'disabled', value:$('disabled').checked}));
$('diagnostics').addEventListener('change', () => send({action:'diagnostics', value:$('diagnostics').checked}));
$('retry').addEventListener('click', () => { signature = ''; send({action:'retry'}); });
$('fullscreen').addEventListener('click', async () => {
  try { if (document.fullscreenElement) await document.exitFullscreen(); else await $('preview').requestFullscreen(); }
  catch (error) { $('notice').textContent = `无法进入全屏：${error.message}`; }
});

function fatal(message) {
  failed = true; $('loading').hidden = false;
  $('loading-text').textContent = message;
  document.querySelector('.spinner').hidden = true;
  $('state').textContent = '初始化失败';
}
function update() {
  if (failed || !started) return;
  const state = JSON.parse(demo_status());
  if (!state.page) return;
  page = state.page; index = state.index;
  document.querySelectorAll('[data-page]').forEach(button => {
    button.setAttribute('aria-pressed', String(button.dataset.page === page)); button.disabled = false;
  });
  const nextSignature = JSON.stringify([page,index,state.ready,state.assets,state.clips,state.skins?.map(s => [s[0],s[1]])]);
  if (signature !== nextSignature) {
    signature = nextSignature;
    options($('asset'), state.assets, index);
    options($('clip'), state.clips, selectedClip || state.clip);
    options($('fallback'), state.clips, state.fallback);
    $('skins').replaceChildren(...state.skins.map(([slot,variants,selected]) => {
      const label = document.createElement('label'); label.textContent = slot;
      const select = document.createElement('select'); select.dataset.slot = slot;
      options(select,variants,selected);
      select.addEventListener('change', () => send({action:'skin',slot,variant:select.value}));
      label.append(select); return label;
    }));
  }
  $('asset').disabled = state.assets.length < 2;
  const animation = page === 'animation' || page === 'skins';
  $('animation-controls').hidden = !animation || !state.ready;
  $('playback-controls').hidden = (!animation && page !== 'ui') || !state.ready;
  $('skin-controls').hidden = page !== 'skins' || !state.ready;
  $('seek-label').hidden = !animation;
  $('disable-label').hidden = page !== 'buttons' || !state.ready;
  $('loading').hidden = state.ready && !state.error;
  $('loading-text').textContent = state.error ? `加载失败：${state.error}` : '正在加载资源与依赖…';
  $('retry').hidden = !state.error;
  document.querySelector('.spinner').hidden = Boolean(state.error);
  $('notice').textContent = state.notice;
  $('pause').textContent = state.playing ? '暂停' : '继续';
  $('speed').value = String(state.speed);
  $('zoom').value = String(state.zoom);
  $('render-scale').textContent = `资源倍率 ${state.render_scale.toFixed(2)}×`;
  $('fallback').value = state.fallback;
  $('frame').textContent = `${state.frame + 1} / ${state.frames}`;
  if (!seeking) { $('seek').max = Math.max(0,state.frames-1); $('seek').value = state.frame; }
  $('state').textContent = state.ready ? `${state.clip || (page === 'ui' ? '矢量 UI' : '原生按钮')} · ${state.terminal ? '终态锁定' : state.playing ? '播放中' : '已暂停'}` : '加载中';
  $('clicks').textContent = page === 'buttons' ? `按下次数 ${state.clicks}` : '';
  $('fps').textContent = state.fps == null ? '关闭' : `${state.fps.toFixed(1)} FPS`;
  $('events').replaceChildren(...(state.events.length ? state.events : ['暂无事件']).map(text => {
    const li = document.createElement('li'); li.textContent = text; return li;
  }));
}

document.querySelectorAll('[data-page]').forEach(button => button.disabled = true);
try {
  if (!window.isSecureContext) throw new Error('WebGPU 需要 HTTPS，或使用 localhost 本地预览。');
  if (!navigator.gpu) throw new Error('当前浏览器未提供 WebGPU，请启用 WebGPU 后重试。此演示不使用 WebGL。');
  if (!await navigator.gpu.requestAdapter()) throw new Error('没有可用的 WebGPU 适配器。');
  await init();
  started = true;
  setInterval(update, 150);
} catch (error) {
  fatal(error.message || String(error));
}
