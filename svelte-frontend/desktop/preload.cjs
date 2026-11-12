const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('wavelengthDesktop', {
  getServerUrl: () => ipcRenderer.invoke('get-server-url'),
  connectToServer: (serverUrl) => ipcRenderer.invoke('connect-server', serverUrl),
});