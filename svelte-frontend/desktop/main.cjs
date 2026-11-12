const { app, BrowserWindow, ipcMain, Menu, shell } = require('electron');
const fs = require('node:fs/promises');
const path = require('node:path');

const settingsPath = () => path.join(app.getPath('userData'), 'settings.json');
let serverUrl = '';
let serverWindow;
let connectionWindow;

function normalizeServerUrl(value) {
  let url;
  try {
    url = new URL(String(value).trim());
  } catch {
    throw new Error('Enter a valid server address.');
  }
  if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) {
    throw new Error('Use an HTTP or HTTPS server address without credentials.');
  }
  if (url.pathname !== '/' || url.search || url.hash) {
    throw new Error('Enter the server address without a path, query, or fragment.');
  }
  return url.origin;
}

function createServerWindow() {
  serverWindow = new BrowserWindow({
    width: 1440,
    height: 920,
    minWidth: 375,
    minHeight: 730,
    backgroundColor: '#101311',
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });

  serverWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: 'deny' };
  });
  serverWindow.webContents.on('will-navigate', (event, target) => {
    const targetUrl = new URL(target);
    if (targetUrl.origin === serverUrl || targetUrl.origin === 'https://accounts.spotify.com') return;
    event.preventDefault();
    shell.openExternal(target);
  });
  serverWindow.on('closed', () => { serverWindow = null; });
  serverWindow.loadURL(serverUrl);
}

function openConnectionWindow() {
  if (connectionWindow && !connectionWindow.isDestroyed()) {
    connectionWindow.focus();
    return;
  }
  connectionWindow = new BrowserWindow({
    width: 520,
    height: 440,
    resizable: false,
    backgroundColor: '#101311',
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });
  connectionWindow.removeMenu();
  connectionWindow.on('closed', () => { connectionWindow = null; });
  connectionWindow.loadFile(path.join(__dirname, 'connect.html'));
}

function setApplicationMenu() {
  Menu.setApplicationMenu(Menu.buildFromTemplate([
    {
      label: 'Wavelength',
      submenu: [
        { label: 'Change server...', click: openConnectionWindow },
        { type: 'separator' },
        { role: 'quit' },
      ],
    },
    {
      label: 'View',
      submenu: [
        { role: 'reload' },
        { role: 'togglefullscreen' },
      ],
    },
  ]));
}

ipcMain.handle('get-server-url', () => serverUrl);
ipcMain.handle('connect-server', async (event, value) => {
  if (!connectionWindow || event.sender !== connectionWindow.webContents) throw new Error('Connection form is unavailable.');
  serverUrl = normalizeServerUrl(value);
  await fs.mkdir(app.getPath('userData'), { recursive: true });
  await fs.writeFile(settingsPath(), JSON.stringify({ serverUrl }), 'utf8');
  if (serverWindow && !serverWindow.isDestroyed()) serverWindow.close();
  connectionWindow.close();
  createServerWindow();
  return serverUrl;
});

app.whenReady().then(async () => {
  setApplicationMenu();
  try {
    const savedSettings = JSON.parse(await fs.readFile(settingsPath(), 'utf8'));
    serverUrl = normalizeServerUrl(savedSettings.serverUrl);
  } catch { }

  if (serverUrl) createServerWindow();
  else openConnectionWindow();

  app.on('activate', () => {
    if (!serverWindow && !connectionWindow) {
      if (serverUrl) createServerWindow();
      else openConnectionWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});