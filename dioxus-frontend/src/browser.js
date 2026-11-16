
export function getLocal(key) { return localStorage.getItem(key); }
export function setLocal(key, value) { localStorage.setItem(key, value); return true; }
export function removeLocal(key) { localStorage.removeItem(key); return true; }
export function installHoverMarquee() {
  const root = document.documentElement;
  if (root.dataset.hoverMarqueeInstalled === "true") return true;
  root.dataset.hoverMarqueeInstalled = "true";

  let active;
  let overlay;
  let originalOpacity;
  const updateOverlay = () => {
    if (!active || !overlay) return;
    const bounds = active.getBoundingClientRect();
    Object.assign(overlay.style, {
      left: `${bounds.left}px`,
      top: `${bounds.top}px`,
      width: `${active.clientWidth}px`,
      height: `${active.clientHeight}px`,
    });
  };
  const stop = () => {
    if (!active) return;
    active.style.opacity = originalOpacity;
    overlay?.remove();
    active = null;
    overlay = null;
    originalOpacity = null;
    window.removeEventListener("scroll", updateOverlay, true);
    window.removeEventListener("resize", updateOverlay);
  };
  const candidateFrom = target => {
    let element = target instanceof Element ? target : target?.parentElement;
    const card = element?.closest(".playlist-card");
    const cardTitle = card?.querySelector(".playlist-card-info h2");
    if (cardTitle instanceof HTMLElement) {
      const style = getComputedStyle(cardTitle);
      if (
        style.textOverflow === "ellipsis" &&
        ["hidden", "clip"].includes(style.overflowX) &&
        cardTitle.scrollWidth > cardTitle.clientWidth + 3 &&
        cardTitle.textContent?.trim()
      ) return cardTitle;
    }
    while (element && element !== document.body) {
      const style = getComputedStyle(element);
      if (
        element instanceof HTMLElement &&
        style.textOverflow === "ellipsis" &&
        ["hidden", "clip"].includes(style.overflowX) &&
        element.scrollWidth > element.clientWidth + 3 &&
        element.textContent?.trim()
      ) return element;
      element = element.parentElement;
    }
    return null;
  };
  const start = target => {
    if (matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const element = candidateFrom(target);
    if (!element || element === active || element.closest(".player-bar")) return;
    stop();

    const text = element.textContent.replace(/\s+/g, " ").trim();
    if (!text) return;
    const style = getComputedStyle(element);
    const bounds = element.getBoundingClientRect();
    const gap = 24;
    const track = document.createElement("span");
    Object.assign(track.style, {
      display: "flex",
      width: "max-content",
      flex: "0 0 auto",
      animation: `hover-title-marquee ${Math.max(5000, (element.scrollWidth + gap) * 28) / 0.85}ms linear infinite`,
    });
    for (let index = 0; index < 2; index += 1) {
      const copy = document.createElement("span");
      copy.textContent = text;
      Object.assign(copy.style, {
        display: "block",
        flex: "0 0 auto",
        paddingRight: `${gap}px`,
        whiteSpace: "pre",
      });
      track.appendChild(copy);
    }

    overlay = document.createElement("span");
    overlay.setAttribute("aria-hidden", "true");
    overlay.className = "hover-title-marquee";
    Object.assign(overlay.style, {
      position: "fixed",
      left: `${bounds.left}px`,
      top: `${bounds.top}px`,
      width: `${element.clientWidth}px`,
      height: `${element.clientHeight}px`,
      display: "flex",
      alignItems: "center",
      boxSizing: "border-box",
      overflow: "hidden",
      padding: style.padding,
      font: style.font,
      letterSpacing: style.letterSpacing,
      lineHeight: style.lineHeight,
      textAlign: style.textAlign,
      textTransform: style.textTransform,
      color: style.color,
      whiteSpace: "nowrap",
      pointerEvents: "none",
      zIndex: "2147483647",
    });
    overlay.appendChild(track);
    document.body.appendChild(overlay);
    originalOpacity = element.style.opacity;
    element.style.opacity = "0";
    active = element;
    window.addEventListener("scroll", updateOverlay, true);
    window.addEventListener("resize", updateOverlay);
  };
  const onPointerOver = event => {
    if (event.pointerType !== "touch") start(event.target);
  };
  const onPointerOut = event => {
    const activeCard = active?.closest(".playlist-card");
    if (
      active &&
      (
        event.target === active ||
        active.contains(event.target) ||
        (
          event.target instanceof Node &&
          activeCard?.contains(event.target) &&
          !(event.relatedTarget instanceof Node && activeCard.contains(event.relatedTarget))
        )
      ) &&
      !(event.relatedTarget instanceof Node && active.contains(event.relatedTarget))
    ) stop();
  };
  document.addEventListener("pointerover", onPointerOver);
  document.addEventListener("pointerout", onPointerOut);
  return true;
}
export function watchNarrowWindow(callback) {
  const update = () => callback(window.innerWidth < 948);
  window.addEventListener("resize", update);
  update();
  return true;
}
export async function chooseAvatar() {
  return await new Promise((resolve, reject) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "image/*";
    input.addEventListener("cancel", () => resolve(null), { once: true });
    input.onchange = () => {
      const file = input.files?.[0];
      if (!file) {
        resolve(null);
        return;
      }
      if (!file.type.startsWith("image/")) {
        reject(new Error("Choose an image file."));
        return;
      }
      const url = URL.createObjectURL(file);
      const image = new Image();
      image.onload = () => {
        URL.revokeObjectURL(url);
        const scale = Math.min(1, 256 / Math.max(image.naturalWidth, image.naturalHeight));
        const canvas = document.createElement("canvas");
        canvas.width = Math.max(1, Math.round(image.naturalWidth * scale));
        canvas.height = Math.max(1, Math.round(image.naturalHeight * scale));
        const context = canvas.getContext("2d");
        if (!context) {
          reject(new Error("Could not process the selected photo."));
          return;
        }
        context.drawImage(image, 0, 0, canvas.width, canvas.height);
        resolve(canvas.toDataURL("image/jpeg", 0.84));
      };
      image.onerror = () => {
        URL.revokeObjectURL(url);
        reject(new Error("The selected photo could not be opened."));
      };
      image.src = url;
    };
    input.click();
  });
}
export function getRoomInviteCode() {
  const code = new URLSearchParams(window.location.search).get("party");
  return code || null;
}
export async function copyRoomCode(code) {
  await navigator.clipboard.writeText(code);
  return true;
}
export async function copyRoomInvite(code) {
  const invite = new URL(window.location.href);
  invite.searchParams.set("party", code);
  await navigator.clipboard.writeText(invite.toString());
  return true;
}
function relativeLuminance(color) {
  const channels = color.slice(1).match(/.{2}/g);
  if (!channels || channels.length !== 3) return null;
  const [red, green, blue] = channels.map(channel => {
    const value = parseInt(channel, 16) / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return red * 0.2126 + green * 0.7152 + blue * 0.0722;
}
function contrastRatio(first, second) {
  return (Math.max(first, second) + 0.05) / (Math.min(first, second) + 0.05);
}
export function setAppearance(theme, accent, background) {
  const root = document.documentElement;
  root.dataset.theme = theme === "system"
    ? (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light")
    : theme;
  root.style.setProperty("--lime", accent);
  const accentLuminance = relativeLuminance(accent);
  if (accentLuminance !== null) {
    const lightInk = "#fff5d2";
    const darkInk = "#171912";
    const lightInkLuminance = relativeLuminance(lightInk);
    const darkInkLuminance = relativeLuminance(darkInk);
    if (lightInkLuminance !== null && darkInkLuminance !== null) {
      root.style.setProperty(
        "--accent-ink",
        contrastRatio(accentLuminance, lightInkLuminance) >= contrastRatio(accentLuminance, darkInkLuminance)
          ? lightInk
          : darkInk
      );
    }
  }
  if (background) {
    const channels = background.slice(1).match(/.{2}/g).map(channel =>
      parseInt(channel, 16) / 255
    ).map(channel => channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4);
    const lightBackground = channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722 > 0.179;
    const foreground = lightBackground ? "#1c2019" : "#edf0e8";
    const panelTint = lightBackground ? "#ffffff" : "#000000";
    root.style.setProperty("--surface-page", background);
    root.style.setProperty("--panel", `color-mix(in srgb, ${background} 92%, ${panelTint})`);
    root.style.setProperty("--panel-hi", `color-mix(in srgb, ${background} 84%, ${panelTint})`);
    root.style.setProperty("--ink", foreground);
    root.style.setProperty("--muted", `color-mix(in srgb, ${foreground} 82%, ${background})`);
    root.style.setProperty("--quiet", `color-mix(in srgb, ${foreground} 62%, ${background})`);
    root.style.setProperty("--line", `color-mix(in srgb, ${foreground} 16%, transparent)`);
  } else {
    ["--surface-page", "--panel", "--panel-hi", "--ink", "--muted", "--quiet", "--line"]
      .forEach(property => root.style.removeProperty(property));
  }
  return true;
}
export function isDesktopWindow() {
  return false;
}
export async function checkForUpdates() {
  return null;
}
export async function installUpdate() {
  throw new Error("App updates are only available in a desktop build.");
}
export async function windowAction() {
  throw new Error("Native window controls are unavailable.");
}
export function installAppBehaviors() {
  const root = document.documentElement;
  if (root.dataset.appBehaviorsInstalled === "true") return true;

  const key = "wavelength-ui-scale";
  const saved = localStorage.getItem(key);
  const initialScale = saved === null ? 1 : Number(saved);
  if (!Number.isFinite(initialScale) || initialScale < 0.7 || initialScale > 1.5) {
    throw new Error("Saved interface scale is invalid.");
  }
  let scale = Math.round(initialScale * 10) / 10;
  const applyScale = value => {
    if (!Number.isFinite(value) || value < 0.7 || value > 1.5) {
      throw new Error("Interface scale is outside the supported range.");
    }
    scale = Math.round(value * 10) / 10;
    root.style.setProperty("--app-scale", String(scale));
    localStorage.setItem(key, String(scale));
  };
  applyScale(scale);

  window.addEventListener("keydown", event => {
    if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
    let nextScale = null;
    if (event.key === "+" || event.key === "=" || event.code === "Equal" || event.code === "NumpadAdd") {
      nextScale = scale + 0.1;
    } else if (event.key === "-" || event.code === "Minus" || event.code === "NumpadSubtract") {
      nextScale = scale - 0.1;
    } else if (event.key === "0" || event.code === "Numpad0") {
      nextScale = 1;
    }
    if (nextScale === null) return;
    event.preventDefault();
    applyScale(Math.max(0.7, Math.min(1.5, nextScale)));
  });
  if (!globalThis.__WAVELENGTH_NATIVE__) {
    document.addEventListener("contextmenu", event => event.preventDefault(), true);
  }
  root.dataset.appBehaviorsInstalled = "true";
  return true;
}
export function setFontScale(value) {
  const scale = Number(value);
  if (!Number.isFinite(scale) || scale < 0.8 || scale > 1.3) {
    throw new Error("Text size is outside the supported range.");
  }
  document.documentElement.style.setProperty("--font-size-scale", String(scale));
  return true;
}
export function effectiveTheme(theme) {
  return theme === "system"
    ? (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light")
    : theme;
}
function spotifyThumbnail(album) {
  const images = Array.isArray(album?.images) ? album.images : [];
  const candidates = images
    .map((image, index) => ({ image, index }))
    .filter(({ image }) => typeof image?.url === "string" && /^https?:\/\//i.test(image.url));
  if (!candidates.length) return null;
  return candidates.reduce((best, candidate) => {
    const area = (Number(candidate.image.width) || 0) * (Number(candidate.image.height) || 0);
    const bestArea = (Number(best.image.width) || 0) * (Number(best.image.height) || 0);
    return area > bestArea || (area === bestArea && candidate.index < best.index) ? candidate : best;
  }).image.url;
}
let railResizeCleanup = null;
export function startRailResize() {
  railResizeCleanup?.();
  const resize = event => {
    const width = Math.min(480, Math.max(300, window.innerWidth - event.clientX));
    document.documentElement.style.setProperty("--rail-width", `${width}px`);
  };
  const stop = () => {
    window.removeEventListener("pointermove", resize);
    window.removeEventListener("pointerup", stop);
    window.removeEventListener("pointercancel", stop);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    railResizeCleanup = null;
  };
  railResizeCleanup = stop;
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
  window.addEventListener("pointermove", resize);
  window.addEventListener("pointerup", stop);
  window.addEventListener("pointercancel", stop);
  return true;
}
export async function searchAudio(query, source) {
  if (source === "spotify" && localStorage.getItem("wavelength-spotify-refresh-token")) {
    const url = new URL("https://api.spotify.com/v1/search");
    url.search = new URLSearchParams({ q: query, type: "track", limit: "50" });
    const response = await fetch(url, {
      headers: { Authorization: `Bearer ${await getSpotifyAccessToken()}` }
    });
    const data = await spotifyResponse(response, "Spotify search failed.");
    return (data.tracks?.items || []).map(item => ({
      id: item.id,
      title: item.name,
      artist: item.artists?.map(artist => artist.name).join(", ") || "Unknown artist",
      album: item.album?.name || "",
      thumbnail: spotifyThumbnail(item.album),
      duration: item.duration_ms / 1000,
      source: "spotify"
    }));
  }
  const url = new URL(`${globalThis.__WAVELENGTH_AUDIO_SERVICE__ || "http://127.0.0.1:8787"}/search`);
  url.search = new URLSearchParams({ q: query, source, limit: "50", offset: "0" });
  const response = await fetch(url);
  const data = await response.json().catch(() => { throw new Error("Audio search returned invalid JSON."); });
  if (!response.ok) throw new Error(data.detail || data.error || "Audio search failed.");
  return data.results || [];
}
const spotifyClientId = "6e2ca58467e84ea5bf33b328d553c1b9";
let spotifyAccessToken = "";
let spotifyAccessTokenExpiresAt = 0;
let spotifyRefreshToken = "";
async function spotifyResponse(response, fallback) {
  if (response.status === 204) return {};
  let data;
  try {
    data = await response.json();
  } catch {
    throw new Error("Spotify returned invalid JSON.");
  }
  if (!response.ok) {
    throw new Error(data.error_description || data.error?.message || data.error || fallback);
  }
  return data;
}
async function spotifyProfile() {
  const response = await fetch("https://api.spotify.com/v1/me", {
    headers: { Authorization: `Bearer ${await getSpotifyAccessToken()}` }
  });
  return await spotifyResponse(response, "Could not load the Spotify profile.");
}
async function getSpotifyAccessToken() {
  if (spotifyAccessToken && spotifyAccessTokenExpiresAt > Date.now() + 60000) {
    return spotifyAccessToken;
  }
  const refreshToken = spotifyRefreshToken || localStorage.getItem("wavelength-spotify-refresh-token");
  if (!refreshToken) throw new Error("Connect Spotify to load your Spotify library.");
  const response = await fetch("https://accounts.spotify.com/api/token", {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({
      grant_type: "refresh_token",
      refresh_token: refreshToken,
      client_id: spotifyClientId
    })
  });
  const data = await spotifyResponse(response, "Spotify token refresh failed.");
  if (typeof data.access_token !== "string") {
    throw new Error("Spotify returned an invalid access token.");
  }
  spotifyAccessToken = data.access_token;
  spotifyAccessTokenExpiresAt = Date.now() + (Number(data.expires_in) || 3600) * 1000;
  if (typeof data.refresh_token === "string" && data.refresh_token) {
    spotifyRefreshToken = data.refresh_token;
    if (!globalThis.__WAVELENGTH_NATIVE__) {
      localStorage.setItem("wavelength-spotify-refresh-token", data.refresh_token);
    }
  }
  return spotifyAccessToken;
}
export async function restoreSpotify() {
  const params = new URLSearchParams(location.search);
  const code = params.get("code");
  const oauthError = params.get("error");
  let connectedNow = false;
  if (oauthError || code) {
    if (oauthError) throw new Error(`Spotify authorization failed: ${oauthError}.`);
    const state = sessionStorage.getItem("wavelength-spotify-oauth-state");
    const verifier = sessionStorage.getItem("wavelength-spotify-oauth-verifier");
    if (!state || state !== params.get("state") || !verifier) {
      throw new Error("Spotify authorization could not be verified. Please connect again.");
    }
    const redirectUri = `${location.origin}/settings`;
    const response = await fetch("https://accounts.spotify.com/api/token", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({
        grant_type: "authorization_code",
        code,
        redirect_uri: redirectUri,
        client_id: spotifyClientId,
        code_verifier: verifier
      })
    });
    const data = await spotifyResponse(response, "Spotify sign-in could not be completed.");
    if (typeof data.access_token !== "string" || typeof data.refresh_token !== "string") {
      throw new Error("Spotify returned incomplete authorization data.");
    }
    spotifyAccessToken = data.access_token;
    spotifyAccessTokenExpiresAt = Date.now() + (Number(data.expires_in) || 3600) * 1000;
    localStorage.setItem("wavelength-spotify-refresh-token", data.refresh_token);
    for (const key of [
      "wavelength-spotify-oauth-state",
      "wavelength-spotify-oauth-verifier"
    ]) sessionStorage.removeItem(key);
    history.replaceState(null, "", "/settings");
    connectedNow = true;
  } else if (!spotifyRefreshToken && !localStorage.getItem("wavelength-spotify-refresh-token")) {
    return null;
  }
  const profile = await spotifyProfile();
  return { profile, connected_now: connectedNow };
}
export async function spotifyLibrary() {
  const tracks = [];
  let url = "https://api.spotify.com/v1/me/tracks?limit=50";
  while (url) {
    const response = await fetch(url, {
      headers: { Authorization: `Bearer ${await getSpotifyAccessToken()}` }
    });
    const data = await spotifyResponse(response, "Could not load saved Spotify tracks.");
    for (const item of data.items || []) {
      const track = item.track;
      if (!track?.id) continue;
      tracks.push({
        id: track.id,
        title: track.name,
        artist: track.artists?.map(artist => artist.name).join(", ") || "Unknown artist",
        album: track.album?.name || "",
        thumbnail: spotifyThumbnail(track.album),
        duration: track.duration_ms / 1000,
        source: "spotify"
      });
    }
    url = data.next;
  }
  return tracks;
}
export async function spotifyPlaylists() {
  const playlists = [];
  let url = "https://api.spotify.com/v1/me/playlists?limit=50";
  while (url) {
    const response = await fetch(url, {
      headers: { Authorization: `Bearer ${await getSpotifyAccessToken()}` }
    });
    const data = await spotifyResponse(response, "Could not load Spotify playlists.");
    for (const item of data.items || []) {
      if (item?.id) playlists.push({
        id: item.id,
        name: item.name || "Untitled playlist",
        tracks_total: item.tracks?.total || 0
      });
    }
    url = data.next;
  }
  return playlists;
}
export async function spotifyPlaylistTracks(id) {
  const tracks = [];
  let url = `https://api.spotify.com/v1/playlists/${encodeURIComponent(id)}/tracks?limit=100`;
  while (url) {
    const response = await fetch(url, {
      headers: { Authorization: `Bearer ${await getSpotifyAccessToken()}` }
    });
    const data = await spotifyResponse(response, "Could not load tracks in this Spotify playlist.");
    for (const item of data.items || []) {
      const track = item.track;
      if (!track?.id) continue;
      tracks.push({
        id: track.id,
        title: track.name,
        artist: track.artists?.map(artist => artist.name).join(", ") || "Unknown artist",
        album: track.album?.name || "",
        thumbnail: spotifyThumbnail(track.album),
        duration: track.duration_ms / 1000,
        source: "spotify"
      });
    }
    url = data.next;
  }
  return tracks;
}
export async function saveSpotifyPlaylist(id, name, trackUris) {
  const token = await getSpotifyAccessToken();
  const headers = { Authorization: `Bearer ${token}`, "Content-Type": "application/json" };
  if (name) {
    const renamed = await fetch(`https://api.spotify.com/v1/playlists/${encodeURIComponent(id)}`, {
      method: "PUT", headers, body: JSON.stringify({ name })
    });
    await spotifyResponse(renamed, "Spotify playlist name could not be updated.");
  }
  const response = await fetch(`https://api.spotify.com/v1/playlists/${encodeURIComponent(id)}/items`, {
    method: "PUT", headers, body: JSON.stringify({ uris: trackUris })
  });
  await spotifyResponse(response, "Spotify playlist could not be updated.");
  return true;
}
export async function createSpotifyPlaylist(name) {
  const response = await fetch("https://api.spotify.com/v1/me/playlists", {
    method: "POST",
    headers: { Authorization: `Bearer ${await getSpotifyAccessToken()}`, "Content-Type": "application/json" },
    body: JSON.stringify({ name, public: false })
  });
  const data = await spotifyResponse(response, "Spotify playlist could not be created.");
  return { id: data.id, name: data.name || name, tracks_total: 0 };
}
export function connectSpotify() {
  const bytes = crypto.getRandomValues(new Uint8Array(64));
  const verifier = btoa(String.fromCharCode(...bytes))
    .replace(/=/g, "").replace(/\+/g, "-").replace(/\//g, "_");
  const stateBytes = crypto.getRandomValues(new Uint8Array(32));
  const state = btoa(String.fromCharCode(...stateBytes))
    .replace(/=/g, "").replace(/\+/g, "-").replace(/\//g, "_");
  return crypto.subtle.digest("SHA-256", new TextEncoder().encode(verifier)).then(digest => {
    const challenge = btoa(String.fromCharCode(...new Uint8Array(digest)))
      .replace(/=/g, "").replace(/\+/g, "-").replace(/\//g, "_");
    sessionStorage.setItem("wavelength-spotify-oauth-state", state);
    sessionStorage.setItem("wavelength-spotify-oauth-verifier", verifier);
    const authorize = new URL("https://accounts.spotify.com/authorize");
    authorize.search = new URLSearchParams({
      response_type: "code",
      client_id: spotifyClientId,
      redirect_uri: `${location.origin}/settings`,
      scope: "user-read-private user-read-email user-library-read user-library-modify playlist-read-private playlist-read-collaborative playlist-modify-public playlist-modify-private",
      code_challenge_method: "S256",
      code_challenge: challenge,
      state
    }).toString();
    location.assign(authorize.toString());
    return true;
  });
}
export function disconnectSpotify() {
  spotifyRefreshToken = "";
  localStorage.removeItem("wavelength-spotify-refresh-token");
  spotifyAccessToken = "";
  spotifyAccessTokenExpiresAt = 0;
  return true;
}
export function setSpotifyRefreshToken(token) {
  if (typeof token !== "string" || token.length > 4096) {
    throw new Error("The Spotify refresh token is invalid.");
  }
  spotifyRefreshToken = token;
  return true;
}
export function getSpotifyRefreshToken() {
  return spotifyRefreshToken;
}
export function playAudio() {
  const audio = document.getElementById("wavelength-audio");
  if (!audio) throw new Error("Audio player is not available.");
  audio.play().catch(error => { throw error; });
  return true;
}
export async function playAudioSource(source) {
  const audio = document.getElementById("wavelength-audio");
  if (!audio) throw new Error("Audio player is not available.");
  if (typeof source !== "string" || source.length === 0) {
    throw new Error("Audio source is not available.");
  }
  const expectedSource = new URL(source, document.baseURI).href;
  if (audio.src !== expectedSource) {
    await new Promise((resolve, reject) => {
      const observer = new MutationObserver(() => {
        if (audio.src === expectedSource) {
          observer.disconnect();
          clearTimeout(timeout);
          resolve();
        }
      });
      observer.observe(audio, { attributes: true, attributeFilter: ["src"] });
      const timeout = setTimeout(() => {
        observer.disconnect();
        reject(new Error("The audio source was not applied to the player."));
      }, 5000);
      if (audio.src === expectedSource) {
        observer.disconnect();
        clearTimeout(timeout);
        resolve();
      }
    });
  }
  try {
    await audio.play();
  } catch (error) {
    const mediaError = audio.error;
    const detail = mediaError
      ? `Media error ${mediaError.code}${mediaError.message ? `: ${mediaError.message}` : ""}`
      : error?.message ?? String(error);
    throw new Error(`Could not play audio: ${detail}`);
  }
  return true;
}
export function pauseAudio() {
  const audio = document.getElementById("wavelength-audio");
  if (!audio) throw new Error("Audio player is not available.");
  audio.pause();
  return true;
}
export function seekAudio(value) {
  const audio = document.getElementById("wavelength-audio");
  if (audio) audio.currentTime = value;
  return true;
}
export function readAudioTime() {
  const audio = document.getElementById("wavelength-audio");
  if (!audio) throw new Error("Audio player is not available.");
  return [audio.currentTime || 0, Number.isFinite(audio.duration) ? audio.duration : 0];
}
export function setAudioVolume(value) {
  const audio = document.getElementById("wavelength-audio");
  if (audio) audio.volume = Math.max(0, Math.min(1, value));
  return true;
}
export async function downloadAudio(id, filename) {
  const directory = await currentDownloadFolderHandle();
  if (directory) {
    const permission = await directory.requestPermission({ mode: "readwrite" });
    if (permission !== "granted") {
      throw new Error(`Write access to "${directory.name}" was not granted. Choose the folder again in Settings.`);
    }
  }
  const response = await fetch(`${globalThis.__WAVELENGTH_AUDIO_SERVICE__ || "http://127.0.0.1:8787"}/download/${encodeURIComponent(id)}?code=solo`);
  if (!response.ok) throw new Error(`Download failed (${response.status}).`);
  const blob = await response.blob();
  if (directory) {
    const safeFilename = filename
      .replace(/[<>:"/\\|?*\u0000-\u001f]/g, "_")
      .replace(/[. ]+$/g, "") || "download";
    const extensionIndex = safeFilename.lastIndexOf(".");
    const baseName = extensionIndex > 0 ? safeFilename.slice(0, extensionIndex) : safeFilename;
    const extension = extensionIndex > 0 ? safeFilename.slice(extensionIndex) : "";
    let availableFilename = null;
    for (let index = 0; index < 1000; index += 1) {
      const candidate = index === 0 ? safeFilename : `${baseName} (${index})${extension}`;
      try {
        await directory.getFileHandle(candidate);
      } catch (error) {
        if (error?.name !== "NotFoundError") throw error;
        availableFilename = candidate;
        break;
      }
    }
    if (!availableFilename) throw new Error("Could not find an unused filename for this download.");
    const file = await directory.getFileHandle(availableFilename, { create: true });
    const writable = await file.createWritable();
    await writable.write(blob);
    await writable.close();
    return availableFilename;
  }
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = filename;
  link.click();
  URL.revokeObjectURL(link.href);
  return filename;
}
const files = new Map(), urls = new Map();
function supported(name) { return /\.(mp3|flac|m4a|ogg|wav|aac)$/i.test(name); }
let downloadFolderHandle = null;
let downloadFolderLoaded = false;
let downloadFolderLoadPromise = null;
function openFolderDatabase() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open("wavelength-library", 2);
    request.onupgradeneeded = () => {
      const database = request.result;
      if (!database.objectStoreNames.contains("folders")) {
        database.createObjectStore("folders", { keyPath: "name" });
      }
      if (!database.objectStoreNames.contains("download-settings")) {
        database.createObjectStore("download-settings", { keyPath: "key" });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error || new Error("Could not open saved music folders."));
  });
}
async function loadDownloadFolderHandle() {
  const database = await openFolderDatabase();
  return await new Promise((resolve, reject) => {
    const request = database
      .transaction("download-settings", "readonly")
      .objectStore("download-settings")
      .get("download-folder");
    request.onsuccess = () => resolve(request.result?.handle || null);
    request.onerror = () => reject(request.error || new Error("Could not read the download folder."));
  });
}
async function currentDownloadFolderHandle() {
  if (downloadFolderLoaded) return downloadFolderHandle;
  if (!downloadFolderLoadPromise) downloadFolderLoadPromise = loadDownloadFolderHandle();
  try {
    downloadFolderHandle = await downloadFolderLoadPromise;
    downloadFolderLoaded = true;
    return downloadFolderHandle;
  } finally {
    downloadFolderLoadPromise = null;
  }
}
export async function savedDownloadFolderName() {
  const handle = await currentDownloadFolderHandle();
  return handle?.name || null;
}
export async function chooseDownloadFolder() {
  if (!window.showDirectoryPicker) {
    throw new Error("Custom download folders are not supported in this browser.");
  }
  let handle;
  try {
    handle = await window.showDirectoryPicker({ mode: "readwrite" });
  } catch (error) {
    if (error?.name === "AbortError") return null;
    throw error;
  }
  const permission = await handle.requestPermission({ mode: "readwrite" });
  if (permission !== "granted") {
    throw new Error(`Write access to "${handle.name}" was not granted.`);
  }
  const database = await openFolderDatabase();
  await new Promise((resolve, reject) => {
    const request = database
      .transaction("download-settings", "readwrite")
      .objectStore("download-settings")
      .put({ key: "download-folder", handle });
    request.onsuccess = () => resolve();
    request.onerror = () => reject(request.error || new Error("Could not save the download folder."));
  });
  downloadFolderHandle = handle;
  downloadFolderLoaded = true;
  return handle.name;
}
export async function clearDownloadFolder() {
  const database = await openFolderDatabase();
  await new Promise((resolve, reject) => {
    const request = database
      .transaction("download-settings", "readwrite")
      .objectStore("download-settings")
      .delete("download-folder");
    request.onsuccess = () => resolve();
    request.onerror = () => reject(request.error || new Error("Could not reset the download folder."));
  });
  downloadFolderHandle = null;
  downloadFolderLoaded = true;
}
async function savedFolders() {
  const database = await openFolderDatabase();
  return await new Promise((resolve, reject) => {
    const request = database.transaction("folders", "readonly").objectStore("folders").getAll();
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error || new Error("Could not read saved music folders."));
  });
}
export async function savedLibraryFolders() {
  return (await savedFolders()).map(folder => folder.name).sort((left, right) => left.localeCompare(right));
}
export async function removeSavedLibraryFolder(name) {
  const database = await openFolderDatabase();
  await new Promise((resolve, reject) => {
    const request = database.transaction("folders", "readwrite").objectStore("folders").delete(name);
    request.onsuccess = () => resolve();
    request.onerror = () => reject(request.error || new Error("Could not remove the saved music folder."));
  });
  for (const id of files.keys()) {
    if (id.startsWith(`local-${name}/`) || id.startsWith(`local-${name}-`)) {
      files.delete(id);
      if (urls.has(id)) {
        URL.revokeObjectURL(urls.get(id));
        urls.delete(id);
      }
    }
  }
}
async function rememberFolder(handle) {
  const database = await openFolderDatabase();
  await new Promise((resolve, reject) => {
    const request = database.transaction("folders", "readwrite").objectStore("folders").put({ name: handle.name, kind: "handle", handle });
    request.onsuccess = () => resolve();
    request.onerror = () => reject(request.error || new Error("Could not save this music folder."));
  });
}
async function rememberFileFolder(name, files) {
  const database = await openFolderDatabase();
  await new Promise((resolve, reject) => {
    const request = database.transaction("folders", "readwrite").objectStore("folders").put({ name, kind: "file-list", files });
    request.onsuccess = () => resolve();
    request.onerror = () => {
      reject(request.error?.name === "QuotaExceededError"
        ? new Error("This folder is too large for the browser's available storage.")
        : request.error || new Error("Could not save this music folder."));
    };
  });
}
function track(file, folder) {
  const id = `local-${folder}-${file.name}`;
  files.set(id, file);
  if (urls.has(id)) URL.revokeObjectURL(urls.get(id));
  urls.set(id, URL.createObjectURL(file));
  return { id, title: file.name.replace(/\.[^.]+$/, ""), artist: "Local file", album: folder, folderPath: folder, source: "local" };
}
async function scanFolder(handle, tracks, folder) {
  for await (const entry of handle.values()) {
    if (entry.kind === "directory") await scanFolder(entry, tracks, `${folder}/${entry.name}`);
    else if (entry.kind === "file" && supported(entry.name)) tracks.push(track(await entry.getFile(), folder));
  }
}
export async function scanLibrary() {
  const tracks = [];
  for (const folder of await savedFolders()) {
    if (folder.kind === "file-list") {
      for (const entry of folder.files || []) {
        const path = entry.relativePath || `${folder.name}/${entry.file.name}`;
        const parent = path.split("/").slice(0, -1).join("/") || folder.name;
        tracks.push(track(entry.file, parent));
      }
      continue;
    }
    const handle = folder.handle || folder;
    const permission = await handle.queryPermission({ mode: "read" });
    if (permission !== "granted") {
      throw new Error(`Access to "${folder.name}" was revoked. Add the folder again to restore access.`);
    }
    await scanFolder(handle, tracks, folder.name);
  }
  return tracks;
}
export async function chooseFolder() {
  if (window.showDirectoryPicker) {
    const handle = await window.showDirectoryPicker();
    const tracks = [];
    await scanFolder(handle, tracks, handle.name);
    await rememberFolder(handle);
    return tracks;
  }
  return await new Promise((resolve, reject) => {
    const input = document.createElement("input");
    input.type = "file"; input.multiple = true; input.setAttribute("webkitdirectory", "");
    input.onchange = async () => {
      try {
        const files = Array.from(input.files || []).filter(file => supported(file.name));
        if (!files.length) {
          resolve([]);
          return;
        }
        const firstPath = files[0].webkitRelativePath || files[0].name || "Music";
        const folderName = firstPath.split("/")[0] || "Music";
        const entries = files.map(file => ({
          file,
          relativePath: file.webkitRelativePath || `${folderName}/${file.name}`
        }));
        await rememberFileFolder(folderName, entries);
        resolve(entries.map(entry => {
          const parent = entry.relativePath.split("/").slice(0, -1).join("/") || folderName;
          return track(entry.file, parent);
        }));
      } catch (error) {
        reject(error);
      }
    };
    input.onerror = () => reject(new Error("Could not read the selected folder."));
    input.click();
  });
}
export function localUrl(id) {
  const url = urls.get(id);
  if (!url) throw new Error("Re-import this local folder to play its tracks.");
  return url;
}
export async function roomRequest(url, path, method, body, access, host) {
  const headers = { "Content-Type": "application/json" };
  if (access) headers["x-room-access-token"] = access;
  if (host) headers.Authorization = `Bearer ${host}`;
  let response;
  try {
    const requestMethod = method.toUpperCase();
    const request = { method: requestMethod, headers, credentials: "omit" };
    if (requestMethod !== "GET" && requestMethod !== "HEAD") request.body = body;
    response = await fetch(`${url.replace(/\/+$/, "")}/api${path}`, request);
  } catch (error) {
    if (error instanceof TypeError) {
      throw new Error("Could not reach the room server. Check its URL and port, confirm it is running, and make sure its CORS settings allow this app.");
    }
    throw error;
  }
  let data;
  try {
    data = await response.json();
  } catch {
    throw new Error("Room server returned invalid JSON.");
  }
  if (!response.ok) throw new Error(data.error || "Room request failed.");
  return data;
}
export async function requestJson(url) {
  const response = await fetch(url);
  let data;
  try {
    data = await response.json();
  } catch {
    throw new Error("The audio service returned invalid JSON.");
  }
  if (!response.ok) throw new Error(data.detail || "Request failed.");
  return data;
}
export async function requestJsonMethod(url, method, body) {
  const response = await fetch(url, {
    method,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body)
  });
  let data;
  try {
    data = await response.json();
  } catch {
    throw new Error("The audio service returned invalid JSON.");
  }
  if (!response.ok) throw new Error(data.detail || data.error || "Request failed.");
  return data;
}
export async function fetchLyrics(title, artist) {
  const query = new URLSearchParams({ title, artist });
  const response = await fetch(`${globalThis.__WAVELENGTH_AUDIO_SERVICE__ || "http://127.0.0.1:8787"}/lyrics?${query}`);
  if (response.status === 404) return null;
  let data;
  try {
    data = await response.json();
  } catch {
    throw new Error("The audio service returned invalid lyrics data.");
  }
  if (!response.ok) throw new Error(data.detail || "Lyrics are temporarily unavailable.");
  return data;
}
let roomSocket = null;
let roomSocketConfig = null;
let roomSocketReady = false;
let roomSocketEvents = [];
let roomSocketRetry = null;
export function connectRoomSocket(url, code, username, accessToken, role, permissions) {
  disconnectRoomSocket();
  roomSocketConfig = { url, code, username, accessToken, role, permissions };
  roomSocketEvents = [];
  openRoomSocket();
  return true;
}
function openRoomSocket() {
  if (!roomSocketConfig) return;
  const config = roomSocketConfig;
  const endpoint = new URL("/socket.io/", config.url);
  endpoint.protocol = endpoint.protocol === "https:" ? "wss:" : "ws:";
  endpoint.search = new URLSearchParams({ EIO: "4", transport: "websocket" }).toString();
  const socket = new WebSocket(endpoint);
  roomSocket = socket;
  roomSocketReady = false;
  socket.onmessage = event => {
    const packet = String(event.data);
    if (packet.startsWith("0")) {
      socket.send(`40${JSON.stringify({
        code: config.code,
        username: config.username,
        accessToken: config.accessToken
      })}`);
      return;
    }
    if (packet === "2") {
      socket.send("3");
      return;
    }
    if (packet.startsWith("40")) {
      roomSocketReady = true;
      roomSocketEvents.push({ event: "connect", data: null });
      return;
    }
    if (!packet.startsWith("42")) return;
    try {
      const [name, data] = JSON.parse(packet.slice(2));
      if (name === "room-state" && data?.permissions) {
        config.permissions = data.permissions;
      }
      roomSocketEvents.push({ event: name, data });
    } catch (error) {
      console.error("Could not decode a room event.", error);
    }
  };
  socket.onerror = () => roomSocketEvents.push({
    event: "error",
    data: "Room socket connection failed. Check that the server or proxy allows WebSocket upgrades on /socket.io/."
  });
  socket.onclose = () => {
    if (roomSocket !== socket) return;
    roomSocketReady = false;
    roomSocketEvents.push({ event: "disconnect", data: null });
    roomSocket = null;
    if (roomSocketConfig === config) {
      roomSocketRetry = window.setTimeout(openRoomSocket, 2000);
    }
  };
}
export function disconnectRoomSocket() {
  roomSocketConfig = null;
  roomSocketReady = false;
  if (roomSocketRetry) window.clearTimeout(roomSocketRetry);
  roomSocketRetry = null;
  if (roomSocket) {
    const socket = roomSocket;
    roomSocket = null;
    socket.close();
  }
}
export function emitRoomCommand(action, details) {
  if (!roomSocketConfig || roomSocketConfig.role !== "guest") return "not-guest";
  if (!roomSocket || roomSocket.readyState !== WebSocket.OPEN || !roomSocketReady) return "unavailable";
  const permission = action === "add-track"
    ? "canAdd"
    : ["remove-track", "clear-queue", "reorder-queue"].includes(action)
      ? "canManageQueue"
      : "canControlPlayback";
  if (!roomSocketConfig.permissions?.[permission]) return "permission-denied";
  roomSocket.send(`42${JSON.stringify(["room-command", { action, ...details }])}`);
  return "sent";
}
export function emitRoomPlaybackControl(control) {
  if (!roomSocketConfig || roomSocketConfig.role !== "host") return "not-host";
  if (!roomSocket || roomSocket.readyState !== WebSocket.OPEN || !roomSocketReady) return "unavailable";
  roomSocket.send(`42${JSON.stringify(["playback-control", control])}`);
  return "sent";
}
export function emitRoomPlaybackProgress(progress) {
  if (!roomSocketConfig || roomSocketConfig.role !== "host") return "not-host";
  if (!roomSocket || roomSocket.readyState !== WebSocket.OPEN || !roomSocketReady) return "unavailable";
  roomSocket.send(`42${JSON.stringify(["playback-progress", progress])}`);
  return "sent";
}
export function drainRoomEvents() {
  return roomSocketEvents.splice(0, roomSocketEvents.length);
}
export async function uploadRoomTrack(url, code, trackId, hostToken) {
  const file = files.get(trackId);
  if (!file) throw new Error("The local audio file is no longer available in this browser.");
  const extension = file.name.split(".").pop()?.toLowerCase();
  if (!["mp3", "flac", "m4a", "ogg", "wav", "aac", "opus"].includes(extension)) {
    throw new Error("This local audio format cannot be shared with the room.");
  }
  const response = await fetch(`${url.replace(/\/+$/, "")}/api/parties/${encodeURIComponent(code)}/audio`, {
    method: "POST",
    headers: { Authorization: `Bearer ${hostToken}`, "x-audio-extension": extension },
    body: file
  });
  let data;
  try {
    data = await response.json();
  } catch {
    throw new Error("Room server returned invalid audio upload data.");
  }
  if (!response.ok) throw new Error(data.error || "Local audio could not be shared.");
  if (typeof data.audioId !== "string" || !data.audioId) {
    throw new Error("Room server did not return a shared audio ID.");
  }
  return data.audioId;
}
export function hasLocalAudioFile(trackId) {
  return files.has(trackId);
}
export async function roomAudioUrl(audioId) {
  if (!roomSocketConfig) throw new Error("This track is not attached to an active room.");
  const { url, code, accessToken } = roomSocketConfig;
  const path = `/api/parties/${encodeURIComponent(code)}/audio/${encodeURIComponent(audioId)}/access`;
  const response = await fetch(`${url.replace(/\/+$/, "")}${path}`, {
    headers: { "x-room-access-token": accessToken },
    credentials: "omit"
  });
  let data;
  try {
    data = await response.json();
  } catch {
    throw new Error("Room server returned invalid audio access data.");
  }
  if (!response.ok) throw new Error(data.error || "Shared audio access could not be created.");
  if (typeof data.url !== "string" || !data.url) {
    throw new Error("Room server did not return a shared audio URL.");
  }
  return new URL(data.url, url).toString();
}
export function openExternalTrack(source, id) {
  const url = source === "spotify"
    ? `https://open.spotify.com/track/${encodeURIComponent(id)}`
    : source === "youtube"
      ? `https://www.youtube.com/watch?v=${encodeURIComponent(id)}`
      : null;
  if (!url) throw new Error("This track has no supported external page.");
  window.open(url, "_blank", "noopener,noreferrer");
}
export function setEqualizer(settings) {
  const audio = document.getElementById("wavelength-audio");
  const Context = window.AudioContext || window.webkitAudioContext;
  if (!audio || !Context) throw new Error("Equalizer is unavailable in this browser.");
  if (!settings || !["gains", "frequencies", "qs", "types"].every(key => Array.isArray(settings[key]) && settings[key].length === 10)) {
    throw new Error("Equalizer settings are invalid.");
  }
  if (!window.__wavelengthEqualizer) {
    audio.crossOrigin = "anonymous";
    const context = new Context();
    const source = context.createMediaElementSource(audio);
    let previous = source;
    const filters = settings.frequencies.map((frequency, index) => {
      const filter = context.createBiquadFilter();
      filter.frequency.value = frequency;
      filter.Q.value = settings.qs[index];
      filter.type = settings.types[index];
      previous.connect(filter);
      previous = filter;
      filter.gain.value = settings.gains[index];
      return filter;
    });
    const analyser = context.createAnalyser();
    analyser.fftSize = 2048;
    analyser.smoothingTimeConstant = 0.82;
    previous.connect(analyser);
    analyser.connect(context.destination);
    window.__wavelengthEqualizer = {
      audio,
      context,
      filters,
      analyser,
      canvasId: null,
      selectedBand: 0,
      preview: null,
      frame: null,
      spectrum: new Uint8Array(analyser.frequencyBinCount)
    };
  } else {
    const state = window.__wavelengthEqualizer;
    const { context, filters } = state;
    state.audio = audio;
    if (!state.analyser) {
      state.analyser = context.createAnalyser();
      state.analyser.fftSize = 2048;
      state.analyser.smoothingTimeConstant = 0.82;
      filters[filters.length - 1].disconnect(context.destination);
      filters[filters.length - 1].connect(state.analyser);
      state.analyser.connect(context.destination);
      state.spectrum = new Uint8Array(state.analyser.frequencyBinCount);
    }
    filters.forEach((filter, index) => {
      filter.type = settings.types[index];
      [
        [filter.frequency, settings.frequencies[index]],
        [filter.Q, settings.qs[index]],
        [filter.gain, settings.gains[index]]
      ].forEach(([parameter, value]) => parameter.setTargetAtTime(value, context.currentTime, 0.015));
    });
  }
  void window.__wavelengthEqualizer.context.resume();
  return true;
}
export function startEqualizerVisualization(canvasId, selectedBand) {
  const state = window.__wavelengthEqualizer;
  if (!state) throw new Error("Equalizer audio is not initialized.");
  state.canvasId = canvasId;
  state.selectedBand = Math.max(0, Math.min(9, Number(selectedBand) || 0));
  state.preview = null;
  if (state.frame) cancelAnimationFrame(state.frame);

  const draw = () => {
    const canvas = document.getElementById(state.canvasId);
    if (!canvas || !canvas.isConnected) {
      state.frame = null;
      return;
    }
    const bounds = canvas.getBoundingClientRect();
    const ratio = Math.max(1, window.devicePixelRatio || 1);
    const pixelWidth = Math.max(1, Math.round(bounds.width * ratio));
    const pixelHeight = Math.max(1, Math.round(bounds.height * ratio));
    if (canvas.width !== pixelWidth || canvas.height !== pixelHeight) {
      canvas.width = pixelWidth;
      canvas.height = pixelHeight;
    }
    const context = canvas.getContext("2d");
    if (!context) {
      state.frame = null;
      return;
    }
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    context.clearRect(0, 0, bounds.width, bounds.height);
    const style = getComputedStyle(document.documentElement);
    const accent = style.getPropertyValue("--lime").trim() || "#d7e98b";
    const muted = style.getPropertyValue("--muted").trim() || "#9da196";
    const line = style.getPropertyValue("--line").trim() || "rgba(237, 240, 232, .11)";
    const plot = {
      left: 44,
      right: Math.max(45, bounds.width - 14),
      top: 16,
      bottom: Math.max(47, bounds.height - 30)
    };
    const plotWidth = Math.max(1, plot.right - plot.left);
    const plotHeight = Math.max(1, plot.bottom - plot.top);
    const xForFrequency = frequency =>
      plot.left + Math.log(Math.max(20, frequency) / 20) / Math.log(1000) * plotWidth;
    const yForGain = gain =>
      plot.top + (18 - Math.max(-18, Math.min(18, gain))) / 36 * plotHeight;

    context.font = bounds.width < 360
      ? "9px 'IBM Plex Mono', monospace"
      : "11px 'IBM Plex Mono', monospace";
    context.textBaseline = "middle";
    for (const gain of [-18, -12, -6, 0, 6, 12, 18]) {
      const y = yForGain(gain);
      context.beginPath();
      context.moveTo(plot.left, y);
      context.lineTo(plot.right, y);
      context.strokeStyle = gain === 0 ? "rgba(237, 240, 232, .24)" : line;
      context.lineWidth = 1;
      context.stroke();
      context.fillStyle = muted;
      context.textAlign = "right";
      context.fillText(`${gain > 0 ? "+" : ""}${gain}`, plot.left - 8, y);
    }
    const frequencyTicks = [
      [20, "20"],
      [100, "100"],
      [1000, "1k"],
      [10000, "10k"],
      [20000, "20k"]
    ];
    context.textBaseline = "top";
    for (const [frequency, label] of frequencyTicks) {
      const x = xForFrequency(frequency);
      context.beginPath();
      context.moveTo(x, plot.top);
      context.lineTo(x, plot.bottom);
      context.strokeStyle = line;
      context.lineWidth = 1;
      context.stroke();
      context.fillStyle = muted;
      context.textAlign = frequency === 20 ? "left" : frequency === 20000 ? "right" : "center";
      context.fillText(label, x, plot.bottom + 8);
    }

    context.save();
    context.beginPath();
    context.rect(plot.left, plot.top, plotWidth, plotHeight);
    context.clip();
    if (!state.audio.paused && state.analyser) {
      state.analyser.getByteFrequencyData(state.spectrum);
      context.beginPath();
      for (let x = 0; x < plotWidth; x += 3) {
        const frequency = 20 * Math.pow(1000, x / plotWidth);
        const bin = Math.min(
          state.spectrum.length - 1,
          Math.max(0, Math.round(frequency / state.context.sampleRate * state.analyser.fftSize))
        );
        const y = plot.bottom - state.spectrum[bin] / 255 * plotHeight * 0.28;
        context.moveTo(plot.left + x, plot.bottom);
        context.lineTo(plot.left + x, y);
      }
      context.strokeStyle = "rgba(157, 161, 150, .32)";
      context.lineWidth = 2;
      context.stroke();
    }

    const samples = 320;
    const sampleRateLimit = Math.min(20000, state.context.sampleRate / 2);
    const frequencies = new Float32Array(samples);
    const magnitudes = new Float32Array(samples);
    const phases = new Float32Array(samples);
    const response = new Float32Array(samples);
    for (let index = 0; index < samples; index++) {
      frequencies[index] = 20 * Math.pow(sampleRateLimit / 20, index / (samples - 1));
    }
    for (const filter of state.filters) {
      filter.getFrequencyResponse(frequencies, magnitudes, phases);
      for (let index = 0; index < samples; index++) {
        response[index] += 20 * Math.log10(Math.max(magnitudes[index], 0.000001));
      }
    }
    context.beginPath();
    for (let index = 0; index < samples; index++) {
      const x = plot.left + index / (samples - 1) * plotWidth;
      const y = yForGain(response[index]);
      if (index === 0) context.moveTo(x, y);
      else context.lineTo(x, y);
    }
    context.strokeStyle = accent;
    context.lineWidth = 2;
    context.lineJoin = "round";
    context.stroke();

    if (state.preview) {
      const x = Math.max(plot.left, Math.min(plot.right, state.preview.x));
      const y = Math.max(plot.top, Math.min(plot.bottom, state.preview.y));
      context.save();
      context.globalAlpha = 0.45;
      context.strokeStyle = accent;
      context.lineWidth = 1;
      context.setLineDash([4, 4]);
      context.beginPath();
      context.moveTo(x, plot.top);
      context.lineTo(x, plot.bottom);
      context.moveTo(plot.left, y);
      context.lineTo(plot.right, y);
      context.stroke();
      context.setLineDash([]);
      context.beginPath();
      context.arc(x, y, 7, 0, Math.PI * 2);
      context.fillStyle = style.getPropertyValue("--panel").trim() || "#151714";
      context.fill();
      context.stroke();
      context.restore();
    }

    state.filters.forEach((filter, index) => {
      const x = xForFrequency(filter.frequency.value);
      const y = yForGain(filter.gain.value);
      const selected = index === state.selectedBand;
      context.beginPath();
      context.arc(x, y, selected ? 6 : 4, 0, Math.PI * 2);
      context.fillStyle = selected ? accent : style.getPropertyValue("--panel").trim() || "#151714";
      context.fill();
      context.strokeStyle = accent;
      context.lineWidth = selected ? 2 : 1.5;
      context.stroke();
    });
    context.restore();
    state.frame = requestAnimationFrame(draw);
  };

  state.frame = requestAnimationFrame(draw);
  return true;
}
export function setEqualizerSelectedBand(index) {
  const state = window.__wavelengthEqualizer;
  if (!state) return false;
  state.selectedBand = Math.max(0, Math.min(9, Number(index) || 0));
  return true;
}
export function setEqualizerPreview(index, x, y) {
  const state = window.__wavelengthEqualizer;
  if (!state) throw new Error("Equalizer audio is not initialized.");
  if (!Number.isInteger(index) || index < 0 || index > 9 || !Number.isFinite(x) || !Number.isFinite(y)) {
    throw new Error("Equalizer placement coordinates are invalid.");
  }
  state.selectedBand = index;
  state.preview = { x, y };
  return true;
}
export function clearEqualizerPreview() {
  const state = window.__wavelengthEqualizer;
  if (state) state.preview = null;
  return true;
}
export function equalizerCanvasSize() {
  const canvas = document.getElementById("equalizer-response-canvas");
  if (!canvas) throw new Error("Equalizer graph is not available.");
  const bounds = canvas.getBoundingClientRect();
  return [bounds.width, bounds.height];
}


globalThis.__WAVELENGTH_BROWSER__ = { getLocal, setLocal, removeLocal, installHoverMarquee, watchNarrowWindow, chooseAvatar, getRoomInviteCode, copyRoomCode, copyRoomInvite, setAppearance, isDesktopWindow, checkForUpdates, installUpdate, windowAction, installAppBehaviors, setFontScale, effectiveTheme, startRailResize, searchAudio, restoreSpotify, spotifyLibrary, spotifyPlaylists, spotifyPlaylistTracks, saveSpotifyPlaylist, createSpotifyPlaylist, connectSpotify, disconnectSpotify, setSpotifyRefreshToken, getSpotifyRefreshToken, playAudio, playAudioSource, pauseAudio, seekAudio, readAudioTime, setAudioVolume, downloadAudio, savedDownloadFolderName, chooseDownloadFolder, clearDownloadFolder, savedLibraryFolders, removeSavedLibraryFolder, scanLibrary, chooseFolder, localUrl, roomRequest, requestJson, requestJsonMethod, fetchLyrics, connectRoomSocket, disconnectRoomSocket, emitRoomCommand, emitRoomPlaybackControl, emitRoomPlaybackProgress, drainRoomEvents, uploadRoomTrack, hasLocalAudioFile, roomAudioUrl, openExternalTrack, setEqualizer, startEqualizerVisualization, setEqualizerSelectedBand, setEqualizerPreview, clearEqualizerPreview, equalizerCanvasSize };