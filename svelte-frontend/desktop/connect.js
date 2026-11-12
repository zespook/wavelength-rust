const form = document.querySelector('#connect-form');
const input = document.querySelector('#server-url');
const button = form.querySelector('button');
const status = document.querySelector('#status');

window.wavelengthDesktop.getServerUrl().then((serverUrl) => {
  if (serverUrl) input.value = serverUrl;
});

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  button.disabled = true;
  status.textContent = '';
  try {
    await window.wavelengthDesktop.connectToServer(input.value);
  } catch (error) {
    status.textContent = error.message || 'Unable to connect to the server.';
    button.disabled = false;
  }
});