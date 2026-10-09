import init from './pkg/bevy_flash_demo.js';
const loading = document.getElementById('loading');
const canvas = document.getElementById('bevy');
canvas.addEventListener('contextmenu', event => event.preventDefault());
if (!navigator.gpu) {
  loading.textContent = 'WebGPU is required. Open this page in a WebGPU-capable browser over HTTPS.';
} else {
  try {
    await init();
    loading.remove();
    canvas.focus();
  } catch (error) {
    console.error(error);
    loading.textContent = `Failed to start: ${error.message ?? error}`;
  }
}
