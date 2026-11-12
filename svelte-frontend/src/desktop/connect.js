import { invoke } from '@tauri-apps/api/core';

const form = document.querySelector('#connect-form');
const input = document.querySelector('#server-url');
const button = form.querySelector('button');
const status = document.querySelector('#status');

invoke('get_server_url').then((serverUrl) => {
  if (serverUrl) input.value = serverUrl;
}).catch(() => { });

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  button.disabled = true;
  status.textContent = '';
  try {
    await invoke('connect_server', { serverUrl: input.value });
  } catch (error) {
    status.textContent = String(error || 'Unable to connect to the server.');
    button.disabled = false;
  }
});