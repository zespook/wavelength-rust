import assert from "node:assert/strict";
import test from "node:test";
import {
  connectRoomSocket,
  disconnectRoomSocket,
  drainRoomEvents,
  emitRoomHostState,
  emitRoomCommand,
  fetchLyrics,
  preloadAudio,
  scrollActiveLyricIntoView,
  setSpotifyRefreshToken,
  searchAudio,
  renameSpotifyPlaylist,
  saveSpotifyPlaylist,
  spotifyAlbumTracks,
  spotifyAlbums,
  spotifyPlaylistTracks,
  spotifyPlaylists,
  setEqualizer,
  roomRequest
} from "../src/browser.js";

test("setEqualizer bypasses its filters unless explicitly enabled", () => {
  const originalWindow = globalThis.window;
  const originalDocument = globalThis.document;
  const originalState = globalThis.window?.__wavelengthEqualizer;
  const audio = {};

  const parameter = () => ({
    value: 0,
    setTargetAtTime(value) {
      this.value = value;
    }
  });
  class AudioNode {
    connections = [];
    connect(node) {
      this.connections.push(node);
    }
  }
  class AudioContext {
    currentTime = 0;
    destination = new AudioNode();
    createMediaElementSource() {
      return new AudioNode();
    }
    createGain() {
      return Object.assign(new AudioNode(), { gain: parameter() });
    }
    createBiquadFilter() {
      return Object.assign(new AudioNode(), {
        frequency: parameter(),
        Q: parameter(),
        gain: parameter()
      });
    }
    createAnalyser() {
      return Object.assign(new AudioNode(), { frequencyBinCount: 1 });
    }
    resume() {
      return Promise.resolve();
    }
  }

  globalThis.window = { AudioContext };
  globalThis.document = { getElementById: () => audio };

  try {
    const settings = {
      gains: Array(10).fill(0),
      frequencies: Array(10).fill(1000),
      qs: Array(10).fill(1),
      types: Array(10).fill("peaking"),
      enabled: false
    };

    assert.equal(setEqualizer(settings), true);
    const state = globalThis.window.__wavelengthEqualizer;
    assert.equal(state.bypassGain.gain.value, 1);
    assert.equal(state.equalizerGain.gain.value, 0);

    assert.equal(setEqualizer({ ...settings, enabled: true }), true);
    assert.equal(state.bypassGain.gain.value, 0);
    assert.equal(state.equalizerGain.gain.value, 1);
  } finally {
    if (originalWindow === undefined) delete globalThis.window;
    else {
      globalThis.window = originalWindow;
      if (originalState === undefined) delete globalThis.window.__wavelengthEqualizer;
      else globalThis.window.__wavelengthEqualizer = originalState;
    }
    if (originalDocument === undefined) delete globalThis.document;
    else globalThis.document = originalDocument;
  }
});

test("preloadAudio schedules deduplicated IDs with the requested cache scope", async () => {
  const originalFetch = globalThis.fetch;
  const originalService = globalThis.__WAVELENGTH_AUDIO_SERVICE__;
  const requests = [];
  globalThis.__WAVELENGTH_AUDIO_SERVICE__ = "https://audio.example";
  globalThis.fetch = async (url, options) => {
    requests.push({ url: new URL(url), options });
    return new Response('{"scheduled":true}', {
      headers: { "Content-Type": "application/json" }
    });
  };

  try {
    assert.equal(globalThis.__WAVELENGTH_BROWSER__.preloadAudio, preloadAudio);
    await preloadAudio(["yt-one", "spotify-two", "yt-one"], "room");

    assert.equal(requests.length, 1);
    assert.equal(requests[0].url.pathname, "/preload");
    assert.equal(requests[0].url.searchParams.get("video_ids"), "yt-one,spotify-two");
    assert.equal(requests[0].url.searchParams.get("scope"), "room");
    assert.equal(requests[0].options.method, "POST");
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.__WAVELENGTH_AUDIO_SERVICE__ = originalService;
  }
});

test("roomRequest sends serialized JSON once and omits bodies on GET", async () => {
  const originalFetch = globalThis.fetch;
  const requests = [];
  globalThis.fetch = async (url, options) => {
    requests.push({ url, options });
    return new Response('{"ok":true}', {
      headers: { "Content-Type": "application/json" }
    });
  };

  try {
    await roomRequest(
      "https://rooms.example/",
      "/session",
      "POST",
      '{"passcode":"test"}',
      "",
      ""
    );
    await roomRequest("https://rooms.example", "/rooms/ABCD", "GET", "{}", "", "");

    assert.equal(requests[0].url, "https://rooms.example/api/v2/session");
    assert.equal(requests[1].url, "https://rooms.example/api/v2/rooms/ABCD");
    assert.equal(requests[0].options.body, '{"passcode":"test"}');
    assert.equal("body" in requests[1].options, false);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("roomRequest retries GET once after Retry-After", async () => {
  const originalFetch = globalThis.fetch;
  const originalSetTimeout = globalThis.setTimeout;
  const requests = [];
  const delays = [];
  const responses = [
    new Response('{"error":"Too many attempts"}', {
      status: 429,
      headers: { "Content-Type": "application/json", "Retry-After": "3" }
    }),
    new Response('{"code":"ABCD"}', {
      headers: { "Content-Type": "application/json" }
    })
  ];
  globalThis.fetch = async (url, options) => {
    requests.push({ url, options });
    return responses.shift();
  };
  globalThis.setTimeout = (callback, delay) => {
    delays.push(delay);
    callback();
    return 1;
  };

  try {
    const result = await roomRequest("https://rooms.example", "/rooms/ABCD", "GET", "{}", "access", "");
    assert.deepEqual(result, { code: "ABCD" });
    assert.equal(requests.length, 2);
    assert.deepEqual(delays, [3000]);
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.setTimeout = originalSetTimeout;
  }
});

test("roomRequest does not retry writes after a rate limit", async () => {
  const originalFetch = globalThis.fetch;
  const originalSetTimeout = globalThis.setTimeout;
  const requests = [];
  globalThis.fetch = async (url, options) => {
    requests.push({ url, options });
    return new Response('{"error":"Too many attempts"}', {
      status: 429,
      headers: { "Content-Type": "application/json", "Retry-After": "3" }
    });
  };
  globalThis.setTimeout = () => {
    throw new Error("Write requests must not be retried.");
  };

  try {
    await assert.rejects(
      roomRequest("https://rooms.example", "/rooms", "POST", '{"host":"Host"}', "access", ""),
      /Too many attempts/
    );
    assert.equal(requests.length, 1);
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.setTimeout = originalSetTimeout;
  }
});

test("room sockets use a one-time ticket and direct JSON events", async () => {
  const OriginalWebSocket = globalThis.WebSocket;
  const originalWindow = globalThis.window;
  const originalFetch = globalThis.fetch;
  const requests = [];
  const sockets = [];
  class MockWebSocket {
    static OPEN = 1;
    readyState = 0;
    sent = [];
    constructor(url) {
      this.url = String(url);
      sockets.push(this);
    }
    send(message) {
      this.sent.push(message);
    }
    close() {
      this.readyState = 3;
    }
  }
  globalThis.WebSocket = MockWebSocket;
  globalThis.window = {
    setTimeout: () => 1,
    clearTimeout: () => {}
  };
  globalThis.fetch = async (url, options) => {
    requests.push({ url, options });
    return new Response('{"ticket":"one-time-ticket"}', {
      headers: { "Content-Type": "application/json" }
    });
  };

  try {
    connectRoomSocket("https://rooms.example", "AB12", "Guest", "access", "", "guest", {
      canManageQueue: true
    });
    connectRoomSocket("https://rooms.example", "AB12", "Guest", "access", "", "guest", {
      canManageQueue: true
    });
    await new Promise(resolve => setImmediate(resolve));
    assert.equal(requests.length, 1);
    const socket = sockets[0];
    const endpoint = new URL(socket.url);
    assert.equal(endpoint.pathname, "/api/v2/rooms/AB12/socket");
    assert.equal(endpoint.protocol, "wss:");
    assert.equal(endpoint.searchParams.get("ticket"), "one-time-ticket");
    assert.equal(endpoint.searchParams.has("accessToken"), false);
    assert.equal(requests[0].url, "https://rooms.example/api/v2/rooms/AB12/socket-ticket");
    assert.equal(requests[0].options.headers["x-room-access-token"], "access");

    socket.readyState = MockWebSocket.OPEN;
    socket.onopen();
    socket.onmessage({
      data: '{"event":"connected","data":{"socketId":"socket-1"}}'
    });
    assert.equal(emitRoomCommand("clear-queue", {}), "sent");
    assert.equal(emitRoomHostState({ queue: [] }), "not-host");
    assert.deepEqual(JSON.parse(socket.sent[0]), {
      event: "room-command",
      data: { action: "clear-queue" }
    });
    socket.onmessage({
      data: '{"event":"room-state","data":{"code":"AB12","permissions":{"canAdd":true}},"sender":"socket-2"}'
    });
    assert.ok(drainRoomEvents().some(event => event.event === "room-state"));
  } finally {
    disconnectRoomSocket();
    globalThis.WebSocket = OriginalWebSocket;
    globalThis.window = originalWindow;
    globalThis.fetch = originalFetch;
  }
});

test("host room state is sent over the active socket", async () => {
  const OriginalWebSocket = globalThis.WebSocket;
  const originalWindow = globalThis.window;
  const originalFetch = globalThis.fetch;
  const sockets = [];
  class MockWebSocket {
    static OPEN = 1;
    readyState = MockWebSocket.OPEN;
    sent = [];
    constructor(url) {
      this.url = String(url);
      sockets.push(this);
    }
    send(message) {
      this.sent.push(message);
    }
    close() {
      this.readyState = 3;
    }
  }
  globalThis.WebSocket = MockWebSocket;
  globalThis.window = {
    setTimeout: () => 1,
    clearTimeout: () => {}
  };
  globalThis.fetch = async () => new Response('{"ticket":"host-ticket"}', {
    headers: { "Content-Type": "application/json" }
  });

  try {
    connectRoomSocket("https://rooms.example", "AB12", "Host", "access", "host-token", "host", {});
    await new Promise(resolve => setImmediate(resolve));
    const socket = sockets[0];
    socket.onopen();
    assert.equal(emitRoomHostState({ queue: [] }), "sent");
    assert.deepEqual(JSON.parse(socket.sent[0]), {
      event: "host-room-state",
      data: { queue: [] }
    });
  } finally {
    disconnectRoomSocket();
    globalThis.WebSocket = OriginalWebSocket;
    globalThis.window = originalWindow;
    globalThis.fetch = originalFetch;
  }
});

test("socket-ticket retries honor the server Retry-After header", async () => {
  const originalWindow = globalThis.window;
  const originalFetch = globalThis.fetch;
  const delays = [];
  globalThis.window = {
    setTimeout: (_callback, delay) => {
      delays.push(delay);
      return 1;
    },
    clearTimeout: () => {}
  };
  globalThis.fetch = async () => new Response('{"error":"Too many attempts"}', {
    status: 429,
    headers: { "Content-Type": "application/json", "Retry-After": "12" }
  });

  try {
    connectRoomSocket("https://rooms.example", "AB12", "Guest", "access", "", "guest", {});
    await new Promise(resolve => setImmediate(resolve));
    assert.deepEqual(delays, [12000]);
  } finally {
    disconnectRoomSocket();
    globalThis.window = originalWindow;
    globalThis.fetch = originalFetch;
  }
});

test("missing rooms end a saved session instead of retrying forever", async () => {
  const originalWindow = globalThis.window;
  const originalFetch = globalThis.fetch;
  let retryScheduled = false;
  globalThis.window = {
    setTimeout: () => {
      retryScheduled = true;
      return 1;
    },
    clearTimeout: () => {}
  };
  globalThis.fetch = async () => new Response('{"error":"Room not found"}', {
    status: 404,
    headers: { "Content-Type": "application/json" }
  });

  try {
    connectRoomSocket("https://rooms.example", "AB12", "Guest", "access", "", "guest", {});
    await new Promise(resolve => setImmediate(resolve));
    assert.ok(drainRoomEvents().some(event => event.event === "room-closed"));
    assert.equal(retryScheduled, false);
  } finally {
    disconnectRoomSocket();
    globalThis.window = originalWindow;
    globalThis.fetch = originalFetch;
  }
});

test("Spotify lyrics requests include album, duration, and source", async () => {
  const originalFetch = globalThis.fetch;
  const originalService = globalThis.__WAVELENGTH_AUDIO_SERVICE__;
  let requestUrl;
  globalThis.__WAVELENGTH_AUDIO_SERVICE__ = "https://audio.example";
  globalThis.fetch = async url => {
    requestUrl = String(url);
    return new Response('{"syncedLyrics":"[00:01.00]First line"}', {
      headers: { "Content-Type": "application/json" }
    });
  };

  try {
    const result = await fetchLyrics("Song title", "Artist", "Album", 213.4, "spotify");
    const query = new URL(requestUrl).searchParams;
    assert.equal(query.get("title"), "Song title");
    assert.equal(query.get("artist"), "Artist");
    assert.equal(query.get("album"), "Album");
    assert.equal(query.get("duration"), "213");
    assert.equal(query.get("source"), "spotify");
    assert.equal(result.syncedLyrics, "[00:01.00]First line");
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.__WAVELENGTH_AUDIO_SERVICE__ = originalService;
  }
});

test("YouTube lyrics requests carry duration without the placeholder album", async () => {
  const originalFetch = globalThis.fetch;
  const originalService = globalThis.__WAVELENGTH_AUDIO_SERVICE__;
  let requestUrl;
  globalThis.__WAVELENGTH_AUDIO_SERVICE__ = "https://audio.example";
  globalThis.fetch = async url => {
    requestUrl = String(url);
    return new Response('{"plainLyrics":"First line"}', {
      headers: { "Content-Type": "application/json" }
    });
  };

  try {
    await fetchLyrics("Artist - Song", "Artist - Topic", "YouTube", 182.5, "youtube");
    const query = new URL(requestUrl).searchParams;
    assert.equal(query.get("source"), "youtube");
    assert.equal(query.get("duration"), "183");
    assert.equal(query.has("album"), false);
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.__WAVELENGTH_AUDIO_SERVICE__ = originalService;
  }
});

test("active fullscreen lyrics are centered once per track and line", () => {
  const originalDocument = globalThis.document;
  const scrolls = [];
  const container = { dataset: {} };
  const line = {
    closest: () => container,
    scrollIntoView: options => scrolls.push(options)
  };
  globalThis.document = {
    getElementById: id => id === "fullscreen-lyric-4" ? line : null
  };

  try {
    assert.equal(scrollActiveLyricIntoView(4, "track-1"), true);
    assert.equal(scrollActiveLyricIntoView(4, "track-1"), true);
    assert.equal(scrollActiveLyricIntoView(4, "track-2"), true);
    assert.deepEqual(scrolls, [
      { block: "center", behavior: "smooth" },
      { block: "center", behavior: "smooth" }
    ]);
    assert.equal(scrollActiveLyricIntoView(5, "track-2"), false);
  } finally {
    globalThis.document = originalDocument;
  }
});

test("the desktop browser bridge exposes fullscreen lyric scrolling", () => {
  assert.equal(
    globalThis.__WAVELENGTH_BROWSER__.scrollActiveLyricIntoView,
    scrollActiveLyricIntoView
  );
});

test("lyrics requests retry temporary failures and cache successful results", async () => {
  const originalFetch = globalThis.fetch;
  const originalSetTimeout = globalThis.setTimeout;
  const originalService = globalThis.__WAVELENGTH_AUDIO_SERVICE__;
  const delays = [];
  let requests = 0;
  globalThis.__WAVELENGTH_AUDIO_SERVICE__ = "https://audio.example";
  globalThis.setTimeout = (callback, delay) => {
    delays.push(delay);
    callback();
    return 1;
  };
  globalThis.fetch = async () => {
    requests += 1;
    if (requests === 1) {
      return new Response('{"detail":"Temporarily unavailable"}', {
        status: 503,
        headers: {
          "Content-Type": "application/json",
          "Retry-After": "2"
        }
      });

    }
    return new Response('{"syncedLyrics":"[00:01.00]First line"}', {
      headers: { "Content-Type": "application/json" }
    });
  };

  try {
    const first = await fetchLyrics("Retry title", "Retry artist", "", 0, "spotify");
    const second = await fetchLyrics("Retry title", "Retry artist", "", 0, "spotify");
    assert.deepEqual(first, second);
    assert.equal(requests, 2);
    assert.deepEqual(delays, [2000]);
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.setTimeout = originalSetTimeout;
    globalThis.__WAVELENGTH_AUDIO_SERVICE__ = originalService;
  }
});

test("Spotify library import loads playlist artwork, saved albums, and playlist items", async () => {
  const originalFetch = globalThis.fetch;
  const requests = [];
  setSpotifyRefreshToken("test-refresh-token");
  globalThis.fetch = async (input, options) => {
    const url = new URL(input);
    requests.push({ url, options });
    const json = body => new Response(JSON.stringify(body), {
      headers: { "Content-Type": "application/json" }
    });

    if (url.hostname === "accounts.spotify.com") {
      return json({ access_token: "test-access-token", expires_in: 3600 });
    }
    if (url.pathname === "/v1/me/playlists") {
      return json({
        items: [{
          id: "playlist-1",
          name: "Saved mix",
          tracks: { total: 1 },
          images: [{ url: "https://images.example/playlist.jpg", width: 300, height: 300 }]
        }],
        next: null
      });
    }
    if (url.pathname === "/v1/me/albums") {
      return json({
        items: [{
          album: {
            id: "album-1",
            name: "Studio album",
            artists: [{ name: "Example artist" }],
            images: [{ url: "https://images.example/album.jpg", width: 300, height: 300 }],
            total_tracks: 1
          }
        }],
        next: null
      });
    }
    if (url.pathname === "/v1/playlists/playlist-1/items") {
      return json({
        items: [{
          item: {
            id: "track-1",
            name: "Playlist track",
            artists: [{ name: "Example artist" }],
            duration_ms: 180000
          }
        }],
        next: null
      });
    }
    if (url.pathname === "/v1/albums/album-1/tracks") {
      return json({
        items: [{
          id: "track-2",
          name: "Album track",
          artists: [{ name: "Example artist" }],
          duration_ms: 200000
        }],
        next: null
      });
    }
    throw new Error(`Unexpected Spotify request: ${url}`);
  };

  try {
    const playlists = await spotifyPlaylists();
    const albums = await spotifyAlbums();
    const playlistTracks = await spotifyPlaylistTracks(playlists[0].id);
    const albumTracks = await spotifyAlbumTracks(albums[0]);

    assert.equal(playlists[0].thumbnail, "https://images.example/playlist.jpg");
    assert.deepEqual(albums[0], {
      id: "album-1",
      name: "Studio album",
      artist: "Example artist",
      thumbnail: "https://images.example/album.jpg",
      tracks_total: 1
    });
    assert.equal(playlistTracks[0].source, "spotify");
    assert.equal(playlistTracks[0].title, "Playlist track");
    assert.equal(albumTracks[0].album, "Studio album");
    assert.equal(albumTracks[0].thumbnail, "https://images.example/album.jpg");
    assert.ok(requests.some(({ url }) => url.pathname === "/v1/playlists/playlist-1/items"));
  } finally {
    globalThis.fetch = originalFetch;
    setSpotifyRefreshToken("");
  }
});

test("Spotify playlist detail rejects repeated pagination URLs", async () => {
  const originalFetch = globalThis.fetch;
  const requests = [];
  setSpotifyRefreshToken("test-refresh-token");
  globalThis.fetch = async input => {
    const url = new URL(input);
    requests.push(url);
    return new Response(JSON.stringify({ items: [], next: url.href }), {
      headers: { "Content-Type": "application/json" }
    });
  };

  try {
    await assert.rejects(spotifyPlaylistTracks("playlist-loop"), /repeated playlist page/);
    assert.equal(requests.length, 1);
  } finally {
    globalThis.fetch = originalFetch;
    setSpotifyRefreshToken("");
  }
});

test("Spotify text search uses the restored in-memory refresh token", async () => {
  const originalFetch = globalThis.fetch;
  const originalLocalStorage = globalThis.localStorage;
  const requests = [];
  globalThis.localStorage = {
    getItem: () => null,
    setItem: () => {},
    removeItem: () => {}
  };
  setSpotifyRefreshToken("restored-refresh-token");
  globalThis.fetch = async (input, options) => {
    const url = new URL(input);
    requests.push({ url, options });
    if (url.hostname === "accounts.spotify.com") {
      return new Response('{"access_token":"restored-access-token","expires_in":3600}', {
        headers: { "Content-Type": "application/json" }
      });
    }
    return new Response(JSON.stringify({
      tracks: {
        items: [{
          id: "track-1",
          name: "Search result",
          artists: [{ name: "Artist" }],
          album: { name: "Album", images: [] },
          duration_ms: 120000
        }]
      }
    }), { headers: { "Content-Type": "application/json" } });
  };

  try {
    const results = await searchAudio("Search result", "spotify");
    assert.equal(results[0].title, "Search result");
    const spotifySearch = requests.find(({ url }) => url.hostname === "api.spotify.com" && url.pathname === "/v1/search");
    assert.ok(spotifySearch);
    assert.equal(spotifySearch.url.searchParams.get("limit"), "10");
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.localStorage = originalLocalStorage;
    setSpotifyRefreshToken("");
  }
});

test("Spotify search without a connection accepts links but rejects text queries", async () => {
  const originalFetch = globalThis.fetch;
  const originalLocalStorage = globalThis.localStorage;
  const originalService = globalThis.__WAVELENGTH_AUDIO_SERVICE__;
  const requests = [];
  globalThis.localStorage = {
    getItem: () => null,
    setItem: () => {},
    removeItem: () => {}
  };
  globalThis.__WAVELENGTH_AUDIO_SERVICE__ = "https://audio.example";
  setSpotifyRefreshToken("");
  globalThis.fetch = async input => {
    const url = new URL(input);
    requests.push(url);
    return new Response(JSON.stringify({
      results: [{ id: "track-1", title: "Linked track", source: "spotify" }]
    }), { headers: { "Content-Type": "application/json" } });
  };

  try {
    await assert.rejects(
      searchAudio("some text", "spotify"),
      /Connect Spotify to search by text/
    );
    const results = await searchAudio("https://open.spotify.com/track/track1?si=share", "spotify");
    assert.equal(results[0].title, "Linked track");
    assert.equal(requests.length, 1);
    assert.equal(requests[0].pathname, "/search");
    assert.equal(requests[0].searchParams.get("q"), "https://open.spotify.com/track/track1?si=share");
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.localStorage = originalLocalStorage;
    globalThis.__WAVELENGTH_AUDIO_SERVICE__ = originalService;
    setSpotifyRefreshToken("");
  }
});

test("YouTube text search uses the audio service for dropdown results", async () => {
  const originalFetch = globalThis.fetch;
  const originalLocalStorage = globalThis.localStorage;
  const originalService = globalThis.__WAVELENGTH_AUDIO_SERVICE__;
  const requests = [];
  globalThis.localStorage = {
    getItem: () => null,
    setItem: () => {},
    removeItem: () => {}
  };
  globalThis.__WAVELENGTH_AUDIO_SERVICE__ = "https://audio.example";
  globalThis.fetch = async input => {
    const url = new URL(input);
    requests.push(url);
    return new Response(JSON.stringify({
      results: [{ id: "video-1", title: "YouTube result", source: "youtube" }]
    }), { headers: { "Content-Type": "application/json" } });
  };

  try {
    const results = await searchAudio("some song", "youtube");
    assert.equal(results[0].title, "YouTube result");
    assert.equal(requests[0].searchParams.get("q"), "some song");
    assert.equal(requests[0].searchParams.get("source"), "youtube");
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.localStorage = originalLocalStorage;
    globalThis.__WAVELENGTH_AUDIO_SERVICE__ = originalService;
    setSpotifyRefreshToken("");
  }
});

test("Spotify playlist renames and track edits accept successful responses with an empty body", async () => {
  const originalFetch = globalThis.fetch;
  const requests = [];
  setSpotifyRefreshToken("test-refresh-token");
  globalThis.fetch = async (input, options) => {
    const url = new URL(input);
    requests.push({ url, options });
    if (url.hostname === "accounts.spotify.com") {
      return new Response('{"access_token":"test-access-token","expires_in":3600}', {
        headers: { "Content-Type": "application/json" }
      });
    }
    return new Response(null, { status: 200 });
  };

  try {
    await renameSpotifyPlaylist("playlist-1", "Updated mix");
    await saveSpotifyPlaylist("playlist-1", ["spotify:track:track-1"]);
    const spotifyRequests = requests.filter(({ url }) => url.hostname === "api.spotify.com");
    assert.deepEqual(
      spotifyRequests.map(({ url, options }) => ({
        path: url.pathname,
        method: options.method
      })),
      [
        { path: "/v1/playlists/playlist-1", method: "PUT" },
        { path: "/v1/playlists/playlist-1/items", method: "PUT" }
      ]
    );
    assert.deepEqual(
      spotifyRequests.map(({ options }) => JSON.parse(options.body)),
      [{ name: "Updated mix" }, { uris: ["spotify:track:track-1"] }]
    );
  } finally {
    globalThis.fetch = originalFetch;
    setSpotifyRefreshToken("");
  }
});

test("Spotify album detail times out while the response body is stalled", async () => {
  const originalFetch = globalThis.fetch;
  const originalSetTimeout = globalThis.setTimeout;
  setSpotifyRefreshToken("test-refresh-token");
  globalThis.setTimeout = callback => {
    callback();
    return 1;
  };
  globalThis.fetch = async (_input, options) => ({
    status: 200,
    ok: true,
    json: () => options.signal.aborted
      ? Promise.reject(new DOMException("Aborted", "AbortError"))
      : new Promise(() => {})
  });

  try {
    await assert.rejects(
      spotifyAlbumTracks({ id: "album-stalled" }),
      /did not respond in time/
    );
  } finally {
    globalThis.fetch = originalFetch;
    globalThis.setTimeout = originalSetTimeout;
    setSpotifyRefreshToken("");
  }
});
