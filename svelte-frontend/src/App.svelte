<script>
  import { onMount, tick } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import brandMark from "../src-tauri/icons/128x128.png";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { check as checkDesktopUpdate } from "@tauri-apps/plugin-updater";
  import {
    Check,
    Copy,
    Download,
    Home,
    Library,
    Link,
    ListMusic,
    Minus,
    Play,
    Plus,
    Search,
    Square,
    Trash2,
    X,
    RefreshCw,
    Disc3,
    Clock3,
    Users,
    FolderOpen,
    ChevronLeft,
    ChevronRight,
  } from "lucide-svelte";
  import { parseBlob } from "music-metadata";
  import { io as createSocket } from "socket.io-client";
  import SearchResultsPage from "./components/SearchResultsPage.svelte";
  import TrackContextMenu from "./components/TrackContextMenu.svelte";
  import SongDetailsModal from "./components/SongDetailsModal.svelte";
  import HomePage from "./components/pages/HomePage.svelte";
  import LibraryPage from "./components/pages/LibraryPage.svelte";
  import PlaylistsPage from "./components/pages/PlaylistsPage.svelte";
  import CollectionDetailPage from "./components/pages/CollectionDetailPage.svelte";
  import QueuePage from "./components/pages/QueuePage.svelte";
  import SettingsPage from "./components/pages/SettingsPage.svelte";
  import CustomizationPage from "./components/pages/CustomizationPage.svelte";
  import ActivityPage from "./components/pages/ActivityPage.svelte";
  import EqualizerPage from "./components/pages/EqualizerPage.svelte";
  import NowPlayingPage from "./components/pages/NowPlayingPage.svelte";
  import FullscreenLyrics from "./components/FullscreenLyrics.svelte";
  import PlayerBar from "./components/PlayerBar.svelte";
  import { marquee } from "./marquee.js";
  import ListeningHistory from "./ListeningHistory.svelte";
  import SelectMenu from "./SelectMenu.svelte";

  const isLocalPlayer = () => isTauri() || import.meta.env.DEV;
  const appWindow = isTauri() ? getCurrentWindow() : null;
  const equalizerBands = [
    60, 170, 310, 600, 1000, 3000, 6000, 12000, 14000, 16000,
  ];
  const builtInEqualizerPresets = [
    { id: "flat", name: "Flat", gains: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { id: "bass", name: "Bass boost", gains: [6, 5, 4, 2, 0, 0, 0, 0, 0, 0] },
    { id: "vocal", name: "Vocal", gains: [-2, -1, 0, 2, 4, 4, 2, 0, -1, -2] },
    {
      id: "treble",
      name: "Treble boost",
      gains: [0, 0, 0, 0, 0, 1, 3, 5, 6, 6],
    },
    { id: "rock", name: "Rock", gains: [4, 3, 1, -1, -2, 1, 3, 4, 4, 4] },
  ];
  const equalizerNodeColors = [
    "#d8bd69",
    "#d99163",
    "#d27879",
    "#bd7fb0",
    "#9c8fd0",
    "#7799d0",
    "#63aaa9",
    "#78ae79",
    "#aaba6d",
    "#c9b979",
  ];
  const equalizerTypeOptions = [
    { value: "peaking", label: "Bell" },
    { value: "lowshelf", label: "Low shelf" },
    { value: "highshelf", label: "High shelf" },
  ];

  const views = [
    { id: "overview", path: "/player", label: "Home", icon: Home },
  ];
  const searchSourceOptions = [
    { value: "spotify", label: "Spotify" },
    { value: "youtube", label: "YouTube" },
  ];
  const previewPage = new URLSearchParams(window.location.search).get("preview");
  const previewMode =
    import.meta.env.DEV && ["search", "logs"].includes(previewPage);
  const previewTracks = [
    {
      id: "demo-midnight-city-lights",
      title: "Midnight City Lights",
      artist: "The Glass Hours",
      album: "After Hours",
      length: "3:48",
      source: "spotify",
    },
    {
      id: "demo-slow-motion",
      title: "Slow Motion",
      artist: "June Arcade",
      album: "Soft Focus",
      length: "4:12",
      source: "spotify",
    },
    {
      id: "demo-golden",
      title: "Golden",
      artist: "Mira Sol",
      album: "Little Weather",
      length: "3:26",
      source: "spotify",
    },
    {
      id: "demo-satellite-heart",
      title: "Satellite Heart",
      artist: "Northbound",
      album: "Signals at Dawn",
      length: "5:03",
      source: "spotify",
    },
    {
      id: "demo-blue-hour",
      title: "Blue Hour",
      artist: "The Glass Hours",
      album: "After Hours",
      length: "3:57",
      source: "spotify",
    },
  ];
  const previewDownloadLogs = [
    {
      timestamp: Date.now() - 45_000,
      level: "info",
      source: "Spotify",
      message: "Audio source matched and added to the preparation queue.",
      trackId: "demo-01J9YQ9XK4N8CZ6P7TSM2R5A1B",
      progress: 100,
    },
    {
      timestamp: Date.now() - 91_000,
      level: "info",
      source: "YouTube",
      message: "Preparing a high-quality audio stream for playback.",
      trackId: "demo-01J9YQ9XK4N8CZ6P7TSM2R5A1C",
      progress: 68,
    },
    {
      timestamp: Date.now() - 148_000,
      level: "warning",
      source: "Spotify",
      message:
        "The first match was unavailable; retrying with an alternate source and checking metadata before download.",
      trackId: "demo-01J9YQ9XK4N8CZ6P7TSM2R5A1D",
      progress: 34,
    },
    {
      timestamp: Date.now() - 223_000,
      level: "info",
      source: "Local library",
      message: "Artwork and track details loaded from the selected folder.",
      trackId: "demo-01J9YQ9XK4N8CZ6P7TSM2R5A1E",
      progress: null,
    },
    {
      timestamp: Date.now() - 302_000,
      level: "error",
      source: "Qobuz",
      message:
        "The audio service could not open this candidate. Confirm the service is available, then retry the download.",
      trackId: "demo-01J9YQ9XK4N8CZ6P7TSM2R5A1F",
      progress: null,
    },
  ];
  const previewNotifications = [
    { timestamp: Date.now() - 32_000, message: "Added Slow Motion to the queue." },
    { timestamp: Date.now() - 116_000, message: "Download complete: Midnight City Lights." },
    { timestamp: Date.now() - 264_000, message: "Playback resumed." },
  ];
  const previewQueue = previewTracks.slice(1, 4).map((track) => ({
    ...track,
    id: `preview-${track.id.slice(5)}`,
  }));
  const previewQueueDownloadStatuses = Object.fromEntries(
    previewQueue.map((track, index) => [
      track.id,
      { status: "downloading", progress: [38, 66, 91][index] },
    ]),
  );
  const statsPeriodOptions = [
    { value: "week", label: "1 week" },
    { value: "month", label: "1 month" },
    { value: "3months", label: "3 months" },
    { value: "6months", label: "6 months" },
    { value: "year", label: "1 year" },
    { value: "all", label: "All time" },
  ];
  const importedSortOptions = [
    { value: "folders", label: "Folders" },
    { value: "album", label: "Album" },
    { value: "artist", label: "Artist" },
  ];
  const themeOptions = [
    { value: "system", label: "System" },
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
  ];
  const appFoldersKey = "folders:app";
  let tracks = [];
  let onlineTracks = [];
  let importedTracks = [];
  let importedSearch = "";
  let importedSortBy = "folders";
  let expandedImportedFolders = {};

  let activeView = previewMode && previewPage === "search" ? "search" : "overview";
  let search = previewMode && previewPage === "search" ? "after hours" : "";
  let searchSource = "spotify";
  let topbarSearchOpen = false;
  let topbarSearchTimer = null;
  let searchRequestId = 0;
  let searchAbortController = null;
  let searchOffset = 0;
  let searchTotal = 0;
  let searchHasMore = false;
  let searchLoadingMore = false;
  let searchLoading = false;
  let searchAtBottom = false;
  let mobileNav = false;
  let sidebarCollapsed = false;
  let mobileViewport = window.innerWidth <= 735;
  let playing = null;
  let isPlaying = false;
  let joinedParty = localStorage.getItem("wavelength-party-code") || "";
  let partyHostToken =
    localStorage.getItem("wavelength-party-host-token") || "";
  let roomServerUrl = "";
  let roomServerInput = "";
  let localAudioBasePromise = null;
  let roomServerUrlLoaded = false;
  let roomServerSaving = false;
  let partyMembers = [];
  let partyHostName = "";
  let partyPermissions = {
    canAdd: false,
    canManageQueue: false,
    canControlPlayback: false,
  };
  const guestPermissionOptions = [
    { key: "canAdd", label: "Allow guests to add tracks" },
    { key: "canManageQueue", label: "Allow guests to manage queue" },
    { key: "canControlPlayback", label: "Allow guests to control playback" },
  ];
  let partyRole = localStorage.getItem("wavelength-party-role") || "";
  let partyDisplayName = localStorage.getItem("wavelength-party-name") || "";
  $: guestCanManageQueue =
    partyRole !== "guest" || Boolean(partyPermissions.canManageQueue);
  $: guestCanControlPlayback =
    partyRole !== "guest" || Boolean(partyPermissions.canControlPlayback);
  let pendingInviteCode = (
    new URLSearchParams(window.location.search).get("party") ||
    localStorage.getItem("wavelength-pending-invite") ||
    ""
  )
    .trim()
    .toUpperCase();
  let roomSessionReady = !pendingInviteCode || !joinedParty;
  let inviteGuestName = "";
  $: inviteGateRequired = Boolean(
    pendingInviteCode &&
      (!roomSessionReady || joinedParty !== pendingInviteCode),
  );
  let appTheme = localStorage.getItem("wavelength-theme") || "dark";
  let appAccent = localStorage.getItem("wavelength-accent") || "#d7c7a4";
  let appBackgroundColor = (() => {
    const saved = localStorage.getItem("wavelength-background-color") || "";
    return /^#[a-f\d]{6}$/i.test(saved) ? saved : "";
  })();
  let customAccentInput = appAccent;
  let customAccentError = "";
  let systemThemeMedia = null;
  const uiScaleStorageKey = "wavelength-ui-scale";
  const uiScaleMinimum = 0.75;
  const uiScaleMaximum = 1.5;
  let uiScale = (() => {
    const savedScale = localStorage.getItem(uiScaleStorageKey);
    const parsedScale = savedScale === null ? 1 : Number(savedScale);
    return Number.isFinite(parsedScale)
      ? Math.min(uiScaleMaximum, Math.max(uiScaleMinimum, parsedScale))
      : 1;
  })();

  function resolvedTheme() {
    if (appTheme !== "system") return appTheme;
    const systemPrefersDark =
      systemThemeMedia?.matches ??
      window.matchMedia("(prefers-color-scheme: dark)").matches;
    return systemPrefersDark ? "dark" : "light";
  }
  const accentOptions = [
    { name: "Orange", value: "#fb6b1d" },
    { name: "Red", value: "#e83b3b" },
    { name: "Plum", value: "#831c5d" },
    { name: "Magenta", value: "#c32454" },
    { name: "Pink", value: "#f04f78" },
    { name: "Coral pink", value: "#f68181" },
    { name: "Sand", value: "#ab947a" },
    { name: "Teal", value: "#0b5e65" },
    { name: "Cyan", value: "#0b8a8f" },
    { name: "Green", value: "#1ebc73" },
    { name: "Lime", value: "#91db69" },
    { name: "Gold", value: "#fbb954" },
    { name: "Copper", value: "#cd683d" },
    { name: "Rust", value: "#9e4539" },
    { name: "Wine", value: "#7a3045" },
    { name: "Violet", value: "#6b3e75" },
    { name: "Purple", value: "#905ea9" },
    { name: "Lavender", value: "#a884f3" },
    { name: "Blue", value: "#4d9be6" },
    { name: "Indigo", value: "#4d65b4" },
  ];
  let partyRefreshTimer = null;
  let partySocket = null;
  let pendingPartySocketState = null;
  let partyPublishTimer = null;
  let partyRefreshInFlight = false;
  let partyPublishInFlight = false;
  let partyPublishPending = false;
  let partyPlaybackSyncInFlight = false;
  let partyProgressTimer = null;
  let partyProgressLastSentAt = 0;
  let roomSyncRevision = 0;
  let lastAppliedRoomRevision = -1;
  let partyCode = joinedParty || pendingInviteCode;
  let roomNameMode = "custom";
  let inviteCopied = false;
  let codeCopied = false;
  let inviteCopyTimer = null;
  let username = localStorage.getItem("wavelength-username") || "";
  let showFolderManager = false;
  let qobuzCredentialsOpen = false;
  let showAppUpdatePrompt = false;
  let pendingDesktopUpdate = null;
  let appUpdateInstalling = false;
  let contentModalBounds = {
    left: 0,
    top: 0,
    contentLeft: 0,
    contentTop: 0,
    width: 0,
    height: 0,
    centerX: 0,
    centerY: 0,
  };
  let showCreatePlaylist = false;
  let createPlaylistName = "";
  let createPlaylistTrack = null;
  let createPlaylistInput;
  let showAccentPicker = false;
  let downloadDirectoryHandle = null;
  let downloadDirectoryName = "";
  let downloadDirectoryLoaded = false;
  let downloadFolderPromptShown = false;
  const playedTrackDownloads = new Map();
  $: downloadFolderReady =
    downloadDirectoryLoaded && Boolean(downloadDirectoryName.trim());
  let qobuzCredentials = (() => {
    try {
      const saved = JSON.parse(
        localStorage.getItem("wavelength-qobuz-credentials") || "{}",
      );
      return saved && typeof saved === "object" && !Array.isArray(saved)
        ? saved
        : {};
    } catch {
      return {};
    }
  })();
  let qobuzConfigured = false;
  let qobuzSaving = false;
  let appUpdateStartupTimer = null;
  let appUpdatePollingTimer = null;
  let appUpdateChecking = false;
  let appUpdateStatus = "";
  let localFolders = [];
  let localUploadCache = (() => {
    try {
      return JSON.parse(
        localStorage.getItem("wavelength-shared-audio") || "{}",
      );
    } catch {
      return {};
    }
  })();
  let notice = "";
  let noticeTimer = null;
  let noticeDeadline = 0;
  let noticeRemaining = 3000;
  $: if (notice) scheduleNoticeDismiss();
  let audioUrl = "";
  let audioElement;
  let pendingHistoryTrack = null;
  let playbackRequestId = 0;
  let playbackRequestPending = false;
  let lastPlaybackPersistedAt = 0;
  let pendingPlaybackSeek = null;
  let contentInner;
  let searchTable;
  let recentAddedGrid;
  let contentHasOverflow = false;
  let contentDrag = null;
  let isContentDragging = false;
  let suppressDragClick = false;
  let currentTime = 0;
  let duration = 0;
  let volume = (() => {
    try {
      const savedVolume = localStorage.getItem("wavelength-volume");
      if (savedVolume === null) return 0.3;
      const parsedVolume = Number(savedVolume);
      return Number.isFinite(parsedVolume)
        ? Math.min(1, Math.max(0, parsedVolume))
        : 0.3;
    } catch {
      return 0.3;
    }
  })();
  let equalizerContext = null;
  let equalizerSource = null;
  let equalizerAnalyser = null;
  let equalizerSpectrumCanvas;
  let equalizerSpectrumFrame = 0;
  let equalizerSpectrumData;
  let equalizerFilters = [];
  let equalizerUnavailable = false;
  let equalizerPresetName = "";
  let selectedEqualizerBand = 0;
  let equalizerDragBand = -1;
  let customEqualizerPresets = (() => {
    try {
      const saved = JSON.parse(
        localStorage.getItem("wavelength-equalizer-presets") || "[]",
      );
      return Array.isArray(saved)
        ? saved.filter(
            (preset) =>
              preset &&
              typeof preset.id === "string" &&
              typeof preset.name === "string" &&
              Array.isArray(preset.gains) &&
              preset.gains.length === equalizerBands.length,
          )
        : [];
    } catch {
      return [];
    }
  })();
  let equalizerGains = (() => {
    try {
      const saved = JSON.parse(
        localStorage.getItem("wavelength-equalizer-gains") || "null",
      );
      if (Array.isArray(saved) && saved.length === equalizerBands.length)
        return saved.map((gain) =>
          Math.max(-18, Math.min(18, Number(gain) || 0)),
        );
    } catch {}
    return [...builtInEqualizerPresets[0].gains];
  })();
  let equalizerFrequencies = (() => {
    try {
      const saved = JSON.parse(
        localStorage.getItem("wavelength-equalizer-frequencies") || "null",
      );
      if (Array.isArray(saved) && saved.length === equalizerBands.length)
        return saved.map((frequency, index) => {
          const value = Number(frequency);
          return Number.isFinite(value)
            ? Math.max(20, Math.min(20000, value))
            : equalizerBands[index];
        });
    } catch {}
    return [...equalizerBands];
  })();
  let equalizerQs = (() => {
    try {
      const saved = JSON.parse(
        localStorage.getItem("wavelength-equalizer-qs") || "null",
      );
      if (Array.isArray(saved) && saved.length === equalizerBands.length)
        return saved.map((value) =>
          Math.max(0.3, Math.min(10, Number(value) || 1.1)),
        );
    } catch {}
    return equalizerBands.map(() => 1.1);
  })();
  let equalizerTypes = (() => {
    const validTypes = equalizerTypeOptions.map((option) => option.value);
    try {
      const saved = JSON.parse(
        localStorage.getItem("wavelength-equalizer-types") || "null",
      );
      if (Array.isArray(saved) && saved.length === equalizerBands.length)
        return saved.map((type) =>
          validTypes.includes(type) ? type : "peaking",
        );
    } catch {}
    return equalizerBands.map(() => "peaking");
  })();
  let activeEqualizerPresetId =
    localStorage.getItem("wavelength-equalizer-preset") || "flat";
  if (
    activeEqualizerPresetId !== "custom" &&
    ![...builtInEqualizerPresets, ...customEqualizerPresets].some(
      (preset) => preset.id === activeEqualizerPresetId,
    )
  )
    activeEqualizerPresetId = "flat";
  $: equalizerPresetOptions = [
    { value: "custom", label: "Custom" },
    ...builtInEqualizerPresets.map((preset) => ({
      value: preset.id,
      label: preset.name,
    })),
    ...customEqualizerPresets.map((preset) => ({
      value: preset.id,
      label: preset.name,
    })),
  ];
  let previousVolume = (() => {
    try {
      const saved = Number(localStorage.getItem("wavelength-previous-volume"));
      if (Number.isFinite(saved) && saved > 0) return Math.min(1, saved);
    } catch {}
    return volume || 0.3;
  })();
  let isMuted = volume === 0;
  let volumeSliderOpen = false;
  let queueVisible = true;
  let draggedQueueIndex = null;
  let dragOverQueueIndex = null;
  let queuePointerGesture = null;
  let queuePointerHoldTimer = null;
  let lyricsFullscreen = false;
  let syncedLyricLines = [];
  let activeLyricIndex = -1;
  let lyricsList;
  let lyricRequestId = 0;
  $: if (lyricsFullscreen && activeLyricIndex >= 0 && lyricsList)
    tick().then(() =>
      lyricsList
        ?.querySelector(".active")
        ?.scrollIntoView({ block: "center", behavior: "smooth" }),
    );
  let partyDetailsVisible = false;
  let appNavigationStack = (() => {
    try {
      return JSON.parse(
        sessionStorage.getItem("wavelength-navigation-stack") || "[]",
      );
    } catch {
      return [];
    }
  })();
  if (!appNavigationStack.length)
    appNavigationStack = [window.location.pathname];
  let forwardNavigationStack = (() => {
    try {
      const saved = JSON.parse(
        sessionStorage.getItem("wavelength-navigation-forward-stack") || "[]",
      );
      return Array.isArray(saved) ? saved : [];
    } catch {
      return [];
    }
  })();
  $: railView = partyDetailsVisible ? "party" : queueVisible ? "queue" : null;
  $: if (
    (showFolderManager || qobuzCredentialsOpen || showAppUpdatePrompt || showCreatePlaylist || showAccentPicker) &&
    contentInner
  ) {
    railView;
  }
  let railWidth = 320;
  let resizingRail = false;
  let libraryFilter = "imported";
  let playHistory = JSON.parse(
    localStorage.getItem("wavelength-play-history") || "[]",
  );
  let playCounts = JSON.parse(
    localStorage.getItem("wavelength-play-counts") || "{}",
  );
  let playEvents = JSON.parse(
    localStorage.getItem("wavelength-play-events") || "[]",
  );
  let statsPeriod = "all";
  let spotifyClientId =
    localStorage.getItem("wavelength-spotify-client-id") || "";
  let spotifyConnected =
    localStorage.getItem("wavelength-spotify-connected") === "true";
  let spotifyProfile = null;
  let spotifyPlaylists = [];
  let spotifySavedTracks = [];
  let spotifyAccessToken = "";
  let spotifyAccessTokenExpiresAt = 0;
  let spotifyRefreshToken = "";
  let spotifyRefreshPromise = null;
  let pendingSpotifyOAuth = null;
  let appVersion = isTauri() ? "Loading" : "Web app";
  let selectedCollectionId = "";
  let shuffleOn = false;
  let repeatMode = "off";
  let repeatOn = false;
  $: repeatOn = repeatMode !== "off";
  $: repeatOn = repeatMode !== "off";
  let queue = previewMode
    ? previewQueue
    : JSON.parse(localStorage.getItem("wavelength-queue") || "[]");
  let removedQueueHistory = [];
  let queueDownloadStatuses = previewMode ? previewQueueDownloadStatuses : {};
  let downloadCompletionMarks = {};
  let queueStatusRefreshInFlight = false;
  let downloadLogs =
    previewMode && previewPage === "logs" ? previewDownloadLogs : [];
  let downloadLogsError = "";
  let downloadLogsLoading = false;
  let notificationLogs = previewMode && previewPage === "logs"
    ? previewNotifications
    : (() => {
    try {
      const saved = JSON.parse(
        localStorage.getItem("wavelength-notification-logs") || "[]",
      );
      return Array.isArray(saved)
        ? saved
            .filter(
              (entry) =>
                entry &&
                typeof entry.message === "string" &&
                Number.isFinite(Number(entry.timestamp)),
            )
            .slice(0, 200)
        : [];
    } catch {
      return [];
    }
      })();
  let downloadRefreshTimer = null;
  let logsRefreshTimer = null;
  let favorites = JSON.parse(
    localStorage.getItem("wavelength-favorites") || "[]",
  );
  let playlists = JSON.parse(
    localStorage.getItem("wavelength-playlists") ||
      '[{"id":"liked-songs","name":"Liked Songs","tracks":[]}]',
  );
  let sharedPlaylists = [];
  let searchResults =
    previewMode && previewPage === "search" ? previewTracks : [];
  let topbarSearchResults = [];
  let lyrics = "";
  let nowPlayingMetadata = {};
  let nowPlayingMetadataTrackKey = "";
  let nowPlayingMetadataRequest = 0;
  $: {
    const trackKey = playing
      ? `${getTrackSource(playing)}:${playing.id || playing.title}`
      : "";
    if (trackKey !== nowPlayingMetadataTrackKey) {
      nowPlayingMetadataTrackKey = trackKey;
      loadNowPlayingMetadata(playing);
    }
  }
  let contextMenu = null;
  let playlistAddTarget = null;
  let contextMenuTimer = null;
  let songDetailsTrack = null;
  if (
    !playlists.some(
      (playlist) =>
        playlist.id === "liked-songs" || playlist.name === "Liked Songs",
    )
  )
    playlists = [
      { id: "liked-songs", name: "Liked Songs", tracks: [] },
      ...playlists,
    ];

  function folderDatabase() {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open("wavelength-library", 1);
      request.onupgradeneeded = () =>
        request.result.createObjectStore("folders");
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  function idbRequest(request) {
    return new Promise((resolve, reject) => {
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  function idbTransaction(transaction) {
    return new Promise((resolve, reject) => {
      transaction.oncomplete = resolve;
      transaction.onerror = () => reject(transaction.error);
      transaction.onabort = () => reject(transaction.error);
    });
  }

  async function loadSavedFolders() {
    if (!window.indexedDB) return;
    try {
      const database = await folderDatabase();
      const store = database
        .transaction("folders", "readonly")
        .objectStore("folders");
      const [keys, values] = await Promise.all([
        idbRequest(store.getAllKeys()),
        idbRequest(store.getAll()),
      ]);
      const entries = new Map(keys.map((key, index) => [key, values[index]]));
      const hasAppFolders = entries.has(appFoldersKey);
      const appHandles = entries.get(appFoldersKey) || [];
      const legacyHandles = keys
        .filter(
          (key) =>
            typeof key === "string" &&
            key !== appFoldersKey &&
            !key.startsWith("downloads:"),
        )
        .flatMap((key) =>
          Array.isArray(entries.get(key)) ? entries.get(key) : [],
        );
      const handles = hasAppFolders
        ? appHandles
        : [
            ...new Map(
              legacyHandles.map((handle) => [handle.name, handle]),
            ).values(),
          ];
      if (!hasAppFolders && handles.length) {
        const transaction = database.transaction("folders", "readwrite");
        transaction.objectStore("folders").put(handles, appFoldersKey);
        await idbTransaction(transaction);
      }
      localFolders = handles.map((handle) => ({ name: handle.name, handle }));
    } catch {
      notice = "Could not load saved folders.";
    }
  }

  async function saveFolders() {
    if (!window.indexedDB) return;
    const database = await folderDatabase();
    const transaction = database.transaction("folders", "readwrite");
    transaction.objectStore("folders").put(
      localFolders.map((folder) => folder.handle),
      appFoldersKey,
    );
    await idbTransaction(transaction);
  }

  async function loadDownloadDirectory() {
    if (isTauri()) {
      try {
        const directory = await invoke("get_download_directory");
        downloadDirectoryHandle = null;
        downloadDirectoryName = directory
          ? directory.split(/[\\/]/).filter(Boolean).at(-1) || directory
          : "";
      } catch {
        downloadDirectoryName = "";
      } finally {
        downloadDirectoryLoaded = true;
      }
      return;
    }
    if (!window.indexedDB) {
      downloadDirectoryLoaded = true;
      return;
    }
    try {
      const database = await folderDatabase();
      const transaction = database.transaction("folders", "readonly");
      const request = transaction
        .objectStore("folders")
        .get(`downloads:${username || "guest"}`);
      await new Promise((resolve) => {
        request.onsuccess = () => {
          downloadDirectoryHandle = request.result || null;
          downloadDirectoryName = downloadDirectoryHandle?.name || "";
          downloadDirectoryLoaded = true;
          resolve();
        };
        request.onerror = () => {
          downloadDirectoryLoaded = true;
          resolve();
        };
      });
    } catch {
      downloadDirectoryLoaded = true;
    }
  }

  async function chooseDownloadDirectory() {
    if (isTauri()) {
      try {
        const directory = await invoke("choose_download_directory");
        if (!directory) return;
        downloadDirectoryName =
          directory.split(/[\\/]/).filter(Boolean).at(-1) || directory;
        downloadDirectoryLoaded = true;
        downloadFolderPromptShown = false;
        if (isPlaying) savePlayedTrack(playing);
        notice = `Downloads will be saved to ${downloadDirectoryName}.`;
      } catch (error) {
        notice = error.message || "Could not set the downloads folder.";
      }
      return;
    }
    if (!window.isSecureContext) {
      notice = "Folder access requires HTTPS or this device's localhost address.";
      return;
    }
    if (!window.indexedDB) {
      notice = "Folder access is unavailable in this browser.";
      return;
    }
    if (!window.showDirectoryPicker) {
      notice = "Choose a downloads folder using Chrome or Edge.";
      return;
    }
    try {
      const handle = await window.showDirectoryPicker();
      const database = await folderDatabase();
      const transaction = database.transaction("folders", "readwrite");
      transaction
        .objectStore("folders")
        .put(handle, `downloads:${username || "guest"}`);
      await new Promise((resolve, reject) => {
        transaction.oncomplete = resolve;
        transaction.onerror = () => reject(transaction.error);
        transaction.onabort = () => reject(transaction.error);
      });
      downloadDirectoryHandle = handle;
      downloadDirectoryName = handle.name;
      downloadDirectoryLoaded = true;
      downloadFolderPromptShown = false;
      if (isPlaying) savePlayedTrack(playing);
      notice = `Downloads will be saved to ${handle.name}.`;
    } catch (error) {
      notice = error?.name === "AbortError"
        ? "No downloads folder was selected."
        : error.message || "Could not set the downloads folder.";
    }
  }

  async function scanFolder(folder, requestAccess = false) {
    let permission = await folder.handle.queryPermission?.({ mode: "read" });
    if (permission !== "granted" && requestAccess)
      permission = await folder.handle.requestPermission?.({ mode: "read" });
    if (permission !== "granted") {
      if (requestAccess)
        throw new Error("Folder permission was not granted.");
      return [];
    }
    const found = [];
    async function scanDirectory(directory, folderPath) {
      for await (const entry of directory.values()) {
        if (entry.kind === "directory") {
          await scanDirectory(entry, `${folderPath}/${entry.name}`);
          continue;
        }
        if (!/\.(mp3|flac|m4a|ogg|wav|aac)$/i.test(entry.name)) continue;
        const file = await entry.getFile();
        let metadata = null;
        try {
          metadata = await parseBlob(file);
        } catch {}
        const duration = metadata?.format.duration;
        const id = `local-${folderPath}-${entry.name}`;
        const fingerprint = `${file.size}:${file.lastModified}`;
        const upload = localUploadCache[id];
        found.push({
          id,
          title: metadata?.common.title || entry.name.replace(/\.[^.]+$/, ""),
          artist:
            metadata?.common.artist ||
            metadata?.common.albumartist ||
            "Local file",
          album: metadata?.common.album || "",
          length: duration ? formatTime(duration) : "—",
          color: "#718979",
          folderPath,
          file,
          serverAudioId:
            upload?.fingerprint === fingerprint &&
            Date.now() - upload.uploadedAt < 7 * 24 * 60 * 60 * 1000
              ? upload.audioId
              : "",
        });
      }
    }
    await scanDirectory(folder.handle, folder.name);
    return found;
  }

  async function reloadImportedTracks(requestAccess = false) {
    const batches = await Promise.all(
      localFolders.map((folder) => scanFolder(folder, requestAccess)),
    );
    importedTracks = batches.flat();
    const importedById = new Map(
      importedTracks.map((track) => [track.id, track]),
    );
    favorites = favorites.map((track) => importedById.get(track.id) || track);
    tracks = [...onlineTracks, ...importedTracks];
  }

  function persistLastPlayback(force = false) {
    if (!playing) return;
    const now = Date.now();
    if (!force && now - lastPlaybackPersistedAt < 1500) return;
    const artwork = getTrackArt(playing);
    const snapshot = {
      id: String(playing.id || ""),
      title: String(playing.title || ""),
      artist: String(playing.artist || ""),
      album: String(playing.album || ""),
      source: getTrackSource(playing).toLowerCase(),
      videoId: String(playing.videoId || ""),
      thumbnail:
        typeof artwork === "string" && /^https?:\/\//i.test(artwork)
          ? artwork
          : "",
      color: String(playing.color || ""),
      length: String(playing.length || ""),
      duration: Number(duration) || 0,
      currentTime: Number(currentTime) || 0,
    };
    try {
      localStorage.setItem(
        "wavelength-last-playback",
        JSON.stringify(snapshot),
      );
    } catch {}
    lastPlaybackPersistedAt = now;
  }

  function restoreLastPlayback() {
    let snapshot;
    try {
      snapshot = JSON.parse(
        localStorage.getItem("wavelength-last-playback") || "null",
      );
    } catch {}
    if (!snapshot?.id) snapshot = playHistory[0];
    if (!snapshot?.id) return;
    const savedTrack = [...tracks, ...queue, ...favorites, ...playHistory].find(
      (track) => track.id === snapshot.id,
    );
    const resumeAt = Math.max(0, Number(snapshot.currentTime) || 0);
    playing = { ...snapshot, ...(savedTrack || {}), id: snapshot.id, resumeAt };
    currentTime = resumeAt;
    duration = Math.max(0, Number(snapshot.duration) || 0);
    isPlaying = false;
    audioUrl = "";
    loadLyrics(playing);
    persistLastPlayback(true);
  }

  function rememberUploadedTrack(track, audioId) {
    const fingerprint = `${track.file.size}:${track.file.lastModified}`;
    localUploadCache = {
      ...localUploadCache,
      [track.id]: { fingerprint, audioId, uploadedAt: Date.now() },
    };
    localStorage.setItem(
      "wavelength-shared-audio",
      JSON.stringify(localUploadCache),
    );
    const uploaded = { ...track, serverAudioId: audioId };
    importedTracks = importedTracks.map((item) =>
      item.id === track.id ? uploaded : item,
    );
    tracks = tracks.map((item) => (item.id === track.id ? uploaded : item));
    favorites = favorites.map((item) =>
      item.id === track.id ? uploaded : item,
    );
    queue = queue.map((item) => (item.id === track.id ? uploaded : item));
    if (playing?.id === track.id) playing = uploaded;
    return uploaded;
  }

  async function ensureLocalTrackUploaded(track) {
    if (
      !track?.file ||
      track.serverAudioId ||
      !joinedParty ||
      partyRole !== "host" ||
      !roomServerUrl
    )
      return track;
    const extension = track.file.name.split(".").pop()?.toLowerCase() || "";
    const result = await api(
      `/parties/${encodeURIComponent(joinedParty)}/audio`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/octet-stream",
          "X-Audio-Extension": extension,
        },
        body: track.file,
      },
    );
    return rememberUploadedTrack(track, result.audioId);
  }

  async function uploadQueuedLocalTracks() {
    if (!joinedParty || partyRole !== "host" || !roomServerUrl) return;
    let failed = 0;
    for (const track of queue) {
      if (!track.file || track.serverAudioId) continue;
      try {
        await ensureLocalTrackUploaded(track);
      } catch {
        failed += 1;
      }
    }
    if (failed)
      notice = `${failed} local ${failed === 1 ? "track was" : "tracks were"} not uploaded.`;
  }

  function isImportedTrack(track) {
    return Boolean(
      track.file || track.artist === "Local file" || track.source === "local",
    );
  }
  function statsCutoff(period, now = new Date()) {
    if (period === "week") return now.getTime() - 7 * 24 * 60 * 60 * 1000;
    const months = { month: 1, "3months": 3, "6months": 6, year: 12 }[period];
    if (!months) return 0;
    const cutoff = new Date(now);
    const day = cutoff.getDate();
    cutoff.setDate(1);
    cutoff.setMonth(cutoff.getMonth() - months);
    const lastDay = new Date(
      cutoff.getFullYear(),
      cutoff.getMonth() + 1,
      0,
    ).getDate();
    cutoff.setDate(Math.min(day, lastDay));
    return cutoff.getTime();
  }
  $: visibleTracks = tracks.filter((track) => {
    const matchesSearch = `${track.title} ${track.artist} ${track.album}`
      .toLowerCase()
      .includes(search.toLowerCase());
    if (!matchesSearch) return false;
    if (libraryFilter === "imported" && !isImportedTrack(track)) return false;
    if (libraryFilter === "online" && isImportedTrack(track)) return false;
    return true;
  });
  $: importedVisibleTracks = tracks.filter(isImportedTrack);
  $: filteredImportedTracks = importedVisibleTracks
    .filter((track) =>
      `${track.title} ${track.artist} ${track.album}`
        .toLowerCase()
        .includes(importedSearch.trim().toLowerCase()),
    )
    .sort((left, right) =>
      String(left[importedSortBy] || "").localeCompare(
        String(right[importedSortBy] || ""),
        undefined,
        { sensitivity: "base" },
      ),
    );
  $: importedFolderTree = (() => {
    const roots = new Map();
    for (const track of filteredImportedTracks) {
      const segments = String(track.folderPath || track.album || "Imported")
        .split("/")
        .filter(Boolean);
      let siblings = roots;
      let node = null;
      let path = "";
      for (const name of segments) {
        path = path ? `${path}/${name}` : name;
        if (!siblings.has(name))
          siblings.set(name, {
            path,
            name,
            children: new Map(),
            tracks: [],
            trackCount: 0,
          });
        node = siblings.get(name);
        node.trackCount += 1;
        siblings = node.children;
      }
      if (node) node.tracks.push(track);
    }
    const groupsFor = (folderTracks) => {
      const groups = new Map();
      const groupField = importedSortBy === "folders" ? null : importedSortBy;
      for (const track of folderTracks) {
        const name = groupField
          ? track[groupField] || `Unknown ${groupField}`
          : "All songs";
        if (!groups.has(name)) groups.set(name, []);
        groups.get(name).push(track);
      }
      return [...groups]
        .sort(([left], [right]) =>
          left.localeCompare(right, undefined, { sensitivity: "base" }),
        )
        .map(([name, songs]) => ({
          name,
          tracks: songs.sort((left, right) =>
            left.title.localeCompare(right.title, undefined, {
              sensitivity: "base",
            }),
          ),
        }));
    };
    const flatten = (siblings, depth) =>
      [...siblings.values()]
        .sort((left, right) =>
          left.name.localeCompare(right.name, undefined, {
            sensitivity: "base",
          }),
        )
        .flatMap((node) => {
          const expanded = expandedImportedFolders[node.path] ?? false;
          const current = [
            {
              path: node.path,
              name: node.name,
              depth,
              expanded,
              trackCount: node.trackCount,
              groups: groupsFor(node.tracks),
            },
          ];
          return expanded
            ? [...current, ...flatten(node.children, depth + 1)]
            : current;
        });
    return flatten(roots, 0);
  })();
  $: importedBrowseGroups = (() => {
    if (importedSortBy === "folders") return [];
    const groups = new Map();
    for (const track of filteredImportedTracks) {
      const name =
        importedSortBy === "album"
          ? track.album || "Singles"
          : track.artist || "Unknown artist";
      if (!groups.has(name)) groups.set(name, []);
      groups.get(name).push(track);
    }
    return [...groups]
      .sort(([left], [right]) =>
        left.localeCompare(right, undefined, { sensitivity: "base" }),
      )
      .map(([name, groupTracks]) => ({
        name,
        path: `browse:${importedSortBy}:${name}`,
        expanded:
          expandedImportedFolders[`browse:${importedSortBy}:${name}`] ?? false,
        trackCount: groupTracks.length,
        tracks: groupTracks.sort((left, right) => {
          const albumOrder = String(left.album || "").localeCompare(
            String(right.album || ""),
            undefined,
            { sensitivity: "base" },
          );
          return importedSortBy === "artist" && albumOrder
            ? albumOrder
            : left.title.localeCompare(right.title, undefined, {
                sensitivity: "base",
              });
        }),
      }));
  })();
  $: displayedTracks = activeView === "search" ? searchResults : visibleTracks;
  $: topbarSuggestions = (() => {
    const query = search.trim().toLowerCase();
    if (!query) return [];
    const matches = topbarSearchResults.filter((track) =>
      `${track.title} ${track.artist} ${track.album}`
        .toLowerCase()
        .includes(query),
    );
    return [
      ...new Map(matches.map((track) => [track.id, track])).values(),
    ].slice(0, 10);
  })();
  $: recentlyAdded = importedTracks.slice(0, 9);
  $: activeStatsEvents = statsPeriod === "all"
    ? []
    : playEvents.filter((event) => {
        const playedAt = new Date(event.playedAt).getTime();
        return Number.isFinite(playedAt) && playedAt >= statsCutoff(statsPeriod);
      });
  $: artistStats = Object.entries(
    statsPeriod === "all"
      ? playHistory.reduce((counts, item) => {
          const artist = item.artist || "Unknown artist";
          counts[artist] = (counts[artist] || 0) + (playCounts[item.id] || 1);
          return counts;
        }, {})
      : activeStatsEvents.reduce((counts, event) => {
          const artist = event.artist || "Unknown artist";
          counts[artist] = (counts[artist] || 0) + 1;
          return counts;
        }, {}),
  )
    .sort((left, right) => right[1] - left[1])
    .slice(0, 5);
  $: songStats = statsPeriod === "all"
    ? playHistory
        .map((item) => ({ track: item, listens: playCounts[item.id] || 1 }))
        .sort((left, right) => right.listens - left.listens)
        .slice(0, 5)
    : Object.values(
        activeStatsEvents.reduce((counts, event) => {
          const current = counts[event.id] || { track: event, listens: 0 };
          current.listens += 1;
          counts[event.id] = current;
          return counts;
        }, {}),
      )
        .sort((left, right) => right.listens - left.listens)
        .slice(0, 5);
  $: libraryCollections = [
    {
      id: "liked-songs",
      name: "Liked Songs",
      kind: "Webapp playlist",
      color: "#d5e86d",
      tracks: favorites,
    },
    ...playlists
      .filter(
        (playlist) =>
          playlist.id !== "liked-songs" &&
          playlist.name !== "Liked Songs" &&
          !spotifyPlaylists.some(
            (remote) => remote.spotifyId === playlist.spotifyId,
          ),
      )
      .map((playlist) => ({
        ...playlist,
        kind: playlist.sourceSpotifyId ? "Wavelength playlist" : "Playlist",
        color: "#718979",
      })),
    ...sharedPlaylists
      .filter(
        (playlist) => !playlists.some((local) => local.id === playlist.id),
      )
      .map(resolveSharedPlaylist),
    ...(spotifySavedTracks.length
      ? [
          {
            id: "spotify-saved-songs",
            name: "Spotify Liked Songs",
            kind: "Spotify playlist",
            color: "#d5e86d",
            tracks: spotifySavedTracks,
          },
        ]
      : []),
    ...spotifyPlaylists.map((remote) => {
      const local = playlists.find(
        (playlist) => playlist.spotifyId === remote.spotifyId,
      );
      return {
        ...remote,
        tracks: local?.tracks || remote.tracks,
        copiedToWavelength: playlists.some(
          (playlist) => playlist.sourceSpotifyId === remote.spotifyId,
        ),
      };
    }),
    ...[
      ...new Map(
        onlineTracks
          .filter((track) => track.album)
          .map((track) => [`${track.artist || ""}\u0000${track.album}`, track]),
      ).values(),
    ].map((track) => ({
      id: `album-${encodeURIComponent(`${track.artist || ""}::${track.album}`)}`,
      name: track.album,
      kind: "Album",
      color: track.color,
      tracks: onlineTracks.filter(
        (candidate) =>
          candidate.album === track.album && candidate.artist === track.artist,
      ),
    })),
  ];
  $: savedPlaylistCollections = libraryCollections.filter((collection) =>
    [
      "Playlist",
      "Wavelength playlist",
      "Spotify playlist",
      "Webapp playlist",
      "Shared playlist",
    ].includes(collection.kind),
  );
  $: selectedCollection =
    libraryCollections.find(
      (collection) => collection.id === selectedCollectionId,
    ) || null;
  $: isFavorite = (track) =>
    favorites.some((item) => item.id === track.id) ||
    spotifySavedTracks.some((item) => item.id === track.id);
  $: pageTitle =
    activeView === "search"
      ? "Search"
      : activeView === "playlists"
        ? "Playlists"
        : activeView === "songs"
          ? "Local library"
          : activeView === "collection"
            ? selectedCollection?.name || "Collection"
            : activeView === "queue"
              ? "Queue"
              : activeView === "settings"
                ? "Settings"
                : activeView === "customization"
                  ? "Customization"
                  : activeView === "equalizer"
                    ? "Equalizer"
                    : activeView === "logs"
                      ? "Logs"
                      : activeView === "history"
                        ? "Listening history"
                        : activeView === "now-playing"
                          ? "Now playing"
                          : "Home";

  function getTrackArt(track) {
    if (!track) return "";
    return (
      track.art ||
      track.cover ||
      track.coverArt ||
      track.image ||
      track.imageUrl ||
      track.thumbnail ||
      track.albumArt ||
      track.artwork ||
      track.poster ||
      track.photo ||
      ""
    );
  }

  async function loadNowPlayingMetadata(track) {
    const requestId = ++nowPlayingMetadataRequest;
    nowPlayingMetadata = {
      bpm: track?.bpm ?? track?.tempo ?? null,
      scale: track?.scale || "",
      releaseDate: track?.releaseDate || track?.release_date || "",
    };
    if (!track) return;

    const params = new URLSearchParams({
      title: String(track.title || ""),
      artist: String(track.artist || ""),
    });
    const uri = spotifyTrackUri(track);
    if (uri) params.set("spotify_id", uri.slice("spotify:track:".length));

    try {
      const metadata = await api(`/audio/track-metadata?${params}`);
      if (requestId !== nowPlayingMetadataRequest) return;
      nowPlayingMetadata = {
        bpm: metadata.bpm ?? nowPlayingMetadata.bpm ?? null,
        scale: metadata.scale || nowPlayingMetadata.scale || "",
        releaseDate:
          metadata.releaseDate || nowPlayingMetadata.releaseDate || "",
      };
    } catch {}
  }

  function getCollectionArt(collection) {
    if (isLikedSongsCollection(collection)) return "";
    const track = collection?.tracks?.find((item) => getTrackArt(item));
    return getTrackArt(track);
  }

  function isLikedSongsCollection(collection) {
    return (
      collection?.id === "liked-songs" || collection?.name === "Liked Songs"
    );
  }

  function getTrackSource(track) {
    if (
      track?.file ||
      String(track?.id || "").startsWith("local-") ||
      String(track?.source || "").toLowerCase() === "local"
    )
      return "Local";
    if (
      String(track?.source || "").toLowerCase() === "youtube" ||
      String(track?.id || "").startsWith("yt-")
    )
      return "YouTube";
    if (
      String(track?.source || "").toLowerCase() === "spotify" ||
      track?.spotify_url ||
      track?.spotify_uri ||
      track?.spotifyUri ||
      track?.external_urls?.spotify
    )
      return "Spotify";
    if (track?.videoId) return "YouTube";
    return "Spotify";
  }

  function getQueueAudioId(track) {
    const source = getTrackSource(track);
    const spotifyId =
      source === "Spotify"
        ? spotifyTrackUri(track)?.match(/^spotify:track:([A-Za-z0-9]+)$/)?.[1]
        : "";
    const id = String(
      source === "YouTube"
        ? track?.videoId || track?.id || ""
        : spotifyId || track?.id || track?.videoId || "",
    );
    return source === "YouTube" ? (id.startsWith("yt-") ? id : `yt-${id}`) : id;
  }

  function isQueueDownloadable(track) {
    if (
      !track ||
      track.file ||
      getTrackSource(track) === "Local" ||
      String(track.id || "").startsWith("demo-")
    )
      return false;
    const source = getTrackSource(track);
    if (source === "YouTube") return true;
    return (
      source === "Spotify" &&
      (String(track.source || "").toLowerCase() === "spotify" ||
        Boolean(spotifyTrackUri(track)) ||
        /^[A-Za-z0-9]{22}$/.test(String(track.id || "")))
    );
  }

  function toPartyTrack(track) {
    const source = getTrackSource(track);
    const spotifyId =
      spotifyTrackUri(track)?.match(/^spotify:track:([A-Za-z0-9]+)$/)?.[1] ||
      (source === "Local" && /^[A-Za-z0-9]{22}$/.test(String(track?.id || ""))
        ? String(track.id)
        : "");
    const videoId =
      source === "YouTube"
        ? String(track?.videoId || track?.id || "").replace(/^yt-/, "")
        : "";
    const localOnly = source === "Local" && !spotifyId && !videoId;
    const originalId = String(track?.id || "track");
    const id = localOnly
      ? originalId.startsWith("local-")
        ? originalId
        : `local-${originalId}`
      : videoId
        ? `yt-${videoId}`
        : spotifyId || getQueueAudioId(track);
    const artwork = getTrackArt(track);
    return {
      id,
      source: source.toLowerCase(),
      videoId,
      audioId: track?.serverAudioId || "",
      title: String(track?.title || "Untitled track"),
      artist: String(track?.artist || "Unknown artist"),
      thumbnail:
        typeof artwork === "string" && /^https?:\/\//i.test(artwork)
          ? artwork
          : "",
      duration: Number.isFinite(Number(track?.duration))
        ? Number(track.duration)
        : 0,
      addedBy: partyDisplayName,
      status: "pending",
    };
  }

  function getQueueRowState(track) {
    const nowPlaying = Boolean(track && playing?.id === track.id);
    return { nowPlaying };
  }

  function formatLogTime(timestamp) {
    return new Date(timestamp).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }

  function savePlaylists() {
    localStorage.setItem("wavelength-playlists", JSON.stringify(playlists));
    schedulePartyStatePublish();
  }

  function playlistReferencesForParty() {
    return playlists.map((playlist) => ({
      id: String(playlist.id || ""),
      name: String(playlist.name || "Playlist"),
      tracks: (playlist.tracks || [])
        .map((track) => ({
          id: getQueueAudioId(track),
          platform: getTrackSource(track).toLowerCase(),
        }))
        .filter((reference) => reference.id),
    }));
  }

  function resolveSharedPlaylist(playlist) {
    const resolvedTracks = (playlist.tracks || []).map((reference) => {
      const id = String(reference.id || "");
      const platform = String(reference.platform || "").toLowerCase();
      const match = [...tracks, ...favorites, ...playHistory].find(
        (track) =>
          getQueueAudioId(track) === id &&
          getTrackSource(track).toLowerCase() === platform,
      );
      if (match) return match;
      const youtube = platform === "youtube";
      return {
        id,
        videoId: youtube ? id.replace(/^yt-/, "") : id,
        title: id,
        artist: platform || "Unknown platform",
        album: playlist.name || "",
        source: platform,
        mediaType: "track",
      };
    });
    return {
      ...playlist,
      kind: "Shared playlist",
      color: "#718979",
      tracks: resolvedTracks,
    };
  }

  function formatTime(value) {
    if (typeof value === "string" && value.includes(":")) return value;
    const seconds = Math.max(0, Number(value) || 0);
    return `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;
  }

  function persistEqualizerSettings() {
    localStorage.setItem(
      "wavelength-equalizer-gains",
      JSON.stringify(equalizerGains),
    );
    localStorage.setItem(
      "wavelength-equalizer-frequencies",
      JSON.stringify(equalizerFrequencies),
    );
    localStorage.setItem(
      "wavelength-equalizer-qs",
      JSON.stringify(equalizerQs),
    );
    localStorage.setItem(
      "wavelength-equalizer-types",
      JSON.stringify(equalizerTypes),
    );
    localStorage.setItem(
      "wavelength-equalizer-preset",
      activeEqualizerPresetId,
    );
  }

  function applyEqualizerGains({ immediate = false } = {}) {
    equalizerFilters.forEach((filter, index) => {
      const currentTime = equalizerContext.currentTime;
      filter.type = equalizerTypes[index] || "peaking";
      [
        [filter.frequency, equalizerFrequencies[index] || equalizerBands[index]],
        [filter.Q, equalizerQs[index] || 1.1],
        [filter.gain, equalizerGains[index] || 0],
      ].forEach(([parameter, value]) => {
        if (immediate) {
          parameter.cancelScheduledValues(currentTime);
          parameter.setValueAtTime(value, currentTime);
        } else {
          parameter.setTargetAtTime(value, currentTime, 0.015);
        }
      });
    });
    drawEqualizerSpectrum();
  }

  function selectEqualizerPreset(event) {
    const presetId = event.detail;
    if (presetId === "custom") return;
    const preset = [...builtInEqualizerPresets, ...customEqualizerPresets].find(
      (item) => item.id === presetId,
    );
    if (!preset) return;
    activeEqualizerPresetId = preset.id;
    equalizerGains = [...preset.gains];
    equalizerFrequencies =
      Array.isArray(preset.frequencies) &&
      preset.frequencies.length === equalizerBands.length
        ? preset.frequencies.map((frequency) =>
            Math.max(20, Math.min(20000, Number(frequency) || 20)),
          )
        : [...equalizerBands];
    equalizerQs =
      Array.isArray(preset.qs) && preset.qs.length === equalizerBands.length
        ? preset.qs.map((value) => Math.max(0.3, Math.min(10, Number(value) || 1.1)))
        : equalizerBands.map(() => 1.1);
    equalizerTypes =
      Array.isArray(preset.types) && preset.types.length === equalizerBands.length
        ? preset.types.map((type) =>
            equalizerTypeOptions.some((option) => option.value === type)
              ? type
              : "peaking",
          )
        : equalizerBands.map(() => "peaking");
    persistEqualizerSettings();
    applyEqualizerGains({ immediate: true });
  }

  function setEqualizerGain(index, event) {
    equalizerGains = equalizerGains.map((gain, bandIndex) =>
      bandIndex === index ? Number(event.currentTarget.value) : gain,
    );
    activeEqualizerPresetId = "custom";
    persistEqualizerSettings();
    applyEqualizerGains();
  }

  function setEqualizerFrequency(index, event) {
    const position = Number(event.currentTarget.value) / 1000;
    const frequency = Math.round(20 * 1000 ** position);
    equalizerFrequencies = equalizerFrequencies.map((value, bandIndex) =>
      bandIndex === index ? frequency : value,
    );
    activeEqualizerPresetId = "custom";
    persistEqualizerSettings();
    applyEqualizerGains();
  }

  function setEqualizerQ(index, event) {
    equalizerQs = equalizerQs.map((value, bandIndex) =>
      bandIndex === index ? Number(event.currentTarget.value) : value,
    );
    activeEqualizerPresetId = "custom";
    persistEqualizerSettings();
    applyEqualizerGains();
  }

  function setEqualizerType(index, event) {
    const type = event.detail;
    if (!equalizerTypeOptions.some((option) => option.value === type)) return;
    equalizerTypes = equalizerTypes.map((value, bandIndex) =>
      bandIndex === index ? type : value,
    );
    activeEqualizerPresetId = "custom";
    persistEqualizerSettings();
    applyEqualizerGains();
  }

  function resetEqualizerBand(index) {
    equalizerFrequencies = equalizerFrequencies.map((frequency, bandIndex) =>
      bandIndex === index ? equalizerBands[index] : frequency,
    );
    equalizerGains = equalizerGains.map((gain, bandIndex) =>
      bandIndex === index ? 0 : gain,
    );
    equalizerQs = equalizerQs.map((q, bandIndex) =>
      bandIndex === index ? 1.1 : q,
    );
    equalizerTypes = equalizerTypes.map((type, bandIndex) =>
      bandIndex === index ? "peaking" : type,
    );
    activeEqualizerPresetId = "custom";
    persistEqualizerSettings();
    applyEqualizerGains({ immediate: true });
  }

  function resetEqualizer() {
    equalizerFrequencies = [...equalizerBands];
    equalizerGains = [...builtInEqualizerPresets[0].gains];
    equalizerQs = equalizerBands.map(() => 1.1);
    equalizerTypes = equalizerBands.map(() => "peaking");
    activeEqualizerPresetId = "flat";
    persistEqualizerSettings();
    applyEqualizerGains({ immediate: true });
  }

  function equalizerPlotBounds(bounds) {
    const left = 40;
    const top = 18;
    const bottom = bounds.height - 30;
    return {
      left,
      top,
      bottom,
      width: Math.max(1, bounds.width - left - 14),
      height: Math.max(1, bottom - top),
    };
  }

  function equalizerFrequencyToX(frequency, plot) {
    const ratio = Math.log(Math.max(20, frequency) / 20) / Math.log(1000);
    return plot.left + Math.max(0, Math.min(1, ratio)) * plot.width;
  }

  function equalizerGainToY(gain, plot) {
    return plot.top + ((18 - gain) / 36) * plot.height;
  }

  function updateEqualizerBandFromPointer(event, index) {
    const bounds = equalizerSpectrumCanvas?.getBoundingClientRect();
    if (!bounds) return;
    const plot = equalizerPlotBounds(bounds);
    const x = Math.max(0, Math.min(1, (event.clientX - bounds.left - plot.left) / plot.width));
    const y = Math.max(0, Math.min(1, (event.clientY - bounds.top - plot.top) / plot.height));
    const frequency = Math.round(20 * 1000 ** x);
    const gain = Math.round(Math.max(-18, Math.min(18, (0.5 - y) * 36)) * 10) / 10;
    equalizerFrequencies = equalizerFrequencies.map((value, bandIndex) =>
      bandIndex === index ? frequency : value,
    );
    equalizerGains = equalizerGains.map((value, bandIndex) =>
      bandIndex === index ? gain : value,
    );
    selectedEqualizerBand = index;
    activeEqualizerPresetId = "custom";
    applyEqualizerGains();
  }

  function startEqualizerBandDrag(event) {
    if (equalizerUnavailable || !equalizerSpectrumCanvas) return;
    event.preventDefault();
    const bounds = equalizerSpectrumCanvas.getBoundingClientRect();
    const plot = equalizerPlotBounds(bounds);
    const x = event.clientX - bounds.left;
    const y = event.clientY - bounds.top;
    let nearestIndex = 0;
    let nearestDistance = Infinity;
    equalizerFrequencies.forEach((frequency, index) => {
      const dx = equalizerFrequencyToX(frequency, plot) - x;
      const dy = equalizerGainToY(equalizerGains[index], plot) - y;
      const distance = dx * dx + dy * dy;
      if (distance < nearestDistance) {
        nearestDistance = distance;
        nearestIndex = index;
      }
    });
    equalizerDragBand = nearestIndex;
    selectedEqualizerBand = nearestIndex;
    equalizerSpectrumCanvas.setPointerCapture(event.pointerId);
    updateEqualizerBandFromPointer(event, nearestIndex);
  }

  function moveEqualizerBand(event) {
    if (equalizerDragBand >= 0)
      updateEqualizerBandFromPointer(event, equalizerDragBand);
  }

  function endEqualizerBandDrag() {
    if (equalizerDragBand < 0) return;
    equalizerDragBand = -1;
    persistEqualizerSettings();
  }

  function saveEqualizerPreset() {
    const name = equalizerPresetName.trim();
    if (!name) {
      notice = "Enter a name for this equalizer preset.";
      return;
    }
    const preset = {
      id: `custom-${Date.now()}`,
      name,
      gains: [...equalizerGains],
      frequencies: [...equalizerFrequencies],
      qs: [...equalizerQs],
      types: [...equalizerTypes],
    };
    customEqualizerPresets = [...customEqualizerPresets, preset];
    activeEqualizerPresetId = preset.id;
    equalizerPresetName = "";
    localStorage.setItem(
      "wavelength-equalizer-presets",
      JSON.stringify(customEqualizerPresets),
    );
    persistEqualizerSettings();
    notice = `${name} preset saved.`;
  }

  function setupEqualizer() {
    if (!audioElement || equalizerSource) return;
    const AudioContextClass = window.AudioContext || window.webkitAudioContext;
    if (!AudioContextClass) {
      equalizerUnavailable = true;
      return;
    }
    try {
      audioElement.crossOrigin = "anonymous";
      equalizerContext = new AudioContextClass();
      equalizerSource = equalizerContext.createMediaElementSource(audioElement);
      let previousNode = equalizerSource;
      equalizerFilters = equalizerBands.map((frequency, index) => {
        const filter = equalizerContext.createBiquadFilter();
        filter.type = "peaking";
        filter.frequency.value = equalizerFrequencies[index] || frequency;
        filter.Q.value = equalizerQs[index] || 1.1;
        filter.gain.value = equalizerGains[index] || 0;
        previousNode.connect(filter);
        previousNode = filter;
        return filter;
      });
      equalizerAnalyser = equalizerContext.createAnalyser();
      equalizerAnalyser.fftSize = 2048;
      equalizerAnalyser.smoothingTimeConstant = 0.82;
      equalizerSpectrumData = new Uint8Array(
        equalizerAnalyser.frequencyBinCount,
      );
      previousNode.connect(equalizerAnalyser);
      equalizerAnalyser.connect(equalizerContext.destination);
    } catch {
      equalizerUnavailable = true;
      equalizerContext?.close().catch(() => {});
      equalizerContext = null;
      equalizerSource = null;
      equalizerAnalyser = null;
      equalizerSpectrumData = null;
      equalizerFilters = [];
    }
  }

  function drawEqualizerSpectrum() {
    if (!equalizerSpectrumCanvas) return;
    const bounds = equalizerSpectrumCanvas.getBoundingClientRect();
    if (!bounds.width || !bounds.height) return;
    const pixelRatio = Math.min(window.devicePixelRatio || 1, 2);
    const pixelWidth = Math.round(bounds.width * pixelRatio);
    const pixelHeight = Math.round(bounds.height * pixelRatio);
    if (
      equalizerSpectrumCanvas.width !== pixelWidth ||
      equalizerSpectrumCanvas.height !== pixelHeight
    ) {
      equalizerSpectrumCanvas.width = pixelWidth;
      equalizerSpectrumCanvas.height = pixelHeight;
    }
    const canvasContext = equalizerSpectrumCanvas.getContext("2d");
    if (!canvasContext) return;
    canvasContext.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
    canvasContext.clearRect(0, 0, bounds.width, bounds.height);
    const palette = getComputedStyle(document.documentElement);
    const lime = palette.getPropertyValue("--lime").trim() || "#d5e86d";
    const line = palette.getPropertyValue("--line").trim() || "#2a302b";
    const muted = palette.getPropertyValue("--muted").trim() || "#92938d";
    const panel = palette.getPropertyValue("--panel").trim() || "#151714";
    const plot = equalizerPlotBounds(bounds);
    const gainToY = (gain) => plot.top + ((18 - gain) / 36) * plot.height;
    const frequencyTicks = [20, 30, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000];
    const majorTicks = new Set([20, 100, 1000, 10000, 20000]);

    canvasContext.font = "10px 'IBM Plex Mono', monospace";
    canvasContext.textBaseline = "middle";
    frequencyTicks.forEach((frequency) => {
      const x = equalizerFrequencyToX(frequency, plot);
      canvasContext.globalAlpha = majorTicks.has(frequency) ? 0.5 : 0.22;
      canvasContext.strokeStyle = line;
      canvasContext.lineWidth = 1;
      canvasContext.beginPath();
      canvasContext.moveTo(x, plot.top);
      canvasContext.lineTo(x, plot.bottom);
      canvasContext.stroke();
    });
    [-18, -12, -6, 0, 6, 12, 18].forEach((gain) => {
      const y = gainToY(gain);
      canvasContext.globalAlpha = gain === 0 ? 0.55 : 0.28;
      canvasContext.strokeStyle = line;
      canvasContext.beginPath();
      canvasContext.moveTo(plot.left, y);
      canvasContext.lineTo(plot.left + plot.width, y);
      canvasContext.stroke();
      canvasContext.globalAlpha = 0.9;
      canvasContext.fillStyle = muted;
      canvasContext.textAlign = "left";
      canvasContext.fillText(gain > 0 ? `+${gain}` : `${gain}`, 5, y);
    });

    const sampleCount = Math.max(128, Math.floor(plot.width));
    const responseFrequencies = new Float32Array(sampleCount);
    const responseDb = new Float32Array(sampleCount);
    for (let index = 0; index < sampleCount; index += 1) {
      responseFrequencies[index] =
        20 * 1000 ** (index / Math.max(1, sampleCount - 1));
    }
    if (equalizerFilters.length) {
      const magnitudes = new Float32Array(sampleCount);
      const phases = new Float32Array(sampleCount);
      equalizerFilters.forEach((filter) => {
        filter.getFrequencyResponse(responseFrequencies, magnitudes, phases);
        for (let index = 0; index < sampleCount; index += 1)
          responseDb[index] += 20 * Math.log10(Math.max(magnitudes[index], 0.0001));
      });
    } else {
      for (let index = 0; index < sampleCount; index += 1) {
        equalizerFrequencies.forEach((frequency, bandIndex) => {
          const octaves = Math.log2(responseFrequencies[index] / frequency);
          const q = equalizerQs[bandIndex] || 1.1;
          responseDb[index] +=
            (equalizerGains[bandIndex] || 0) *
            Math.exp(-0.5 * (octaves * q * 1.8) ** 2);
        });
      }
    }

    if (equalizerAnalyser && equalizerSpectrumData) {
      equalizerAnalyser.getByteFrequencyData(equalizerSpectrumData);
      const barCount = Math.floor(plot.width / 3);
      const barWidth = plot.width / barCount;
      const nyquist = (equalizerContext?.sampleRate || 48000) / 2;
      for (let index = 0; index < barCount; index += 1) {
        const ratio = index / Math.max(1, barCount - 1);
        const frequency = 20 * 1000 ** ratio;
        const bin = Math.min(
          equalizerSpectrumData.length - 1,
          Math.round((frequency / nyquist) * equalizerSpectrumData.length),
        );
        const magnitude = equalizerSpectrumData[bin] / 255;
        const barHeight = magnitude * plot.height * 0.38;
        canvasContext.globalAlpha = 0.08 + magnitude * 0.24;
        canvasContext.fillStyle = lime;
        canvasContext.fillRect(
          plot.left + index * barWidth,
          plot.bottom - barHeight,
          Math.max(1, barWidth - 1),
          barHeight,
        );
      }
    }

    const responseY = (gain) => gainToY(Math.max(-18, Math.min(18, gain)));
    const baseline = gainToY(0);
    canvasContext.beginPath();
    canvasContext.moveTo(plot.left, baseline);
    for (let index = 0; index < sampleCount; index += 1) {
      const x = plot.left + (index / (sampleCount - 1)) * plot.width;
      canvasContext.lineTo(x, responseY(responseDb[index]));
    }
    canvasContext.lineTo(plot.left + plot.width, baseline);
    canvasContext.closePath();
    canvasContext.globalAlpha = 0.12;
    canvasContext.fillStyle = lime;
    canvasContext.fill();

    canvasContext.beginPath();
    for (let index = 0; index < sampleCount; index += 1) {
      const x = plot.left + (index / (sampleCount - 1)) * plot.width;
      const y = responseY(responseDb[index]);
      if (index === 0) canvasContext.moveTo(x, y);
      else canvasContext.lineTo(x, y);
    }
    canvasContext.globalAlpha = 0.95;
    canvasContext.strokeStyle = lime;
    canvasContext.lineWidth = 2;
    canvasContext.stroke();

    const frequencyLabels =
      bounds.width < 460 ? [20, 100, 1000, 20000] : [20, 100, 1000, 10000, 20000];
    frequencyLabels.forEach((frequency, index, ticks) => {
      const x = equalizerFrequencyToX(frequency, plot);
      canvasContext.globalAlpha = 0.9;
      canvasContext.fillStyle = muted;
      canvasContext.textAlign = index === 0 ? "left" : index === ticks.length - 1 ? "right" : "center";
      canvasContext.fillText(frequency >= 1000 ? `${frequency / 1000}k` : `${frequency}`, x, bounds.height - 10);
    });

    equalizerFrequencies.forEach((frequency, index) => {
      const x = equalizerFrequencyToX(frequency, plot);
      const y = gainToY(equalizerGains[index] || 0);
      const color = equalizerNodeColors[index % equalizerNodeColors.length];
      canvasContext.beginPath();
      canvasContext.arc(x, y, index === selectedEqualizerBand ? 8 : 6, 0, Math.PI * 2);
      canvasContext.fillStyle = index === selectedEqualizerBand ? color : panel;
      canvasContext.fill();
      canvasContext.lineWidth = 2;
      canvasContext.strokeStyle = color;
      canvasContext.stroke();
    });
    canvasContext.globalAlpha = 1;
  }

  function animateEqualizerSpectrum() {
    drawEqualizerSpectrum();
    if (activeView === "equalizer" && isPlaying)
      equalizerSpectrumFrame = requestAnimationFrame(animateEqualizerSpectrum);
    else equalizerSpectrumFrame = 0;
  }

  function stopEqualizerSpectrum() {
    if (equalizerSpectrumFrame) cancelAnimationFrame(equalizerSpectrumFrame);
    equalizerSpectrumFrame = 0;
  }

  $: if (activeView === "equalizer" && equalizerSpectrumCanvas) {
    stopEqualizerSpectrum();
    drawEqualizerSpectrum();
    if (isPlaying)
      equalizerSpectrumFrame = requestAnimationFrame(animateEqualizerSpectrum);
  } else stopEqualizerSpectrum();

  async function resumeEqualizer() {
    if (!equalizerContext) setupEqualizer();
    if (equalizerContext?.state === "suspended")
      await equalizerContext.resume();
  }

  function contrastColor(color) {
    const hex = String(color || "#718979").replace("#", "");
    const value =
      hex.length === 3
        ? hex
            .split("")
            .map((part) => part + part)
            .join("")
        : hex;
    const [red, green, blue] = [0, 2, 4].map(
      (index) => parseInt(value.slice(index, index + 2), 16) || 0,
    );
    return (red * 299 + green * 587 + blue * 114) / 1000 > 150
      ? "#101311"
      : "#f5f7ec";
  }
  function themeSafeAccent(color) {
    const hex = String(color || "#d7c7a4").replace("#", "");
    return /^[0-9a-f]{6}$/i.test(hex) ? `#${hex.toLowerCase()}` : "#d7c7a4";
  }

  function hexLuminance(color) {
    const hex = String(color || "").replace("#", "");
    if (!/^[0-9a-f]{6}$/i.test(hex)) return 0;
    const channels = [0, 2, 4].map(
      (index) => parseInt(hex.slice(index, index + 2), 16) / 255,
    );
    const linear = channels.map((value) =>
      value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4,
    );
    return linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722;
  }

  function customAccentContrast(color) {
    const background = resolvedTheme() === "light" ? "#ece9df" : "#171816";
    const colorLuminance = hexLuminance(color);
    const backgroundLuminance = hexLuminance(background);
    return (
      (Math.max(colorLuminance, backgroundLuminance) + 0.05) /
      (Math.min(colorLuminance, backgroundLuminance) + 0.05)
    );
  }

  function applyAppearance() {
    const root = document.documentElement;
    root.dataset.theme = resolvedTheme();
    const safeAccent = themeSafeAccent(appAccent);
    root.dataset.themePreference = appTheme;
    root.style.setProperty("--lime", safeAccent);
    root.style.setProperty(
      "--accent-rgb",
      safeAccent
        .match(/[a-f\d]{2}/gi)
        ?.map((channel) => parseInt(channel, 16))
        .join(", ") || "215, 199, 164",
    );
    if (appBackgroundColor) {
      const channels = appBackgroundColor
        .slice(1)
        .match(/.{2}/g)
        .map((channel) => parseInt(channel, 16) / 255)
        .map((channel) =>
          channel <= 0.04045
            ? channel / 12.92
            : ((channel + 0.055) / 1.055) ** 2.4,
        );
      const luminance =
        channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
      const lightBackground = luminance > 0.179;
      const foreground = lightBackground ? "#1c2019" : "#edf0e8";
      const panelTint = lightBackground ? "#000000" : "#ffffff";
      root.style.setProperty("--surface-page", appBackgroundColor);
      root.style.setProperty(
        "--panel",
        `color-mix(in srgb, ${appBackgroundColor} 92%, ${panelTint})`,
      );
      root.style.setProperty(
        "--panel-hi",
        `color-mix(in srgb, ${appBackgroundColor} 84%, ${panelTint})`,
      );
      root.style.setProperty("--ink", foreground);
      root.style.setProperty(
        "--muted",
        `color-mix(in srgb, ${foreground} 82%, ${appBackgroundColor})`,
      );
      root.style.setProperty(
        "--quiet",
        `color-mix(in srgb, ${foreground} 62%, ${appBackgroundColor})`,
      );
      root.style.setProperty(
        "--line",
        `color-mix(in srgb, ${foreground} 16%, transparent)`,
      );
    } else {
      [
        "--surface-page",
        "--panel",
        "--panel-hi",
        "--ink",
        "--muted",
        "--quiet",
        "--line",
      ].forEach((property) => root.style.removeProperty(property));
    }
    localStorage.setItem("wavelength-theme", appTheme);
    localStorage.setItem("wavelength-accent", appAccent);
    if (appBackgroundColor)
      localStorage.setItem("wavelength-background-color", appBackgroundColor);
    else localStorage.removeItem("wavelength-background-color");
    localStorage.setItem("wavelength-resolved-accent", safeAccent);
  }

  function applyUiScale() {
    document.documentElement.style.setProperty("--app-scale", String(uiScale));
    localStorage.setItem(uiScaleStorageKey, String(uiScale));
  }

  function handleUiScaleShortcut(event) {
    if (!event.ctrlKey || event.altKey || event.metaKey) return;
    const direction =
      event.key === "+" ||
      event.key === "=" ||
      event.code === "NumpadAdd"
        ? 1
        : event.key === "-" ||
            event.key === "_" ||
            event.code === "NumpadSubtract"
          ? -1
          : 0;
    if (!direction) return;
    event.preventDefault();
    uiScale = Math.min(
      uiScaleMaximum,
      Math.max(
        uiScaleMinimum,
        Math.round((uiScale + direction * 0.1) * 100) / 100,
      ),
    );
    applyUiScale();
  }

  function setTheme(theme) {
    appTheme = theme;
    applyAppearance();
  }

  function setAccent(accent) {
    appAccent = accent;
    applyAppearance();
  }

  function setBackgroundColor(color) {
    appBackgroundColor = color;
    applyAppearance();
  }

  function openAccentPicker() {
    customAccentInput = appAccent.toUpperCase();
    customAccentError = "";
    updateContentModalBounds();
    showAccentPicker = true;
  }

  function applyCustomAccent() {
    const match = customAccentInput.trim().match(/^#?([a-f\d]{6})$/i);
    if (!match) {
      customAccentError = "Enter a six-digit hex color, such as #D7C7A4.";
      return;
    }
    const accent = `#${match[1].toLowerCase()}`;
    const contrast = customAccentContrast(accent);
    if (contrast < 3) {
      customAccentError = `${accent.toUpperCase()} has low contrast in ${resolvedTheme()} mode. Choose a color with at least 3:1 contrast.`;
      return;
    }
    setAccent(accent);
    customAccentError = "";
    showAccentPicker = false;
  }

  function clearPlayHistory() {
    playHistory = [];
    localStorage.removeItem("wavelength-play-history");
  }

  function viewFromPath(pathname) {
    if (pathname === "/search") return "search";
    if (pathname === "/playlists") return "playlists";
    if (pathname === "/logs") return "logs";
    if (pathname === "/history") return "history";
    if (pathname === "/equalizer") return "equalizer";
    if (pathname === "/customization") return "customization";
    if (pathname.startsWith("/collection/")) return "collection";
    if (pathname === "/now-playing") return "now-playing";
    if (pathname === "/queue") return "queue";
    if (pathname === "/settings") return "settings";
    return (
      views.find((view) => view.path === pathname)?.id ||
      (pathname === "/library" ? "songs" : "overview")
    );
  }

  function closeContentModals() {
    showFolderManager = false;
    qobuzCredentialsOpen = false;
    showAppUpdatePrompt = false;
    pendingDesktopUpdate = null;
    showCreatePlaylist = false;
    showAccentPicker = false;
  }

  function navigate(path) {
    if (path === "/search" && !downloadFolderReady) {
      notice = "Set a download folder before searching or downloading.";
      return;
    }
    if (path === "/now-playing" && window.innerWidth > 735) return;
    if (path === "/queue" && window.innerWidth > 735) {
      notice = "Queue is available from the desktop sidebar.";
      return;
    }
    closeContentModals();
    if (path !== "/search") playlistAddTarget = null;
    window.history.pushState({}, "", path);
    if (appNavigationStack.at(-1) !== path) {
      appNavigationStack = [...appNavigationStack, path];
      forwardNavigationStack = [];
      persistNavigationHistory();
    }
    activeView = viewFromPath(path);
    selectedCollectionId =
      activeView === "collection"
        ? decodeURIComponent(path.slice("/collection/".length))
        : "";
    if (
      activeView === "equalizer" ||
      activeView === "customization" ||
      activeView === "collection"
    ) {
      queueVisible = false;
      partyDetailsVisible = false;
    }
    if (activeView === "logs") startLogsAutoRefresh();
    else stopLogsAutoRefresh();
    if (partyDetailsVisible) {
      partyDetailsVisible = false;
      queueVisible = false;
    }
    if (mobileViewport) queueVisible = false;
    if (path === "/library") libraryFilter = "imported";
    lyricsFullscreen = false;
    mobileNav = false;
  }

  function goBack() {
    if (
      activeView === "collection" &&
      appNavigationStack.at(-2) !== "/playlists"
    ) {
      window.history.replaceState({}, "", "/playlists");
      appNavigationStack = ["/playlists"];
      forwardNavigationStack = [];
      activeView = "playlists";
      selectedCollectionId = "";
      persistNavigationHistory();
      return;
    }
    if (appNavigationStack.length < 2) return;
    forwardNavigationStack = [
      ...forwardNavigationStack,
      appNavigationStack.at(-1),
    ];
    appNavigationStack = appNavigationStack.slice(0, -1);
    persistNavigationHistory();
    window.history.back();
  }

  function goForward() {
    if (!forwardNavigationStack.length) return;
    const nextPath = forwardNavigationStack.at(-1);
    forwardNavigationStack = forwardNavigationStack.slice(0, -1);
    appNavigationStack = [...appNavigationStack, nextPath];
    persistNavigationHistory();
    window.history.forward();
  }

  function persistNavigationHistory() {
    sessionStorage.setItem(
      "wavelength-navigation-stack",
      JSON.stringify(appNavigationStack),
    );
    sessionStorage.setItem(
      "wavelength-navigation-forward-stack",
      JSON.stringify(forwardNavigationStack),
    );
  }

  onMount(() => {
    let mounted = true;
    let unlistenSpotifyOAuth;
    if (isTauri()) {
      void getVersion()
        .then((version) => {
          if (mounted) appVersion = version;
        })
        .catch((error) => {
          console.error("Could not read the app version.", error);
          if (mounted) appVersion = "Unavailable";
        });
    }
    systemThemeMedia = window.matchMedia("(prefers-color-scheme: dark)");
    applyAppearance();
    applyUiScale();
    loadDownloadDirectory();
    void restoreQobuzCredentials();
    if (isTauri()) {
      void listen("spotify-oauth-result", async ({ payload }) => {
        if (payload?.status === "error") {
          pendingSpotifyOAuth = null;
          notice = "Spotify sign-in was not completed.";
          return;
        }
        try {
          let profile;
          if (
            payload?.status === "authorized" &&
            typeof payload.code === "string" &&
            pendingSpotifyOAuth
          ) {
            const flow = pendingSpotifyOAuth;
            pendingSpotifyOAuth = null;
            profile = await completeSpotifyOAuth(payload.code, flow);
          } else if (
            payload?.status === "connected" &&
            typeof payload.ticket === "string"
          ) {
            await api("/spotify/desktop-exchange", {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ ticket: payload.ticket }),
            });
            profile = await fetchSpotifyData(true);
          } else {
            pendingSpotifyOAuth = null;
            throw new Error("Spotify sign-in returned an invalid response.");
          }
          if (profile && pendingInviteCode)
            joinPartyAs(profile.display_name || profile.id, pendingInviteCode);
        } catch (error) {
          notice = error?.message || "Spotify sign-in could not be completed.";
        }
      })
        .then((unlisten) => {
          if (mounted) unlistenSpotifyOAuth = unlisten;
          else unlisten();
        })
        .catch((error) => {
          notice = error?.message || "Spotify sign-in could not be prepared.";
        });
      appUpdateStartupTimer = window.setTimeout(
        () => checkForAppUpdates(true),
        10_000,
      );
      appUpdatePollingTimer = window.setInterval(
        () => checkForAppUpdates(true),
        6 * 60 * 60 * 1000,
      );
    }
    systemThemeMedia.addEventListener?.("change", applyAppearance);
    const roomServerReady = loadRoomServerUrl();
    if (audioElement) {
      audioElement.volume = volume;
      setupEqualizer();
    }
    activeView = viewFromPath(window.location.pathname);
    selectedCollectionId =
      activeView === "collection"
        ? decodeURIComponent(
            window.location.pathname.slice("/collection/".length),
          )
        : "";
    if (
      activeView === "equalizer" ||
      activeView === "customization" ||
      activeView === "collection" ||
      activeView === "playlists"
    ) {
      queueVisible = false;
      partyDetailsVisible = false;
    }
    loadLyrics(playing);
    if (activeView === "now-playing" && window.innerWidth > 735) {
      activeView = "overview";
      window.history.replaceState({}, "", "/player");
    }
    if (window.innerWidth <= 1128) queueVisible = false;
    const inviteCode =
      new URLSearchParams(window.location.search).get("party") ||
      localStorage.getItem("wavelength-pending-invite");
    if (inviteCode) {
      pendingInviteCode = inviteCode.trim().toUpperCase();
      partyCode = pendingInviteCode;
      partyDetailsVisible = true;
      queueVisible = false;
    }
    if (activeView === "queue" && window.innerWidth > 735)
      activeView = "overview";
    loadLyrics(playing);
    const handleNavigation = (event) => {
      if (lyricsFullscreen) {
        if (!event.state?.lyricsFullscreen) window.history.forward();
        return;
      }
      closeContentModals();
      activeView = viewFromPath(window.location.pathname);
      const currentPath = appNavigationStack.at(-1);
      const targetPath = window.location.pathname;
      if (currentPath !== targetPath) {
        if (appNavigationStack.at(-2) === targetPath) {
          forwardNavigationStack = [
            ...forwardNavigationStack,
            currentPath,
          ];
          appNavigationStack = appNavigationStack.slice(0, -1);
        } else if (forwardNavigationStack.at(-1) === targetPath) {
          forwardNavigationStack = forwardNavigationStack.slice(0, -1);
          appNavigationStack = [...appNavigationStack, targetPath];
        } else {
          appNavigationStack = [...appNavigationStack, targetPath];
          forwardNavigationStack = [];
        }
        persistNavigationHistory();
      }
      selectedCollectionId =
        activeView === "collection"
          ? decodeURIComponent(
              window.location.pathname.slice("/collection/".length),
            )
          : "";
      if (
        activeView === "equalizer" ||
        activeView === "customization" ||
        activeView === "collection"
      ) {
        queueVisible = false;
        partyDetailsVisible = false;
      }
      if (activeView === "logs") startLogsAutoRefresh();
      else stopLogsAutoRefresh();
      if (partyDetailsVisible) {
        partyDetailsVisible = false;
        queueVisible = false;
      }
      if (activeView === "now-playing" && window.innerWidth > 735) {
        activeView = "overview";
        window.history.replaceState({}, "", "/player");
      }
      if (activeView === "queue" && window.innerWidth > 735)
        activeView = "overview";
    };
    const updateOverflow = () => {
      if (
        showFolderManager ||
        qobuzCredentialsOpen ||
        showAppUpdatePrompt ||
        showCreatePlaylist ||
        showAccentPicker
      )
        tick().then(updateContentModalBounds);
      contentHasOverflow =
        contentInner.scrollHeight > contentInner.clientHeight + 1;
      const tooltipRoot = document.getElementById("app");
      tooltipRoot?.querySelectorAll("*").forEach((element) => {
        if (
          element.closest("[data-marquee]") ||
          element.matches("svg, path, input, textarea, select, script, style")
        )
          return;
        const label = element.textContent?.trim().replace(/\s+/g, " ");
        if (!label || !element.clientWidth) return;
        const style = getComputedStyle(element);
        const singleLineClipped =
          style.textOverflow === "ellipsis" &&
          element.scrollWidth > element.clientWidth + 1;
        const clamped =
          style.webkitLineClamp !== "none" &&
          style.webkitBoxOrient === "vertical" &&
          element.scrollHeight > element.clientHeight + 1;
        const clipped = singleLineClipped || clamped;
        if (
          clipped &&
          (!element.hasAttribute("title") ||
            element.dataset.autoOverflowTitle === "true")
        ) {
          element.title = label;
          element.dataset.autoOverflowTitle = "true";
        } else if (!clipped && element.dataset.autoOverflowTitle === "true") {
          element.removeAttribute("title");
          delete element.dataset.autoOverflowTitle;
        }
      });
    };
    const handleViewportResize = () => {
      const resizedFromMobile = mobileViewport && window.innerWidth > 735;
      mobileViewport = window.innerWidth <= 735;
      volumeSliderOpen = false;
      if (resizedFromMobile && activeView === "now-playing") {
        activeView = "overview";
        window.history.replaceState({}, "", "/player");
      } else if (resizedFromMobile && activeView === "queue") {
        activeView = "overview";
        queueVisible = true;
        partyDetailsVisible = false;
        window.history.replaceState({}, "", "/player");
      } else if (window.innerWidth <= 1128) queueVisible = false;
      else if (activeView === "queue" || activeView === "now-playing") {
        activeView = "overview";
        window.history.replaceState({}, "", "/player");
      }
      tick().then(updateContentModalBounds);
      updateOverflow();
    };
    const handleLyricsKeydown = (event) => {
      if (lyricsFullscreen && event.key === "Escape") {
        event.preventDefault();
        closeLyricsFullscreen();
      }
    };
    const handleOutsideContextMenu = (event) => {
      if (!event.target.closest(".context-menu")) contextMenu = null;
    };
    const preventNativeContextMenu = (event) => event.preventDefault();
    window.addEventListener("popstate", handleNavigation);
    window.addEventListener("click", handleOutsideContextMenu);
    window.addEventListener("keydown", handleLyricsKeydown);
    window.addEventListener("keydown", handleUiScaleShortcut);
    document.addEventListener("contextmenu", preventNativeContextMenu);
    const librariesReady = Promise.all([loadLibrary(), loadSavedFolders()]);
    librariesReady.then(restoreLastPlayback);
    roomServerReady.then(async () => {
      await librariesReady;
      if (joinedParty) await restorePartySession();
    });
    refreshQueueDownloadStatus();
    downloadRefreshTimer = window.setInterval(() => {
      refreshQueueDownloadStatus();
    }, 2500);
    if (activeView === "logs") startLogsAutoRefresh();
    partyRefreshTimer = window.setInterval(() => {
      if (joinedParty) refreshPartyMembers();
    }, 10000);
    const spotifyResult = new URLSearchParams(window.location.search).get(
      "spotify",
    );
    if (spotifyResult) {
      const callbackUrl = new URL(window.location.href);
      callbackUrl.searchParams.delete("spotify");
      window.history.replaceState(
        {},
        "",
        `${callbackUrl.pathname}${callbackUrl.search}${callbackUrl.hash}`,
      );

    async function loadNowPlayingMetadata(track) {
      const requestId = ++nowPlayingMetadataRequest;
      nowPlayingMetadata = {
        bpm: track?.bpm ?? track?.tempo ?? null,
        scale: track?.scale || "",
        releaseDate: track?.releaseDate || track?.release_date || "",
      };
      if (!track) return;

      const params = new URLSearchParams({
        title: String(track.title || ""),
        artist: String(track.artist || ""),
      });
      const uri = spotifyTrackUri(track);
      if (uri) params.set("spotify_id", uri.slice("spotify:track:".length));

      try {
        const metadata = await api(`/audio/track-metadata?${params}`);
        if (requestId !== nowPlayingMetadataRequest) return;
        nowPlayingMetadata = {
          bpm: metadata.bpm ?? nowPlayingMetadata.bpm ?? null,
          scale: metadata.scale || nowPlayingMetadata.scale || "",
          releaseDate:
            metadata.releaseDate || nowPlayingMetadata.releaseDate || "",
        };
      } catch {}
    }
      if (spotifyResult === "error")
        notice = "Spotify sign-in was not completed.";
    }
    const spotifyCallbackParams = new URLSearchParams(window.location.search);
    if (
      !isTauri() &&
      spotifyCallbackParams.has("state") &&
      (spotifyCallbackParams.has("code") ||
        spotifyCallbackParams.has("error"))
    ) {
      void completeSpotifyWebRedirect().catch((error) => {
        notice = error?.message || "Spotify sign-in could not be completed.";
      });
    } else if (spotifyClientId) {
      void restoreStandaloneSpotifySession().catch((error) => {
        notice = error?.message || "Spotify could not be restored.";
      });
    } else if (spotifyConnected || spotifyResult === "connected") {
      spotifyConnected = false;
      spotifyProfile = null;
      spotifyPlaylists = [];
      spotifySavedTracks = [];
      localStorage.removeItem("wavelength-spotify-connected");
      if (spotifyResult === "connected")
        notice = "Enter a Spotify Client ID in Settings to use Spotify OAuth.";
    }
    const overflowRoot = document.getElementById("app") || contentInner;
    const resizeObserver = new ResizeObserver(() => {
      updateOverflow();
      updateContentModalBounds();
    });
    const observeModalEdges = () => {
      const sidebar = document.querySelector(".workspace-cluster > .sidebar");
      const rail = document.querySelector(
        ".workspace-cluster > .content > .right-rail",
      );
      if (sidebar) resizeObserver.observe(sidebar);
      if (rail) resizeObserver.observe(rail);
    };
    const mutationObserver = new MutationObserver(() => {
      updateOverflow();
      updateContentModalBounds();
      observeModalEdges();
    });
    resizeObserver.observe(overflowRoot);
    if (contentInner) resizeObserver.observe(contentInner);
    observeModalEdges();
    mutationObserver.observe(overflowRoot, {
      childList: true,
      subtree: true,
      characterData: true,
    });
    window.addEventListener("resize", handleViewportResize);
    tick().then(updateOverflow);
    return () => {
      window.removeEventListener("popstate", handleNavigation);
      window.removeEventListener("click", handleOutsideContextMenu);
      window.removeEventListener("keydown", handleLyricsKeydown);
      window.removeEventListener("keydown", handleUiScaleShortcut);
      window.removeEventListener("resize", handleViewportResize);
      document.removeEventListener("contextmenu", preventNativeContextMenu);
      mounted = false;
      unlistenSpotifyOAuth?.();
      systemThemeMedia.removeEventListener?.("change", applyAppearance);
      if (appUpdateStartupTimer) clearTimeout(appUpdateStartupTimer);
      if (appUpdatePollingTimer) clearInterval(appUpdatePollingTimer);
      if (partyRefreshTimer) clearInterval(partyRefreshTimer);
      stopLogsAutoRefresh();
      disconnectPartySocket();
      if (partyPublishTimer) clearTimeout(partyPublishTimer);
      if (downloadRefreshTimer) clearInterval(downloadRefreshTimer);
      if (noticeTimer) clearTimeout(noticeTimer);
      if (inviteCopyTimer) clearTimeout(inviteCopyTimer);
      stopEqualizerSpectrum();
      equalizerContext?.close().catch(() => {});
      resizeObserver.disconnect();
      mutationObserver.disconnect();
    };
  });

  async function api(path, options = {}) {
    const headers = {
      "Content-Type": "application/json",
      ...(options.headers || {}),
    };
    const roomRequest = path.startsWith("/parties");
    const localAudioRequest = isLocalPlayer() && path.startsWith("/audio/");
    if (path.startsWith("/spotify/")) {
      if (!spotifyClientId.trim())
        throw new Error(
          "Enter a Spotify Client ID in Settings to use Spotify OAuth.",
        );
      return spotifyApiForAppPath(path, options);
    }
    if (roomRequest && !roomServerUrl)
      throw new Error(
        "Set a room server URL in Settings to use shared listening.",
      );
    if (roomRequest && partyHostToken)
      headers.Authorization = `Bearer ${partyHostToken}`;
    let baseUrl = "/api";
    if (roomRequest) baseUrl = `${roomServerUrl}/api`;
    else if (localAudioRequest) baseUrl = await localAudioBase();
    const requestPath = localAudioRequest ? path.slice("/audio".length) : path;
    const response = await fetch(`${baseUrl}${requestPath}`, {
      ...options,
      headers,
      credentials: roomRequest ? "omit" : "same-origin",
    });
    const data = await response.json().catch(() => ({}));
    if (roomRequest && response.status === 401 && partyHostToken) {
      partyHostToken = "";
      localStorage.removeItem("wavelength-party-host-token");
    }
    if (!response.ok) throw new Error(data.error || "Request failed");
    return data;
  }

  async function localAudioBase() {
    if (!isTauri()) return "/api/audio";
    if (!localAudioBasePromise) {
      localAudioBasePromise = invoke("get_local_service_base").catch(
        (error) => {
          if (import.meta.env.DEV) return "http://127.0.0.1:8787";
          throw error;
        },
      );
    }
    return localAudioBasePromise;
  }

  function normalizeRoomServerUrl(value) {
    if (!String(value || "").trim()) return "";
    const url = new URL(String(value).trim());
    const localHttp =
      url.protocol === "http:" &&
      ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname);
    if (
      !["http:", "https:"].includes(url.protocol) ||
      url.username ||
      url.password ||
      url.pathname !== "/" ||
      url.search ||
      url.hash ||
      (url.protocol === "http:" && !localHttp)
    ) {
      throw new Error(
        "Use an HTTPS server URL, or HTTP on this device, without a path.",
      );
    }
    return url.origin;
  }

  async function loadRoomServerUrl() {
    try {
      const savedUrl = isTauri()
        ? await invoke("get_server_url")
        : localStorage.getItem("wavelength-room-server-url");
      roomServerUrl = normalizeRoomServerUrl(savedUrl || "");
    } catch {
      roomServerUrl = "";
    }
    roomServerInput = roomServerUrl;
    roomServerUrlLoaded = true;
  }

  async function saveRoomServerUrl() {
    roomServerSaving = true;
    try {
      const normalizedUrl = normalizeRoomServerUrl(roomServerInput);
      if (joinedParty && roomServerUrl !== normalizedUrl) {
        if (partyRole === "host") {
          try {
            await api(`/parties/${encodeURIComponent(joinedParty)}`, {
              method: "DELETE",
            });
          } catch {}
        }
        clearPartySession();
      }
      if (isTauri())
        await invoke("set_server_url", { serverUrl: normalizedUrl });
      else localStorage.setItem("wavelength-room-server-url", normalizedUrl);
      roomServerUrl = normalizedUrl;
      roomServerInput = normalizedUrl;
      notice = normalizedUrl
        ? "Room server saved."
        : "Room server disconnected.";
    } catch (error) {
      notice = error.message || "Room server URL could not be saved.";
    } finally {
      roomServerSaving = false;
    }
  }

  function schedulePartyStatePublish() {
    if (!joinedParty || partyRole !== "host") return;
    if (partyPublishTimer) clearTimeout(partyPublishTimer);
    partyPublishTimer = window.setTimeout(() => {
      partyPublishTimer = null;
      publishPartyState();
    }, 75);
  }

  function sendPartyCommand(action, details, permission) {
    if (partyRole !== "guest" || !joinedParty) return false;
    if (!partyPermissions[permission]) {
      notice = permission === "canAdd"
        ? "The host has disabled guest track additions."
        : permission === "canManageQueue"
          ? "The host has disabled guest queue management."
          : "The host has disabled guest playback controls.";
      return true;
    }
    if (!partySocket?.connected) {
      notice = "The room connection is not ready. Try again shortly.";
      return true;
    }
    partySocket.emit("room-command", { action, ...details });
    return true;
  }

  function publishPartyPlaybackControl() {
    if (
      !joinedParty ||
      partyRole !== "host" ||
      !partySocket?.connected ||
      !playing
    )
      return;
    partySocket.emit("playback-control", {
      currentTrackId: toPartyTrack(playing).id,
      currentTime: Math.max(0, Number(currentTime) || 0),
      isPlaying: Boolean(isPlaying),
      sentAtEpochMs: Date.now(),
    });
  }

  function schedulePartyProgressPublish() {
    if (
      !joinedParty ||
      partyRole !== "host" ||
      !partySocket?.connected ||
      !playing ||
      !isPlaying ||
      partyProgressTimer
    )
      return;
    const delay = Math.max(0, 250 - (Date.now() - partyProgressLastSentAt));
    partyProgressTimer = window.setTimeout(() => {
      partyProgressTimer = null;
      if (
        !joinedParty ||
        partyRole !== "host" ||
        !partySocket?.connected ||
        !playing ||
        !isPlaying
      )
        return;
      partyProgressLastSentAt = Date.now();
      partySocket.emit("playback-progress", {
        currentTrackId: toPartyTrack(playing).id,
        currentTime: Math.max(0, Number(currentTime) || 0),
        duration: Math.max(0, Number(duration) || 0),
      });
    }, delay);
  }

  async function refreshQueueDownloadStatus() {
    if (previewMode) return;
    const downloadableTracks = queue.filter(isQueueDownloadable);
    if (!downloadableTracks.length) {
      queueDownloadStatuses = {};
      return;
    }
    if (queueStatusRefreshInFlight) return;
    const trackIds = [
      ...new Set(
        downloadableTracks
          .filter(
            (track) =>
              !["ready", "error", "cancelled"].includes(
                queueDownloadStatuses[getQueueAudioId(track)]?.status,
              ),
          )
          .map(getQueueAudioId),
      ),
    ];
    if (!trackIds.length) return;
    queueStatusRefreshInFlight = true;
    try {
      const data = await api(
        `/audio/status?video_ids=${encodeURIComponent(trackIds.join(","))}`,
      );
      const updates = data.tracks || {};
      const completedIds = Object.entries(updates)
        .filter(
          ([trackId, status]) =>
            status.status === "ready" &&
            ["queued", "downloading"].includes(
              queueDownloadStatuses[trackId]?.status,
            ),
        )
        .map(([trackId]) => trackId);
      if (completedIds.length) {
        downloadCompletionMarks = {
          ...downloadCompletionMarks,
          ...Object.fromEntries(completedIds.map((trackId) => [trackId, true])),
        };
        completedIds.forEach((trackId) =>
          window.setTimeout(() => {
            if (!downloadCompletionMarks[trackId]) return;
            downloadCompletionMarks = {
              ...downloadCompletionMarks,
              [trackId]: "fading",
            };
            window.setTimeout(() => {
              const marks = { ...downloadCompletionMarks };
              delete marks[trackId];
              downloadCompletionMarks = marks;
            }, 420);
          }, 2380),
        );
      }
      queueDownloadStatuses = { ...queueDownloadStatuses, ...updates };
    } catch {
    } finally {
      queueStatusRefreshInFlight = false;
    }
  }

  async function loadDownloadLogs() {
    if (previewMode && previewPage === "logs") return;
    if (downloadLogsLoading) return;
    downloadLogsLoading = true;
    downloadLogsError = "";
    try {
      downloadLogs = (await api("/audio/logs?limit=200")).logs || [];
    } catch (error) {
      downloadLogsError = error.message;
    } finally {
      downloadLogsLoading = false;
    }
  }

  function startLogsAutoRefresh() {
    if (previewMode && previewPage === "logs") return;
    if (logsRefreshTimer) return;
    void loadDownloadLogs();
    logsRefreshTimer = window.setInterval(() => {
      if (activeView === "logs") void loadDownloadLogs();
    }, 1000);
  }

  function stopLogsAutoRefresh() {
    if (logsRefreshTimer) clearInterval(logsRefreshTimer);
    logsRefreshTimer = null;
  }

  async function restoreQobuzCredentials() {
    try {
      if (
        qobuzCredentials.appId &&
        qobuzCredentials.appSecret &&
        qobuzCredentials.userToken
      ) {
        const result = await api("/audio/qobuz/settings", {
          method: "PUT",
          body: JSON.stringify({
            app_id: qobuzCredentials.appId,
            app_secret: qobuzCredentials.appSecret,
            user_token: qobuzCredentials.userToken,
          }),
        });
        qobuzConfigured = Boolean(result.configured);
      }
    } catch {
      qobuzConfigured = false;
    }
  }

  async function saveQobuzCredentials() {
    if (
      !qobuzCredentials.appId?.trim() ||
      !qobuzCredentials.appSecret?.trim() ||
      !qobuzCredentials.userToken?.trim()
    ) {
      notice = "Enter the Qobuz App ID, App secret, and User token.";
      return;
    }
    qobuzSaving = true;
    try {
      const credentials = {
        appId: qobuzCredentials.appId.trim(),
        appSecret: qobuzCredentials.appSecret.trim(),
        userToken: qobuzCredentials.userToken.trim(),
      };
      const result = await api("/audio/qobuz/settings", {
        method: "PUT",
        body: JSON.stringify({
          app_id: credentials.appId,
          app_secret: credentials.appSecret,
          user_token: credentials.userToken,
        }),
      });
      localStorage.setItem(
        "wavelength-qobuz-credentials",
        JSON.stringify(credentials),
      );
      qobuzCredentials = credentials;
      qobuzConfigured = Boolean(result.configured);
      qobuzCredentialsOpen = false;
      notice = "Qobuz credentials saved.";
    } catch (error) {
      notice = error.message || "Qobuz credentials could not be saved.";
    } finally {
      qobuzSaving = false;
    }
  }

  async function checkForAppUpdates(silent = false) {
    if (!isTauri() || appUpdateChecking || showAppUpdatePrompt) return;
    appUpdateChecking = true;
    if (!silent) appUpdateStatus = "Checking signed releases";
    try {
      const update = await checkDesktopUpdate();
      if (!update) {
        if (!silent) {
          appUpdateStatus = "Wavelength is up to date";
          notice = appUpdateStatus;
        }
        return;
      }
      pendingDesktopUpdate = update;
      appUpdateStatus = `Version ${update.version} is available`;
      updateContentModalBounds();
      showAppUpdatePrompt = true;
      notice = `Wavelength ${update.version} is ready to install.`;
    } catch (error) {
      if (!silent) {
        appUpdateStatus = error.message || "Update check failed";
        notice = appUpdateStatus;
      }
    } finally {
      appUpdateChecking = false;
    }
  }

  async function installPendingAppUpdate() {
    if (!pendingDesktopUpdate || appUpdateInstalling) return;
    appUpdateInstalling = true;
    appUpdateStatus = `Installing version ${pendingDesktopUpdate.version}`;
    try {
      await pendingDesktopUpdate.downloadAndInstall();
      await relaunch();
    } catch (error) {
      appUpdateStatus = error.message || "Update installation failed";
      notice = appUpdateStatus;
      showAppUpdatePrompt = false;
      pendingDesktopUpdate = null;
    } finally {
      appUpdateInstalling = false;
    }
  }

  function dismissAppUpdatePrompt() {
    showAppUpdatePrompt = false;
    pendingDesktopUpdate = null;
    appUpdateStatus = "Update available; installation postponed";
  }

  async function preloadQueueTrack(
    track,
    { scope = joinedParty ? "room" : "permanent" } = {},
  ) {
    if (!isQueueDownloadable(track)) return;
    const trackId = getQueueAudioId(track);
    queueDownloadStatuses = {
      ...queueDownloadStatuses,
      [trackId]: { status: "queued", progress: 0 },
    };
    try {
      await api(
        `/audio/preload?video_ids=${encodeURIComponent(trackId)}&scope=${scope}`,
        { method: "POST" },
      );
      await refreshQueueDownloadStatus();
    } catch (error) {
      queueDownloadStatuses = {
        ...queueDownloadStatuses,
        [trackId]: { status: "error", error: error.message },
      };
    }
  }

  function scheduleNoticeDismiss() {
    notificationLogs = [
      { timestamp: Date.now(), message: notice },
      ...notificationLogs,
    ].slice(0, 200);
    try {
      localStorage.setItem(
        "wavelength-notification-logs",
        JSON.stringify(notificationLogs),
      );
    } catch {}
    if (noticeTimer) clearTimeout(noticeTimer);
    noticeRemaining = 3000;
    noticeDeadline = Date.now() + noticeRemaining;
    noticeTimer = window.setTimeout(() => {
      notice = "";
      noticeTimer = null;
    }, noticeRemaining);
  }
  function pauseNoticeDismiss() {
    if (!noticeTimer) return;
    clearTimeout(noticeTimer);
    noticeTimer = null;
    noticeRemaining = Math.max(0, noticeDeadline - Date.now());
  }
  function resumeNoticeDismiss() {
    if (!notice || noticeTimer) return;
    if (noticeRemaining <= 0) {
      notice = "";
      return;
    }
    noticeDeadline = Date.now() + noticeRemaining;
    noticeTimer = window.setTimeout(() => {
      notice = "";
      noticeTimer = null;
    }, noticeRemaining);
  }
  function dismissNotice() {
    if (noticeTimer) clearTimeout(noticeTimer);
    noticeTimer = null;
    notice = "";
  }

  function randomSpotifyValue(length = 64) {
    const characters =
      "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const acceptanceLimit =
      Math.floor(256 / characters.length) * characters.length;
    let value = "";
    while (value.length < length) {
      const bytes = crypto.getRandomValues(
        new Uint8Array(length - value.length),
      );
      for (const byte of bytes) {
        if (byte < acceptanceLimit)
          value += characters[byte % characters.length];
        if (value.length === length) break;
      }
    }
    return value;
  }

  function encodeBase64Url(bytes) {
    return btoa(String.fromCharCode(...new Uint8Array(bytes)))
      .replace(/=/g, "")
      .replace(/\+/g, "-")
      .replace(/\//g, "_");
  }

  async function createSpotifyPkceFlow() {
    const verifier = randomSpotifyValue();
    const digest = await crypto.subtle.digest(
      "SHA-256",
      new TextEncoder().encode(verifier),
    );
    return {
      verifier,
      challenge: encodeBase64Url(digest),
      state: randomSpotifyValue(48),
      clientId: spotifyClientId.trim(),
      redirectUri: isTauri()
        ? "http://127.0.0.1:43821/callback"
        : `${window.location.origin}${window.location.pathname}`,
    };
  }

  async function persistSpotifyRefreshToken(token) {
    spotifyRefreshToken = token || "";
    if (isTauri()) {
      await invoke("set_spotify_refresh_token", {
        refreshToken: spotifyRefreshToken || null,
      });
    } else if (spotifyRefreshToken) {
      localStorage.setItem("wavelength-spotify-refresh-token", spotifyRefreshToken);
    } else {
      localStorage.removeItem("wavelength-spotify-refresh-token");
    }
  }

  async function refreshSpotifyAccessToken() {
    if (!spotifyRefreshToken)
      throw new Error("Connect Spotify to load your Spotify library.");
    if (spotifyRefreshPromise) return spotifyRefreshPromise;

    const refreshToken = spotifyRefreshToken;
    spotifyRefreshPromise = (async () => {
      const response = await fetch("https://accounts.spotify.com/api/token", {
        method: "POST",
        headers: {
          "Content-Type": "application/x-www-form-urlencoded",
        },
        body: new URLSearchParams({
          grant_type: "refresh_token",
          refresh_token: refreshToken,
          client_id: spotifyClientId.trim(),
        }),
      });
      const data = await response.json();
      if (!response.ok)
        throw new Error(
          data.error_description || data.error || "Spotify token refresh failed.",
        );
      if (typeof data.access_token !== "string")
        throw new Error("Spotify returned an invalid access token.");

      spotifyAccessToken = data.access_token;
      spotifyAccessTokenExpiresAt =
        Date.now() + Math.max(0, Number(data.expires_in) || 3600) * 1000;
      if (typeof data.refresh_token === "string" && data.refresh_token)
        await persistSpotifyRefreshToken(data.refresh_token);
      return spotifyAccessToken;
    })();

    try {
      return await spotifyRefreshPromise;
    } finally {
      spotifyRefreshPromise = null;
    }
  }

  async function getSpotifyAccessToken() {
    if (
      spotifyAccessToken &&
      spotifyAccessTokenExpiresAt > Date.now() + 60_000
    )
      return spotifyAccessToken;
    return refreshSpotifyAccessToken();
  }

  async function spotifyApi(path, options = {}, retryUnauthorized = true) {
    const accessToken = await getSpotifyAccessToken();
    const headers = {
      Accept: "application/json",
      Authorization: `Bearer ${accessToken}`,
      ...(options.body ? { "Content-Type": "application/json" } : {}),
      ...(options.headers || {}),
    };
    const url = path.startsWith("https://")
      ? path
      : `https://api.spotify.com/v1${path}`;
    const response = await fetch(url, { ...options, headers });
    if (response.status === 401 && retryUnauthorized) {
      spotifyAccessTokenExpiresAt = 0;
      await getSpotifyAccessToken();
      return spotifyApi(path, options, false);
    }
    const data = response.status === 204 ? {} : await response.json();
    if (!response.ok)
      throw Object.assign(
        new Error(
          data.error?.message ||
            data.error_description ||
            "Spotify request failed.",
        ),
        { status: response.status },
      );
    return data;
  }

  async function spotifyApiItems(path) {
    const nextUrl = path.startsWith("https://")
      ? new URL(path)
      : new URL(`https://api.spotify.com/v1${path}`);
    const items = [];
    let pageUrl = nextUrl;
    while (pageUrl) {
      if (
        pageUrl.origin !== "https://api.spotify.com" ||
        !pageUrl.pathname.startsWith("/v1/")
      )
        throw new Error("Spotify returned an invalid pagination URL.");
      const page = await spotifyApi(
        `${pageUrl.pathname.slice(3)}${pageUrl.search}`,
      );
      if (Array.isArray(page.items)) items.push(...page.items);
      pageUrl = page.next ? new URL(page.next) : null;
    }
    return items;
  }

  async function fetchSpotifyLibraryDirect() {
    const [savedItems, playlists] = await Promise.all([
      spotifyApiItems("/me/tracks?limit=50"),
      spotifyApiItems("/me/playlists?limit=50"),
    ]);
    const playlistsWithTracks = [];
    let unavailablePlaylistCount = 0;
    for (let index = 0; index < playlists.length; index += 4) {
      const batch = await Promise.all(
        playlists.slice(index, index + 4).map(async (playlist) => {
          try {
            const items = await spotifyApiItems(
              `/playlists/${encodeURIComponent(playlist.id)}/items?limit=50`,
            );
            return {
              ...playlist,
              spotifyId: playlist.id,
              tracks: items
                .map((item) => item.item || item.track)
                .filter((track) => track && track.type !== "episode"),
            };
          } catch (error) {
            if (error.status !== 403) throw error;
            unavailablePlaylistCount += 1;
            return { ...playlist, spotifyId: playlist.id, tracks: [] };
          }
        }),
      );
      playlistsWithTracks.push(...batch);
    }
    return {
      savedTracks: savedItems.map((item) => item.track).filter(Boolean),
      playlists: playlistsWithTracks,
      unavailablePlaylistCount,
    };
  }

  async function spotifyApiForAppPath(path, options = {}) {
    const method = (options.method || "GET").toUpperCase();
    if (path === "/spotify/me") return spotifyApi("/me");
    if (path === "/spotify/library") return fetchSpotifyLibraryDirect();
    if (path.startsWith("/spotify/search?")) {
      const params = new URL(path, "https://wavelength.invalid").searchParams;
      const searchParams = new URLSearchParams({
        q: params.get("q") || "",
        type: "track",
        limit: params.get("limit") || "15",
        offset: params.get("offset") || "0",
      });
      const results = await spotifyApi(
        `/search?${searchParams}`,
        options.signal ? { signal: options.signal } : {},
      );
      return { results: results.tracks?.items || [] };
    }
    if (path === "/spotify/playlists" && method === "POST") {
      const input = JSON.parse(options.body || "{}");
      const playlist = await spotifyApi("/me/playlists", {
        method: "POST",
        body: JSON.stringify({ name: input.name }),
      });
      return { id: playlist.id, spotifyId: playlist.id };
    }
    const playlistTracks = path.match(
      /^\/spotify\/playlists\/([^/]+)\/tracks$/,
    );
    if (playlistTracks && method === "POST") {
      const input = JSON.parse(options.body || "{}");
      const id = decodeURIComponent(playlistTracks[1]);
      if (!/^[A-Za-z0-9]+$/.test(id))
        throw new Error("The Spotify playlist ID is invalid.");
      return spotifyApi(`/playlists/${id}/items`, {
        method: "POST",
        body: JSON.stringify({ uris: input.trackIds || [] }),
      });
    }
    if (path === "/spotify/saved-tracks" && ["PUT", "DELETE"].includes(method)) {
      const input = JSON.parse(options.body || "{}");
      const ids = (input.trackIds || [])
        .map((id) => String(id).replace(/^spotify:track:/, ""))
        .filter((id) => /^[A-Za-z0-9]+$/.test(id));
      if (!ids.length) throw new Error("No Spotify tracks were provided.");
      for (let index = 0; index < ids.length; index += 40) {
        const uris = ids
          .slice(index, index + 40)
          .map((id) => `spotify:track:${id}`);
        const query = new URLSearchParams({ uris: uris.join(",") });
        await spotifyApi(`/me/library?${query}`, { method });
      }
      return {};
    }
    if (path === "/spotify/logout") return {};
    throw new Error("This Spotify operation is not supported.");
  }

  async function exchangeSpotifyCode(code, flow) {
    const response = await fetch("https://accounts.spotify.com/api/token", {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams({
        grant_type: "authorization_code",
        code,
        redirect_uri: flow.redirectUri,
        client_id: flow.clientId,
        code_verifier: flow.verifier,
      }),
    });
    const data = await response.json();
    if (!response.ok)
      throw new Error(
        data.error_description || data.error || "Spotify sign-in failed.",
      );
    if (
      typeof data.access_token !== "string" ||
      typeof data.refresh_token !== "string"
    )
      throw new Error("Spotify returned incomplete sign-in credentials.");

    spotifyClientId = flow.clientId;
    spotifyAccessToken = data.access_token;
    spotifyAccessTokenExpiresAt =
      Date.now() + Math.max(0, Number(data.expires_in) || 3600) * 1000;
    await persistSpotifyRefreshToken(data.refresh_token);
    const profile = await fetchSpotifyData(true);
    if (!profile) throw new Error("Spotify profile data could not be loaded.");
    return profile;
  }

  async function completeSpotifyOAuth(code, flow) {
    return exchangeSpotifyCode(code, flow);
  }

  async function completeSpotifyWebRedirect() {
    const params = new URLSearchParams(window.location.search);
    const code = params.get("code");
    const error = params.get("error");
    if (!code && !error) return false;

    const flow = {
      clientId: sessionStorage.getItem("wavelength-spotify-oauth-client-id") || "",
      verifier: sessionStorage.getItem("wavelength-spotify-oauth-verifier") || "",
      state: sessionStorage.getItem("wavelength-spotify-oauth-state") || "",
      redirectUri:
        sessionStorage.getItem("wavelength-spotify-oauth-redirect-uri") || "",
    };
    sessionStorage.removeItem("wavelength-spotify-oauth-client-id");
    sessionStorage.removeItem("wavelength-spotify-oauth-verifier");
    sessionStorage.removeItem("wavelength-spotify-oauth-state");
    sessionStorage.removeItem("wavelength-spotify-oauth-redirect-uri");
    window.history.replaceState(
      window.history.state,
      "",
      window.location.pathname,
    );

    if (params.get("state") !== flow.state || !flow.state)
      throw new Error("Spotify sign-in state did not match. Please try again.");
    if (error) throw new Error("Spotify sign-in was not approved.");
    if (!code || !flow.clientId || !flow.verifier || !flow.redirectUri)
      throw new Error("Spotify sign-in returned an incomplete response.");
    await completeSpotifyOAuth(code, flow);
    return true;
  }

  async function restoreStandaloneSpotifySession() {
    spotifyRefreshToken = isTauri()
      ? (await invoke("get_spotify_refresh_token")) || ""
      : localStorage.getItem("wavelength-spotify-refresh-token") || "";
    if (!spotifyRefreshToken) {
      spotifyConnected = false;
      spotifyProfile = null;
      spotifyPlaylists = [];
      spotifySavedTracks = [];
      localStorage.removeItem("wavelength-spotify-connected");
      return;
    }
    try {
      await getSpotifyAccessToken();
      const profile = await fetchSpotifyData();
      if (!profile) throw new Error("Spotify profile data could not be loaded.");
    } catch (error) {
      spotifyAccessToken = "";
      spotifyAccessTokenExpiresAt = 0;
      spotifyConnected = false;
      spotifyProfile = null;
      spotifyPlaylists = [];
      spotifySavedTracks = [];
      localStorage.removeItem("wavelength-spotify-connected");
      throw error;
    }
  }

  function saveSpotifyClientId() {
    const previousClientId =
      localStorage.getItem("wavelength-spotify-client-id") || "";
    spotifyClientId = spotifyClientId.trim();
    localStorage.setItem("wavelength-spotify-client-id", spotifyClientId);
    if (previousClientId === spotifyClientId) return;

    spotifyAccessToken = "";
    spotifyAccessTokenExpiresAt = 0;
    spotifyConnected = false;
    spotifyProfile = null;
    spotifyPlaylists = [];
    spotifySavedTracks = [];
    localStorage.removeItem("wavelength-spotify-connected");
    void persistSpotifyRefreshToken(null).catch((error) => {
      notice = `Spotify credentials could not be cleared: ${error.message}`;
    });
    notice = "Spotify Client ID saved. Connect Spotify to continue.";
  }

  async function fetchSpotifyData(announce = false) {
    try {
      const profile = await api("/spotify/me");
      if (!profile.id)
        throw new Error(profile.error?.message || "Spotify connection failed.");
      spotifyProfile = profile;
      spotifyConnected = true;
      localStorage.setItem("wavelength-spotify-connected", "true");
      const unavailablePlaylistCount = await fetchSpotifyLibrary();
      if (announce)
        notice = `Spotify connected as ${profile.display_name || profile.id}.`;
      if (unavailablePlaylistCount) {
        notice = `Spotify denied access to tracks in ${unavailablePlaylistCount} playlist${unavailablePlaylistCount === 1 ? "" : "s"}.`;
      }
      return profile;
    } catch (error) {
      spotifyConnected = false;
      spotifyProfile = null;
      spotifyPlaylists = [];
      spotifySavedTracks = [];
      localStorage.removeItem("wavelength-spotify-connected");
      if (announce) notice = error.message || "Spotify connection failed.";
      return null;
    }
  }

  function normalizeSpotifyTrack(track) {
    const id =
      track?.id || String(track?.uri || "").replace(/^spotify:track:/, "");
    if (!id) return null;
    return {
      id,
      videoId: id,
      title: track.name || "Unknown track",
      artist:
        track.artists
          ?.map((artist) => artist.name)
          .filter(Boolean)
          .join(", ") || "Unknown artist",
      album: track.album?.name || "",
      length: track.duration_ms ? formatTime(track.duration_ms / 1000) : "—",
      art: track.album?.images?.[0]?.url || "",
      color: "#718979",
      source: "spotify",
      spotify_uri: track.uri || `spotify:track:${id}`,
      spotify_url:
        track.external_urls?.spotify || `https://open.spotify.com/track/${id}`,
    };
  }

  async function fetchSpotifyLibrary() {
    const data = await api("/spotify/library");
    spotifySavedTracks = (data.savedTracks || [])
      .map(normalizeSpotifyTrack)
      .filter(Boolean);
    spotifyPlaylists = (data.playlists || []).map((playlist) => ({
      ...playlist,
      kind: "Spotify playlist",
      color: "#718979",
      spotifyId: playlist.spotifyId || playlist.id,
      tracks: (playlist.tracks || [])
        .map(normalizeSpotifyTrack)
        .filter(Boolean),
    }));
    for (const playlist of playlists.filter(
      (item) => item.id !== "liked-songs" && !item.spotifyId && !item.localOnly,
    )) {
      await syncPlaylistToSpotify(playlist);
    }
    return data.unavailablePlaylistCount || 0;
  }

  async function connectSpotify() {
    if (pendingInviteCode)
      localStorage.setItem("wavelength-pending-invite", pendingInviteCode);

    if (!spotifyClientId.trim()) {
      notice = "Enter a Spotify Client ID in Settings to use Spotify OAuth.";
      navigate("/settings");
      return;
    }

    try {
      if (!/^[A-Za-z0-9]{16,64}$/.test(spotifyClientId.trim()))
        throw new Error("Enter a valid Spotify Client ID in Settings.");
      const flow = await createSpotifyPkceFlow();
      if (isTauri()) {
        pendingSpotifyOAuth = flow;
        await invoke("start_spotify_pkce_oauth", {
          clientId: flow.clientId,
          codeChallenge: flow.challenge,
          state: flow.state,
        });
      } else {
        sessionStorage.setItem(
          "wavelength-spotify-oauth-client-id",
          flow.clientId,
        );
        sessionStorage.setItem(
          "wavelength-spotify-oauth-verifier",
          flow.verifier,
        );
        sessionStorage.setItem("wavelength-spotify-oauth-state", flow.state);
        sessionStorage.setItem(
          "wavelength-spotify-oauth-redirect-uri",
          flow.redirectUri,
        );
        const authorizeUrl = new URL("https://accounts.spotify.com/authorize");
        authorizeUrl.search = new URLSearchParams({
          response_type: "code",
          client_id: flow.clientId,
          redirect_uri: flow.redirectUri,
          scope:
            "user-read-private user-read-email user-library-read user-library-modify playlist-read-private playlist-read-collaborative playlist-modify-public playlist-modify-private",
          code_challenge_method: "S256",
          code_challenge: flow.challenge,
          state: flow.state,
        }).toString();
        window.location.assign(authorizeUrl.toString());
      }
    } catch (error) {
      pendingSpotifyOAuth = null;
      notice = error?.message || "Spotify sign-in could not be opened.";
    }
  }

  async function disconnectSpotify() {
    let logoutError = null;
    try {
      await persistSpotifyRefreshToken(null);
    } catch (error) {
      logoutError = error;
    }
    spotifyConnected = false;
    spotifyProfile = null;
    spotifyPlaylists = [];
    spotifySavedTracks = [];
    spotifyAccessToken = "";
    spotifyAccessTokenExpiresAt = 0;
    spotifyRefreshToken = "";
    localStorage.removeItem("wavelength-spotify-connected");
    localStorage.removeItem("wavelength-spotify-profile");
    localStorage.removeItem("wavelength-spotify-token");
    notice = logoutError
      ? `Disconnected from Spotify locally, but remote logout failed: ${logoutError.message}`
      : "Spotify disconnected.";
  }

  async function loadLibrary() {
    if (isLocalPlayer()) {
      onlineTracks = [];
      tracks = [...importedTracks];
      return;
    }
    try {
      const data = await api("/library");
      if (data.tracks?.length)
        onlineTracks = data.tracks.map((track) => ({
          ...track,
          color: "#718979",
          length: track.length || "—",
        }));
      tracks = [...onlineTracks, ...importedTracks];
      playing = tracks[0] || playing;
    } catch {
      notice = "Local library is not available yet. Check the mounted folder.";
    }
  }

  async function chooseLocalFolder() {
    if (!window.indexedDB || !window.showDirectoryPicker) {
      notice =
        "Folder access is not supported in this browser. Use Chrome or Edge.";
      return;
    }
    try {
      const directory = await window.showDirectoryPicker();
      if (!localFolders.some((folder) => folder.name === directory.name))
        localFolders = [
          ...localFolders,
          { name: directory.name, handle: directory },
        ];
      await saveFolders();
      await reloadImportedTracks();
      notice = `${directory.name} is now part of your imported library.`;
    } catch (error) {
      if (error?.name !== "AbortError") notice = "Could not open that folder.";
    }
  }

  function updateContentModalBounds() {
    const bounds = contentInner?.getBoundingClientRect();
    const app = document.getElementById("app");
    const content = contentInner?.closest(".content");
    const shell = document.querySelector(".app-shell");
    const sidebar = shell?.querySelector(".sidebar");
    const rail = content?.querySelector(".right-rail");
    const contentBounds = content?.getBoundingClientRect();
    const shellBounds = shell?.getBoundingClientRect();
    if (!bounds || !app || !contentBounds || !shellBounds) return;
    const scale =
      Number.parseFloat(
        getComputedStyle(app).getPropertyValue("--app-scale"),
      ) || 1;
    const desktopLayout = window.innerWidth > 735;
    const sidebarBounds = sidebar?.getBoundingClientRect();
    const railBounds = rail?.getBoundingClientRect();
    const leftEdge =
      desktopLayout && sidebarBounds ? sidebarBounds.right : contentBounds.left;
    const rightEdge =
      desktopLayout && railBounds ? railBounds.left : contentBounds.right;
    contentModalBounds = {
      left: (leftEdge - shellBounds.left) / scale,
      top: (bounds.top - shellBounds.top) / scale,
      contentLeft: (leftEdge - contentBounds.left) / scale,
      contentTop: (bounds.top - contentBounds.top) / scale,
      width: Math.max(0, rightEdge - leftEdge) / scale,
      height: bounds.height / scale,
      centerX: (bounds.left + bounds.width / 2 - contentBounds.left) / scale,
      centerY: (bounds.top + bounds.height / 2 - contentBounds.top) / scale,
    };
  }

  function openFolderManager() {
    updateContentModalBounds();
    showFolderManager = true;
  }

  function openQobuzCredentials() {
    updateContentModalBounds();
    qobuzCredentialsOpen = true;
  }

  async function rescanLocalFolders() {
    if (!window.indexedDB || !window.showDirectoryPicker) {
      notice =
        "Folder access is not supported in this browser. Use Chrome or Edge.";
      return;
    }
    try {
      await reloadImportedTracks(true);
      notice = "Imported folders refreshed.";
    } catch (error) {
      notice =
        error?.name === "NotAllowedError" ||
        error?.message === "Folder permission was not granted."
          ? "Folder permission was not granted. Allow access and try again."
          : error?.name === "NotFoundError"
            ? "A saved folder is no longer available. Remove it and add it again."
            : "Could not read the saved music folders.";
    }
  }

  async function removeLocalFolder(folder) {
    localFolders = localFolders.filter((item) => item.name !== folder.name);
    await saveFolders();
    importedTracks = importedTracks.filter(
      (track) =>
        track.folderPath !== folder.name &&
        !track.folderPath.startsWith(`${folder.name}/`),
    );
    tracks = [...onlineTracks, ...importedTracks];
    notice = `${folder.name} removed from your imported library.`;
  }

  async function runSearch({
    limit = 10,
    offset = 0,
    append = false,
    quiet = false,
    target = "page",
  } = {}) {
    searchAbortController?.abort();
    searchAbortController = null;
    const query = search.trim();
    const requestId = ++searchRequestId;
    if (!downloadFolderReady) {
      topbarSearchResults = [];
      searchResults = [];
      searchLoading = false;
      searchLoadingMore = false;
      searchHasMore = false;
      return;
    }
    if (!query) {
      if (target === "topbar") {
        topbarSearchResults = [];
        return;
      }
      searchResults = [];
      searchLoading = false;
      searchOffset = 0;
      searchTotal = 0;
      searchHasMore = false;
      searchAtBottom = false;
      return;
    }
    const spotifyLink =
      /(?:open\.spotify\.com\/(?:intl-[a-z-]+\/)?(?:track|album|playlist)\/|spotify:(?:track|album|playlist):)/i.test(
        query,
      );
    const youtubeLink =
      /(?:youtu\.be\/|(?:youtube\.com|youtube-nocookie\.com)\/(?:watch|shorts|embed|live))/i.test(
        query,
      );
    const source = spotifyLink
      ? "spotify"
      : youtubeLink
        ? "youtube"
        : searchSource;
    if (source !== searchSource && (spotifyLink || youtubeLink))
      searchSource = source;
    const localMatches =
      offset === 0 && isLocalPlayer()
        ? tracks.filter((track) =>
            `${track.title} ${track.artist} ${track.album}`
              .toLowerCase()
              .includes(query.toLowerCase()),
          )
        : [];
    if (target === "page") searchLoading = true;
    if (offset === 0) {
      if (target === "topbar") topbarSearchResults = localMatches;
      else if (!append) searchResults = localMatches;
    }
    const abortController = new AbortController();
    searchAbortController = abortController;
    const useSpotifyAccountSearch =
      source === "spotify" && !spotifyLink && spotifyConnected;
    try {
      if (
        isTauri() &&
        source === "spotify" &&
        !spotifyLink &&
        !spotifyConnected
      ) {
        throw new Error(
          "Connect Spotify in Settings to search by text, or paste a Spotify track, album, or playlist link.",
        );
      }
      const data = await api(
        useSpotifyAccountSearch
          ? `/spotify/search?q=${encodeURIComponent(query)}&limit=${limit}&offset=${offset}`
          : `/audio/search?q=${encodeURIComponent(query)}&source=${source}&limit=${limit}&offset=${offset}`,
        { signal: abortController.signal },
      );
      if (requestId !== searchRequestId || query !== search.trim()) return;
      const remoteResults = (
        data.results ||
        data.tracks ||
        data.items ||
        []
      ).map((item, index) => ({
        ...item,
        id:
          item.id ||
          item.videoId ||
          `${source}-${offset + index}-${String(
            item.title || item.name || "track",
          )
            .toLowerCase()
            .replace(/[^a-z0-9]+/g, "-")}`,
        title: item.title || item.name,
        artist: item.artist || item.artists?.[0]?.name || "Unknown artist",
        album: item.album || item.album_name || data.playlist?.name || "",
        length:
          item.length ||
          (item.duration
            ? `${Math.floor(item.duration / 60)}:${String(Math.floor(item.duration % 60)).padStart(2, "0")}`
            : "—"),
        color: item.color || "#718979",
        art: getTrackArt(item),
        source: item.source || source,
        mediaType: item.mediaType || item.type || "track",
      }));
      const localIds = new Set(localMatches.map((track) => track.id));
      const pageResults = [
        ...localMatches,
        ...remoteResults.filter((track) => !localIds.has(track.id)),
      ];
      if (target === "topbar") {
        topbarSearchResults = [
          ...new Map(pageResults.map((track) => [track.id, track])).values(),
        ];
        return;
      }
      searchResults = append
        ? [
            ...new Map(
              [...searchResults, ...pageResults].map((track) => [
                track.id,
                track,
              ]),
            ).values(),
          ]
        : [...new Map(pageResults.map((track) => [track.id, track])).values()];
      searchTotal = Number(data.total ?? offset + remoteResults.length);
      searchOffset = offset + limit;
      searchHasMore =
        !spotifyLink &&
        source === "spotify" &&
        remoteResults.length > 0 &&
        searchOffset < searchTotal;
      await tick();
      updateSearchAtBottom();
      if (!searchResults.length && !quiet) notice = "No matching tracks found.";
    } catch (error) {
      if (
        abortController.signal.aborted ||
        requestId !== searchRequestId ||
        query !== search.trim()
      )
        return;
      if (target === "topbar") {
        topbarSearchResults = localMatches;
        return;
      }
      if (!append) searchResults = localMatches;
      searchHasMore = false;
      await tick();
      updateSearchAtBottom();
      if (!quiet) {
        notice = error.message || (localMatches.length
          ? "Audio service is unavailable; showing local matches."
          : "Audio service is unavailable and there are no local matches.");
      }
    } finally {
      if (searchAbortController === abortController)
        searchAbortController = null;
      if (target === "page" && requestId === searchRequestId) {
        searchLoading = false;
        await tick();
        updateSearchAtBottom();
      }
    }
  }

  async function loadMoreSearchResults() {
    if (
      !downloadFolderReady ||
      activeView !== "search" ||
      searchLoadingMore ||
      !searchHasMore
    )
      return;
    searchLoadingMore = true;
    try {
      await runSearch({
        limit: 10,
        offset: searchOffset,
        append: true,
        quiet: true,
      });
    } finally {
      searchLoadingMore = false;
    }
  }

  function updateSearchAtBottom() {
    if (activeView !== "search" || !contentInner) {
      searchAtBottom = false;
      return;
    }
    const tableIsScrollable =
      searchTable && searchTable.scrollHeight > searchTable.clientHeight + 1;
    const scroller = tableIsScrollable ? searchTable : contentInner;
    const canScroll = scroller.scrollHeight > scroller.clientHeight + 1;
    searchAtBottom =
      !canScroll ||
      scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight <= 2;
  }

  function handleSearchScroll(event) {
    const scroller = event.currentTarget;
    if (activeView !== "search") return;
    updateSearchAtBottom();
    if (searchAtBottom) {
      void tick().then(() => {
        if (!searchAtBottom) return;
        const tableIsScrollable =
          searchTable &&
          searchTable.scrollHeight > searchTable.clientHeight + 1;
        const resultsScroller = tableIsScrollable ? searchTable : contentInner;
        resultsScroller.scrollTop = resultsScroller.scrollHeight;
      });
    }
    if (
      scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight <
      320
    )
      loadMoreSearchResults();
  }

  async function submitTopbarSearch() {
    clearTimeout(topbarSearchTimer);
    searchAbortController?.abort();
    searchAbortController = null;
    if (!downloadFolderReady) return;
    topbarSearchOpen = false;
    topbarSearchResults = [];
    searchRequestId += 1;
    searchResults = [];
    searchOffset = 0;
    searchTotal = 0;
    searchHasMore = false;
    searchAtBottom = false;
    if (activeView !== "search") navigate("/search");
    await tick();
    if (contentInner) contentInner.scrollTop = 0;
    await runSearch({ limit: 10 });
  }

  function chooseTopbarSuggestion(track) {
    if (!downloadFolderReady) return;
    topbarSearchOpen = false;
    playTrack(track);
  }

  function handleTopbarSearchInput() {
    const pageSearch = activeView === "search";
    topbarSearchOpen = !pageSearch;
    clearTimeout(topbarSearchTimer);
    searchAbortController?.abort();
    searchAbortController = null;
    searchRequestId += 1;
    if (!downloadFolderReady) {
      topbarSearchResults = [];
      searchResults = [];
      searchLoading = false;
      return;
    }
    if (pageSearch) {
      searchResults = [];
      searchOffset = 0;
      searchTotal = 0;
      searchHasMore = false;
      searchAtBottom = false;
    } else topbarSearchResults = [];
    if (!search.trim()) {
      searchLoading = false;
      return;
    }
    const target = pageSearch ? "page" : "topbar";
    topbarSearchTimer = window.setTimeout(
      () => runSearch({ limit: 10, quiet: true, target }),
      260,
    );
  }

  function changeTopbarSearchSource(source) {
    if (!downloadFolderReady) return;
    searchSource = source;
    topbarSearchResults = [];
    if (search.trim()) handleTopbarSearchInput();
  }

  function handleMobileSearchSourceChange(event) {
    if (event.detail) searchSource = event.detail;
    searchResults = [];
    searchOffset = 0;
    searchHasMore = false;
    searchAtBottom = false;
    searchRequestId += 1;
    if (search.trim()) handleMobileSearchInput();
  }

  function handleMobileSearchInput() {
    clearTimeout(topbarSearchTimer);
    searchAbortController?.abort();
    searchAbortController = null;
    searchRequestId += 1;
    searchResults = [];
    searchOffset = 0;
    searchTotal = 0;
    searchHasMore = false;
    searchAtBottom = false;
    if (!downloadFolderReady || !search.trim()) return;
    topbarSearchTimer = window.setTimeout(() => runSearch({ limit: 10 }), 280);
  }

  function queueSource() {
    return queue.length
      ? queue
      : displayedTracks.length
        ? displayedTracks
        : tracks;
  }

  function resolveNextTrack(direction) {
    const source = queueSource();
    if (!source.length) return null;
    const currentIndex = source.findIndex((item) => item.id === playing?.id);
    if (shuffleOn) {
      const options = source.filter((item) => item.id !== playing?.id);
      return options[Math.floor(Math.random() * options.length)] || source[0];
    }
    if (currentIndex === -1) return source[0];
    const nextIndex = currentIndex + direction;
    if (nextIndex < 0) return source[source.length - 1];
    if (nextIndex >= source.length) return source[0];
    return source[nextIndex];
  }

  function toggleRepeat() {
    if (sendPartyCommand("toggle-repeat", {}, "canControlPlayback")) return;
    repeatMode =
      repeatMode === "off" ? "all" : repeatMode === "all" ? "one" : "off";
    schedulePartyStatePublish();
  }

  function toggleShuffle() {
    if (sendPartyCommand("toggle-shuffle", {}, "canControlPlayback")) return;
    shuffleOn = !shuffleOn;
    schedulePartyStatePublish();
  }

  function clearQueue() {
    if (sendPartyCommand("clear-queue", {}, "canManageQueue")) return;
    queue = [];
    removedQueueHistory = [];
    localStorage.setItem("wavelength-queue", "[]");
    schedulePartyStatePublish();
  }

  function toggleImportedFolder(path, depth) {
    const expanded = expandedImportedFolders[path] ?? false;
    expandedImportedFolders = { ...expandedImportedFolders, [path]: !expanded };
  }

  function spotifyTrackUri(track) {
    const uri = track?.spotify_uri || track?.spotifyUri || track?.uri;
    if (typeof uri === "string" && /^spotify:track:[A-Za-z0-9]+$/.test(uri))
      return uri;
    const url = track?.spotify_url || track?.external_urls?.spotify || "";
    const match = String(url).match(
      /open\.spotify\.com\/(?:intl-[a-z-]+\/)?track\/([A-Za-z0-9]+)/i,
    );
    const id =
      match?.[1] ||
      (getTrackSource(track) === "Spotify" &&
      /^[A-Za-z0-9]+$/.test(String(track?.id || ""))
        ? track.id
        : "");
    return id ? `spotify:track:${id}` : "";
  }

  async function syncPlaylistToSpotify(playlist, track = null) {
    if (
      !spotifyConnected ||
      !playlist ||
      playlist.id === "liked-songs" ||
      playlist.localOnly
    )
      return;
    try {
      if (!playlist.spotifyId) {
        const remote = await api("/spotify/playlists", {
          method: "POST",
          body: JSON.stringify({ name: playlist.name }),
        });
        playlist.spotifyId = remote.spotifyId || remote.id;
        playlists = [...playlists];
        savePlaylists();
      }
      const uri = track && spotifyTrackUri(track);
      if (uri && playlist.spotifyId) {
        await api(
          `/spotify/playlists/${encodeURIComponent(playlist.spotifyId)}/tracks`,
          {
            method: "POST",
            body: JSON.stringify({ trackIds: [uri] }),
          },
        );
      }
      if (uri) notice = `Synced ${track.title} to ${playlist.name} on Spotify.`;
    } catch (error) {
      notice = `Saved locally. Spotify sync failed: ${error.message}`;
    }
  }

  async function addTrackToPlaylist(track, destination = "Favorites") {
    if (!track) return;
    const requested = typeof destination === "string"
      ? { name: destination }
      : destination;
    const playlist = (requested.spotifyId &&
      playlists.find((item) => item.spotifyId === requested.spotifyId)) ||
      playlists.find((item) => item.id === requested.id) ||
      playlists.find((item) => item.name === requested.name) || {
        id: requested.id || `playlist-${Date.now()}`,
        name: requested.name || "Favorites",
        tracks: [],
        ...(requested.spotifyId ? { spotifyId: requested.spotifyId } : {}),
      };
    if (
      !playlists.some(
        (item) =>
          item.id === playlist.id ||
          (requested.spotifyId && item.spotifyId === requested.spotifyId),
      )
    ) {
      playlists = [...playlists, playlist];
    }
    if (!(playlist.tracks || []).some((item) => item.id === track.id))
      playlist.tracks = [...(playlist.tracks || []), track];
    savePlaylists();
    notice = `${track.title} added to ${playlist.name}`;
    contextMenu = null;
    playlistAddTarget = null;
    await syncPlaylistToSpotify(playlist, track);
  }

  function addSongsToPlaylist(playlist) {
    playlistAddTarget = playlist;
    contextMenu = null;
    navigate("/search");
  }

  function duplicateSpotifyPlaylist(playlist) {
    const spotifyId = playlist?.spotifyId;
    if (!spotifyId) return;
    if (playlists.some((item) => item.sourceSpotifyId === spotifyId)) {
      notice = `${playlist.name} is already copied to Wavelength.`;
      return;
    }
    const tracks = (playlist.tracks || []).map((track) => ({ ...track }));
    if (!tracks.length) {
      notice = `${playlist.name} has no tracks to copy.`;
      return;
    }
    const copy = {
      id: `playlist-${Date.now()}`,
      name: playlist.name,
      tracks,
      sourceSpotifyId: spotifyId,
      localOnly: true,
    };
    playlists = [...playlists, copy];
    savePlaylists();
    notice = `Copied ${tracks.length} tracks from ${playlist.name} to Wavelength.`;
  }

  function createPlaylist() {
    openCreatePlaylist();
  }

  async function openCreatePlaylist(track = null) {
    createPlaylistName = "";
    createPlaylistTrack = track;
    contextMenu = null;
    updateContentModalBounds();
    showCreatePlaylist = true;
    await tick();
    createPlaylistInput?.focus();
  }

  function closeCreatePlaylist() {
    showCreatePlaylist = false;
    createPlaylistTrack = null;
  }

  function submitCreatePlaylist() {
    const name = createPlaylistName.trim();
    if (!name) return;
    const track = createPlaylistTrack;
    const playlist = {
      id: `playlist-${Date.now()}`,
      name,
      tracks: track ? [track] : [],
    };
    playlists = [...playlists, playlist];
    savePlaylists();
    closeCreatePlaylist();
    if (track) {
      notice = `${track.title} added to ${name}`;
      syncPlaylistToSpotify(playlist, track);
      return;
    }
    navigate(`/collection/${encodeURIComponent(playlist.id)}`);
    notice = `${name} saved locally.`;
    syncPlaylistToSpotify(playlist);
  }

  function openCollection(collectionId) {
    navigate(`/collection/${encodeURIComponent(collectionId)}`);
  }

  function createPlaylistFromTrack(track) {
    openCreatePlaylist(track);
  }

  function showContextMenu(event, track) {
    event.preventDefault();
    const bounds = contentInner?.getBoundingClientRect();
    if (!bounds) return;
    const menuHeight = Math.max(80, Math.min(480, bounds.height - 16));
    const menuWidth = Math.min(280, Math.max(0, bounds.width - 16));
    contextMenu = {
      x: Math.max(
        bounds.left + 8,
        Math.min(bounds.right - menuWidth - 8, event.clientX || event.pageX || bounds.left + 16),
      ),
      y: Math.max(
        bounds.top + 8,
        Math.min(bounds.bottom - menuHeight - 8, event.clientY || event.pageY || bounds.top + 16),
      ),
      maxHeight: menuHeight,
      track,
    };
  }

  function getTrackShareUrl(track) {
    if (!track) return "";
    if (getTrackSource(track) === "Spotify")
      return track.spotify_url || track.external_urls?.spotify || "";
    if (getTrackSource(track) === "YouTube") {
      const videoId = String(track.videoId || track.id || "").replace(
        /^yt-/,
        "",
      );
      return (
        track.url ||
        (videoId ? `https://www.youtube.com/watch?v=${videoId}` : "")
      );
    }
    return "";
  }

  async function copyTrackLink(track) {
    const url = getTrackShareUrl(track);
    if (!url) {
      notice = "No share link is available for this track.";
      contextMenu = null;
      return;
    }
    try {
      await navigator.clipboard.writeText(url);
      notice = "Song link copied.";
    } catch {
      notice = "Could not copy the song link.";
    }
    contextMenu = null;
  }

  function openExternalPage(url) {
    contextMenu = null;
    if (isTauri()) {
      notice = "External websites cannot be opened inside Wavelength.";
      return;
    }
    if (url) window.open(url, "_blank", "noopener,noreferrer");
  }

  function openSongDetails(track) {
    songDetailsTrack = track;
    contextMenu = null;
  }

  function handleSongPointerDown(event, track) {
    if (event.pointerType === "touch" || event.pointerType === "pen") {
      event.currentTarget.dataset.startX = event.clientX;
      clearTimeout(contextMenuTimer);
      contextMenuTimer = setTimeout(() => showContextMenu(event, track), 480);
    }
  }

  function handleSongTouchEnd(event, track) {
    if (event.pointerType !== "touch") return;
    if (
      event.clientX - (event.currentTarget?.dataset?.startX || event.clientX) <
      -60
    )
      addToQueue(track);
  }

  function clearContextMenuTimer() {
    if (contextMenuTimer) clearTimeout(contextMenuTimer);
    contextMenuTimer = null;
  }

  function persistFavorites() {
    const savedFavorites = favorites.map(({ file, ...track }) => ({
      ...track,
      source: getTrackSource({ ...track, file }).toLowerCase(),
    }));
    localStorage.setItem(
      "wavelength-favorites",
      JSON.stringify(savedFavorites),
    );
    const liked = playlists.find(
      (playlist) =>
        playlist.id === "liked-songs" || playlist.name === "Liked Songs",
    );
    if (liked) {
      liked.tracks = savedFavorites;
      playlists = [...playlists];
      savePlaylists();
    }
  }

  function toggleFavorite(track) {
    if (!track) return;
    const removing = isFavorite(track);
    favorites = removing
      ? favorites.filter((item) => item.id !== track.id)
      : [
          ...favorites,
          { ...track, source: getTrackSource(track).toLowerCase() },
        ];
    persistFavorites();
    const spotifyUri = spotifyTrackUri(track);
    if (spotifyConnected && spotifyUri) {
      spotifySavedTracks = removing
        ? spotifySavedTracks.filter((item) => item.id !== track.id)
        : [
            ...spotifySavedTracks.filter((item) => item.id !== track.id),
            { ...track, source: "spotify", spotify_uri: spotifyUri },
          ];
      api("/spotify/saved-tracks", {
        method: removing ? "DELETE" : "PUT",
        body: JSON.stringify({ trackIds: [spotifyUri] }),
      }).catch((error) => {
        notice = `Saved locally. Spotify liked songs could not be updated: ${error.message}`;
      });
    }
  }

  function likeTracks(tracksToLike) {
    favorites = [
      ...new Map(
        [
          ...favorites,
          ...tracksToLike
            .filter(Boolean)
            .map((track) => ({
              ...track,
              source: getTrackSource(track).toLowerCase(),
            })),
        ].map((track) => [track.id, track]),
      ).values(),
    ];
    persistFavorites();
  }
  async function addToQueue(track) {
    if (!track || queue.some((item) => item.id === track.id)) return;
    removedQueueHistory = removedQueueHistory.filter(
      (item) => item.id !== track.id,
    );
    if (partyRole === "guest") {
      if (!isQueueDownloadable(track) && !track.serverAudioId) {
        notice = "This track is not available to share in the room.";
        return;
      }
      sendPartyCommand(
        "add-track",
        { track: toPartyTrack(track) },
        "canAdd",
      );
      return;
    }
    try {
      track = await ensureLocalTrackUploaded(track);
    } catch (error) {
      notice = `Track added locally but could not be shared: ${error.message}`;
    }
    queue = [...queue, track];
    localStorage.setItem("wavelength-queue", JSON.stringify(queue));
    preloadQueueTrack(track, { scope: joinedParty ? "room" : "permanent" });
    schedulePartyStatePublish();
    if (mobileViewport) {
      if (window.location.pathname !== "/queue") navigate("/queue");
    } else {
      queueVisible = true;
      partyDetailsVisible = false;
    }
  }
  function removeFromQueue(id) {
    const item = queue.find((track) => track.id === id);
    if (
      sendPartyCommand(
        "remove-track",
        { trackId: item ? toPartyTrack(item).id : id },
        "canManageQueue",
      )
    )
      return;
    queue = queue.filter((item) => item.id !== id);
    localStorage.setItem("wavelength-queue", JSON.stringify(queue));
    schedulePartyStatePublish();
  }

  function removePlayedFromQueue(track) {
    if (!track || !queue.some((item) => item.id === track.id)) return false;
    queue = queue.filter((item) => item.id !== track.id);
    removedQueueHistory = [
      ...removedQueueHistory.filter((item) => item.id !== track.id),
      track,
    ].slice(-50);
    localStorage.setItem("wavelength-queue", JSON.stringify(queue));
    schedulePartyStatePublish();
    return true;
  }
  function handleQueueDragStart(event, index) {
    if (!guestCanManageQueue) return;
    draggedQueueIndex = index;
    event.dataTransfer?.setData("text/plain", String(index));
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }
  function handleQueueDragOver(event, index) {
    if (!guestCanManageQueue) return;
    event.preventDefault();
    dragOverQueueIndex = index;
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
  }
  function clearQueuePointerGesture() {
    if (queuePointerHoldTimer) clearTimeout(queuePointerHoldTimer);
    queuePointerHoldTimer = null;
    queuePointerGesture = null;
    window.removeEventListener("pointermove", handleQueuePointerMove);
    window.removeEventListener("pointerup", handleQueuePointerUp);
    window.removeEventListener("pointercancel", handleQueuePointerCancel);
  }
  function handleQueuePointerDown(event, index) {
    if (
      !guestCanManageQueue ||
      (event.pointerType !== "touch" && event.pointerType !== "pen") ||
      event.button !== 0
    )
      return;
    clearQueuePointerGesture();
    const gesture = {
      pointerId: event.pointerId,
      index,
      startX: event.clientX,
      startY: event.clientY,
      active: false,
    };
    queuePointerGesture = gesture;
    queuePointerHoldTimer = window.setTimeout(() => {
      if (queuePointerGesture !== gesture) return;
      gesture.active = true;
      draggedQueueIndex = index;
      dragOverQueueIndex = index;
    }, 400);
    window.addEventListener("pointermove", handleQueuePointerMove, {
      passive: false,
    });
    window.addEventListener("pointerup", handleQueuePointerUp);
    window.addEventListener("pointercancel", handleQueuePointerCancel);
  }
  function handleQueuePointerMove(event) {
    const gesture = queuePointerGesture;
    if (!gesture || event.pointerId !== gesture.pointerId) return;
    if (!gesture.active) {
      if (
        Math.hypot(event.clientX - gesture.startX, event.clientY - gesture.startY) >
        10
      )
        clearQueuePointerGesture();
      return;
    }
    event.preventDefault();
    const row = document
      .elementFromPoint(event.clientX, event.clientY)
      ?.closest(".queue-draggable[data-queue-index]");
    const targetIndex = Number(row?.dataset.queueIndex);
    if (Number.isInteger(targetIndex)) dragOverQueueIndex = targetIndex;
  }
  function handleQueuePointerUp(event) {
    const gesture = queuePointerGesture;
    if (!gesture || event.pointerId !== gesture.pointerId) return;
    if (gesture.active) moveQueueItem(gesture.index, dragOverQueueIndex);
    clearQueuePointerGesture();
    clearQueueDrag();
  }
  function handleQueuePointerCancel(event) {
    if (event.pointerId !== queuePointerGesture?.pointerId) return;
    clearQueuePointerGesture();
    clearQueueDrag();
  }
  function handleQueueDrop(event, index) {
    if (!guestCanManageQueue) return;
    event.preventDefault();
    const data = event.dataTransfer?.getData("text/plain");
    const sourceIndex =
      data === "" || data == null ? draggedQueueIndex : Number(data);
    moveQueueItem(sourceIndex, index);
    clearQueueDrag();
  }
  function moveQueueItem(sourceIndex, targetIndex) {
    if (
      !Number.isInteger(sourceIndex) ||
      sourceIndex < 0 ||
      sourceIndex >= queue.length ||
      sourceIndex === targetIndex
    )
      return;
    if (partyRole === "guest") {
      sendPartyCommand(
        "reorder-queue",
        {
          trackId: toPartyTrack(queue[sourceIndex]).id,
          targetIndex,
        },
        "canManageQueue",
      );
      return;
    }
    const reordered = [...queue];
    const [moved] = reordered.splice(sourceIndex, 1);
    reordered.splice(targetIndex, 0, moved);
    queue = reordered;
    localStorage.setItem("wavelength-queue", JSON.stringify(queue));
    schedulePartyStatePublish();
  }
  function handleQueueReorderKeydown(event, index) {
    if (event.key === "ArrowUp" && index > 0) {
      event.preventDefault();
      moveQueueItem(index, index - 1);
    } else if (event.key === "ArrowDown" && index < queue.length - 1) {
      event.preventDefault();
      moveQueueItem(index, index + 1);
    }
  }
  function clearQueueDrag() {
    draggedQueueIndex = null;
    dragOverQueueIndex = null;
  }
  function handleVolumeButtonClick() {
    if (window.innerWidth <= 735) return;
    if (volume > 0) {
      previousVolume = volume;
      localStorage.setItem(
        "wavelength-previous-volume",
        String(previousVolume),
      );
      volume = 0;
      isMuted = true;
    } else {
      volume = previousVolume || 0.3;
      isMuted = false;
    }
    if (audioElement) audioElement.volume = volume;
    localStorage.setItem("wavelength-volume", String(volume));
  }
  function handleVolumeInput(event) {
    volume = Number(event.currentTarget.value);
    isMuted = volume === 0;
    if (volume > 0) previousVolume = volume;
    if (volume > 0)
      localStorage.setItem(
        "wavelength-previous-volume",
        String(previousVolume),
      );
    if (audioElement) audioElement.volume = volume;
    localStorage.setItem("wavelength-volume", String(volume));
  }
  function scrollRecentlyAdded(direction) {
    if (!recentAddedGrid) return;
    const firstCard = recentAddedGrid.querySelector(".recent-added-card");
    const gap = parseFloat(getComputedStyle(recentAddedGrid).columnGap) || 0;
    const distance = firstCard
      ? firstCard.offsetWidth + gap
      : recentAddedGrid.clientWidth * 0.8;
    const behavior = window.matchMedia("(prefers-reduced-motion: reduce)")
      .matches
      ? "auto"
      : "smooth";
    recentAddedGrid.scrollBy({ left: direction * distance, behavior });
  }
  function startContentDrag(event) {
    if (
      mobileViewport ||
      event.button !== 0 ||
      event.target.closest(
        'button,a,input,textarea,select,[role="button"],.track-row,.queue-draggable',
      )
    )
      return;
    contentDrag = {
      pointerId: event.pointerId,
      startY: event.clientY,
      startScrollTop: contentInner.scrollTop,
      moved: false,
    };
    contentInner.setPointerCapture(event.pointerId);
  }
  function dragContent(event) {
    if (!contentDrag || contentDrag.pointerId !== event.pointerId) return;
    const distance = event.clientY - contentDrag.startY;
    if (!contentDrag.moved && Math.abs(distance) < 4) return;
    contentDrag.moved = true;
    isContentDragging = true;
    event.preventDefault();
    contentInner.scrollTop = contentDrag.startScrollTop - distance;
  }
  function stopContentDrag(event) {
    if (!contentDrag || contentDrag.pointerId !== event.pointerId) return;
    suppressDragClick = contentDrag.moved;
    contentDrag = null;
    isContentDragging = false;
  }
  function stopDraggedClick(event) {
    if (!suppressDragClick) return;
    event.preventDefault();
    event.stopPropagation();
    suppressDragClick = false;
  }
  function recordTrackPlay() {
    if (!pendingHistoryTrack) return;
    const track = pendingHistoryTrack;
    const artwork = getTrackArt(track);
    const playedAt = new Date().toISOString();
    pendingHistoryTrack = null;
    playCounts = { ...playCounts, [track.id]: (playCounts[track.id] || 0) + 1 };
    localStorage.setItem("wavelength-play-counts", JSON.stringify(playCounts));
    playEvents = [
      ...playEvents,
      {
        id: track.id,
        title: track.title,
        artist: track.artist || "Unknown artist",
        playedAt,
      },
    ].slice(-5000);
    localStorage.setItem("wavelength-play-events", JSON.stringify(playEvents));
    playHistory = [
      {
        id: track.id,
        title: track.title,
        artist: track.artist,
        album: track.album,
        source: getTrackSource(track),
        color: track.color,
        art:
          artwork.startsWith("data:") || artwork.startsWith("blob:")
            ? ""
            : artwork,
        playedAt,
      },
      ...playHistory.filter((item) => item.id !== track.id),
    ];
    localStorage.setItem(
      "wavelength-play-history",
      JSON.stringify(playHistory),
    );
  }
  async function prepareTrackAudio(
    track,
    { scope = joinedParty ? "room" : "permanent" } = {},
  ) {
    const trackId = getQueueAudioId(track);
    const query = encodeURIComponent(trackId);
    await api(`/audio/preload?video_ids=${query}&scope=${scope}`, {
      method: "POST",
    });
    const deadline = Date.now() + 10 * 60 * 1000;
    while (Date.now() < deadline) {
      const data = await api(`/audio/status?video_ids=${query}`);
      const status = data.tracks?.[trackId];
      if (status)
        queueDownloadStatuses = { ...queueDownloadStatuses, [trackId]: status };
      if (status?.status === "ready") return;
      if (status?.status === "error" || status?.status === "cancelled") {
        throw new Error(
          status.error ||
            "Audio preparation failed. Check the logs for details.",
        );
      }
      await new Promise((resolve) => window.setTimeout(resolve, 1000));
    }
    throw new Error(
      "Audio did not become ready in time. Check the logs for details.",
    );
  }
  async function playTrack(
    track,
    {
      broadcast = true,
      roomSession = Boolean(joinedParty),
      startTime = 0,
    } = {},
  ) {
    if (!track) return;
    if (
      broadcast &&
      sendPartyCommand(
        "play-track",
        { track: toPartyTrack(track) },
        "canControlPlayback",
      )
    )
      return;
    if (
      isLocalPlayer() &&
      !track.file &&
      !track.serverAudioId &&
      !isQueueDownloadable(track)
    ) {
      notice =
        joinedParty && partyRole === "guest"
          ? "This local-only track is not available on the host’s room stream."
          : "This local-only track is not available on this device.";
      return;
    }
    const requestId = ++playbackRequestId;
    const audioUnlock = resumeEqualizer().then(
      () => null,
      (error) => error,
    );
    playbackRequestPending = true;
    if (isQueueDownloadable(track)) {
      try {
        await prepareTrackAudio(track, {
          scope: roomSession ? "room" : "permanent",
        });
      } catch (error) {
        if (requestId === playbackRequestId) {
          playbackRequestPending = false;
          notice = error.message;
        }
        return;
      }
      if (requestId !== playbackRequestId) return;
    }
    playing = track;
    pendingHistoryTrack = track;
    currentTime = Math.max(0, Number(startTime) || 0);
    pendingPlaybackSeek = currentTime > 0 ? currentTime : null;
    duration = 0;
    activeLyricIndex = -1;
    lyricsFullscreen = false;
    loadLyrics(track);
    isPlaying = true;
    persistLastPlayback(true);
    if (audioUrl?.startsWith("blob:")) URL.revokeObjectURL(audioUrl);
    const trackId = getQueueAudioId(track);
    const streamPath = `/stream/${encodeURIComponent(trackId)}?code=solo&scope=${roomSession ? "room" : "permanent"}`;
    const sharedAudioUrl =
      roomSession && joinedParty && track.serverAudioId
        ? `${roomServerUrl}/api/parties/${encodeURIComponent(joinedParty)}/audio/${encodeURIComponent(track.serverAudioId)}`
        : "";
    audioUrl =
      sharedAudioUrl ||
      (track.file
        ? URL.createObjectURL(track.file)
        : trackId
          ? `${isLocalPlayer() ? await localAudioBase() : "/api/audio"}${streamPath}`
          : "");
    if (audioElement) {
      audioElement.pause();
      audioElement.src = audioUrl;
      if (audioElement.readyState > 0) {
        try {
          audioElement.currentTime = pendingPlaybackSeek ?? 0;
          pendingPlaybackSeek = null;
        } catch {}
      }
      audioElement.volume = volume;
    }
    await tick();
    if (requestId !== playbackRequestId) return;
    if (audioUrl) {
      try {
        const audioUnlockError = await audioUnlock;
        if (audioUnlockError) throw audioUnlockError;
        if (requestId !== playbackRequestId) return;
        await audioElement?.play();
        if (requestId !== playbackRequestId) {
          if (!isPlaying && !playbackRequestPending) audioElement?.pause();
          return;
        }
        playbackRequestPending = false;
      } catch (error) {
        if (requestId === playbackRequestId) {
          playbackRequestPending = false;
          pendingHistoryTrack = null;
          isPlaying = false;
          if (error?.name === "NotAllowedError")
            notice = "Playback was blocked. Press Play again to start audio.";
        }
        return;
      }
    } else {
      pendingHistoryTrack = null;
      isPlaying = true;
      playbackRequestPending = false;
    }
    if (requestId === playbackRequestId && broadcast)
      publishPartyPlaybackControl();
    if (requestId === playbackRequestId && broadcast)
      schedulePartyStatePublish();
  }

  function previousTrack() {
    if (sendPartyCommand("previous-track", {}, "canControlPlayback")) return;
    const restoredTrack =
      removedQueueHistory[removedQueueHistory.length - 1];
    if (restoredTrack) {
      removedQueueHistory = removedQueueHistory.slice(0, -1);
      queue = [
        restoredTrack,
        ...queue.filter((item) => item.id !== restoredTrack.id),
      ];
      localStorage.setItem("wavelength-queue", JSON.stringify(queue));
      schedulePartyStatePublish();
      playTrack(restoredTrack);
      return;
    }
    const source = queueSource();
    if (!source.length) return;
    const currentIndex = source.findIndex((track) => track.id === playing?.id);
    const previous =
      source[currentIndex > 0 ? currentIndex - 1 : source.length - 1] ||
      source[0];
    playTrack(previous);
  }

  function nextTrack() {
    if (sendPartyCommand("next-track", {}, "canControlPlayback")) return;
    const source = queueSource();
    if (!source.length) return;
    const target = resolveNextTrack(1);
    if (target && target.id !== playing?.id) {
      removePlayedFromQueue(playing);
      playTrack(target);
    }
  }

  function handleAudioEnded() {
    if (partyRole === "guest") return;
    if (repeatMode === "one") {
      if (audioElement) {
        const requestId = ++playbackRequestId;
        playbackRequestPending = true;
        audioElement.currentTime = 0;
        resumeEqualizer()
          .then(() =>
            requestId === playbackRequestId ? audioElement.play() : null,
          )
          .then(() => {
            if (requestId !== playbackRequestId) {
              if (!isPlaying && !playbackRequestPending)
                audioElement.pause();
              return;
            }
            playbackRequestPending = false;
            isPlaying = true;
          })
          .catch(() => {
            if (requestId === playbackRequestId) {
              playbackRequestPending = false;
              isPlaying = false;
            }
          });
      }
      schedulePartyStatePublish();
      return;
    }
    removePlayedFromQueue(playing);
    if (repeatMode === "all") nextTrack();
    else {
      isPlaying = false;
      schedulePartyStatePublish();
    }
  }

  function parseSyncedLyrics(value) {
    const timestampPattern = /\[(\d{1,}):(\d{2})(?:[.:](\d{1,3}))?\]/g;
    return String(value || "")
      .split(/\r?\n/)
      .flatMap((rawLine) => {
        const timestamps = [...rawLine.matchAll(timestampPattern)];
        const text = rawLine.replace(timestampPattern, "").trim();
        if (!text) return [];
        if (!timestamps.length) return [{ time: null, text }];
        return timestamps.map((match) => ({
          time:
            Number(match[1]) * 60 +
            Number(match[2]) +
            Number((match[3] || "0").padEnd(3, "0")) / 1000,
          text,
        }));
      })
      .sort((left, right) => (left.time ?? -1) - (right.time ?? -1));
  }
  function updateActiveLyric() {
    if (getTrackSource(playing) === "YouTube") {
      if (activeLyricIndex !== -1) activeLyricIndex = -1;
      return;
    }
    let nextIndex = -1;
    for (let index = 0; index < syncedLyricLines.length; index += 1) {
      const time = syncedLyricLines[index].time;
      if (time !== null && time <= currentTime) nextIndex = index;
      if (time !== null && time > currentTime) break;
    }
    if (activeLyricIndex !== nextIndex) activeLyricIndex = nextIndex;
  }
  function handleAudioTimeUpdate() {
    if (!audioElement) return;
    if (
      pendingPlaybackSeek !== null &&
      Number.isFinite(audioElement.duration)
    ) {
      const targetTime = Math.min(pendingPlaybackSeek, audioElement.duration);
      pendingPlaybackSeek = null;
      try {
        audioElement.currentTime = targetTime;
      } catch {}
    }
    currentTime = Number.isFinite(audioElement.currentTime)
      ? Math.max(0, audioElement.currentTime)
      : 0;
    if (Number.isFinite(audioElement.duration))
      duration = audioElement.duration;
    updateActiveLyric();
    persistLastPlayback();
    schedulePartyProgressPublish();
  }
  function handleAudioPlay() {
    if (partyRole === "guest") {
      savePlayedTrack(playing);
      recordTrackPlay();
      persistLastPlayback(true);
      return;
    }
    if (!isPlaying && !playbackRequestPending) {
      audioElement?.pause();
      return;
    }
    playbackRequestPending = false;
    isPlaying = true;
    savePlayedTrack(playing);
    recordTrackPlay();
    persistLastPlayback(true);
    schedulePartyStatePublish();
  }
  function handleAudioPause() {
    if (partyRole === "guest") {
      persistLastPlayback(true);
      return;
    }
    if (playbackRequestPending || (audioElement && !audioElement.paused)) return;
    isPlaying = false;
    persistLastPlayback(true);
    schedulePartyStatePublish();
  }
  function handleAudioSeeked() {
    handleAudioTimeUpdate();
    persistLastPlayback(true);
    schedulePartyStatePublish();
  }
  async function loadLyrics(track) {
    const requestId = ++lyricRequestId;
    lyrics = "";
    syncedLyricLines = [];
    activeLyricIndex = -1;
    if (!track) return;
    const normalize = (value) =>
      String(value || "")
        .normalize("NFKC")
        .toLowerCase()
        .replace(/\s+/g, " ")
        .trim();
    const localTitle = normalize(track.title);
    const localArtist = normalize(track.artist);
    const youtubeTrack = getTrackSource(track) === "YouTube";
    const onlineMatches = isImportedTrack(track)
      ? onlineTracks
          .filter(
            (candidate) =>
              normalize(candidate.title) === localTitle &&
              candidate.artist &&
              normalize(candidate.artist) !== "local file",
          )
          .sort(
            (left, right) =>
              Number(normalize(right.artist) === localArtist) -
              Number(normalize(left.artist) === localArtist),
          )
      : [];
    const candidates = [...new Set([...onlineMatches, track])];
    let requestFailed = false;
    for (const candidate of candidates) {
      if (!candidate.artist || normalize(candidate.artist) === "local file")
        continue;
      try {
        const lyricsUrl = isLocalPlayer()
          ? `https://lrclib.net/api/get?track_name=${encodeURIComponent(candidate.title)}&artist_name=${encodeURIComponent(candidate.artist)}`
          : `/api/audio/lyrics?title=${encodeURIComponent(candidate.title)}&artist=${encodeURIComponent(candidate.artist)}`;
        const response = await fetch(lyricsUrl);
        if (!response.ok) continue;
        const data = await response.json();
        if (requestId !== lyricRequestId) return;
        const source = youtubeTrack
          ? data.plainLyrics || data.syncedLyrics || ""
          : data.syncedLyrics || data.plainLyrics || "";
        if (!source) continue;
        const parsedLyrics = parseSyncedLyrics(source);
        syncedLyricLines = youtubeTrack
          ? parsedLyrics.map(({ text }) => ({ time: null, text }))
          : parsedLyrics;
        lyrics = source;
        if (!youtubeTrack) updateActiveLyric();
        return;
      } catch {
        requestFailed = true;
      }
    }
    if (requestId !== lyricRequestId) return;
    lyrics = requestFailed
      ? "Lyrics are unavailable for this track."
      : "No lyrics found.";
  }

  function cleanDownloadSegment(value, fallback) {
    let segment = String(value || "")
      .normalize("NFKC")
      .replace(/[\u0000-\u001f<>:"/\\|?*]/g, "")
      .replace(/\s+/g, " ")
      .replace(/[. ]+$/g, "")
      .trim()
      .slice(0, 100);
    if (/^(con|prn|aux|nul|com[1-9]|lpt[1-9])$/i.test(segment))
      segment = `_${segment}`;
    return segment || fallback;
  }

  function getDownloadMetadata(track) {
    const source = getTrackSource(track);
    let title = String(track?.title || "Untitled track").trim();
    let artist = String(
      track?.artist || track?.channel || track?.uploader || "",
    ).trim();
    let album = String(track?.album || track?.album_name || "").trim();

    if (source === "YouTube") {
      title = title
        .replace(
          /\s*[([]\s*(?:official\s*)?(?:music\s+video|video|audio|lyrics?|visuali[sz]er|4k|hd)[^\])]*[)\]]/gi,
          "",
        )
        .replace(
          /\s*[|｜]\s*(?:official\s*)?(?:music\s+video|video|audio|lyrics?)\s*$/i,
          "",
        )
        .replace(/\s+/g, " ")
        .trim();
      artist = artist.replace(/\s+-\s*Topic$/i, "").trim();
      if (!artist || /^(youtube|unknown artist|topic)$/i.test(artist)) {
        const titleParts = title.match(/^(.+?)\s[-–—]\s(.+)$/);
        if (titleParts)
          [, artist, title] = titleParts.map((part) => part.trim());
      } else {
        const artistPrefix = new RegExp(
          `^${artist.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\s*[-–—]\\s*`,
          "i",
        );
        title = title.replace(artistPrefix, "").trim();
      }
      if (!album || /^(youtube|single)$/i.test(album)) album = "Singles";
    }

    return {
      source: cleanDownloadSegment(source, "Unknown source"),
      artist: cleanDownloadSegment(artist, "Unknown artist"),
      album: cleanDownloadSegment(album, "Singles"),
      title: cleanDownloadSegment(title, "Untitled track"),
    };
  }

  function savePlayedTrack(track) {
    if (!isQueueDownloadable(track)) return;
    if (!downloadFolderReady || (!isTauri() && !downloadDirectoryHandle)) {
      if (!downloadFolderPromptShown) {
        notice = "Choose a download folder in Settings to save played songs.";
        downloadFolderPromptShown = true;
      }
      return;
    }
    const trackId = getQueueAudioId(track);
    if (!trackId || playedTrackDownloads.has(trackId)) return;
    const pendingDownload = downloadTrack(track, { automatic: true }).finally(
      () => playedTrackDownloads.delete(trackId),
    );
    playedTrackDownloads.set(trackId, pendingDownload);
  }

  async function downloadTrack(track, { automatic = false } = {}) {
    if (!downloadFolderReady) {
      if (!automatic)
        notice = "Set a download folder before searching or downloading.";
      return;
    }
    try {
      if (track.file) {
        if (!isTauri()) {
          if (!downloadDirectoryHandle) {
            notice = "Set a downloads folder in Settings before downloading.";
            navigate("/settings");
            return;
          }
          const fileHandle = await downloadDirectoryHandle.getFileHandle(
            track.file.name,
            { create: true },
          );
          const writable = await fileHandle.createWritable();
          await writable.write(track.file);
          await writable.close();
          notice = `${track.file.name} saved to ${downloadDirectoryName}.`;
          return;
        }
        const localUrl = URL.createObjectURL(track.file);
        const link = document.createElement("a");
        link.href = localUrl;
        link.download = track.file.name;
        link.click();
        URL.revokeObjectURL(localUrl);
        return;
      }
      const trackId = getQueueAudioId(track);
      if (!trackId) throw new Error("This result has no downloadable track ID");
      if (isTauri()) {
        if (!downloadDirectoryName) {
          notice = "Choose a download folder in Settings first.";
          navigate("/settings");
          return;
        }
        const metadata = getDownloadMetadata(track);
        const path = await invoke("download_audio", {
          trackId,
          filename: `${metadata.artist} - ${metadata.title}`,
        });
        if (!automatic) notice = `Saved to ${path}.`;
        return;
      }
      if (isQueueDownloadable(track)) {
        await api(`/audio/preload?video_ids=${encodeURIComponent(trackId)}`, {
          method: "POST",
        });
        if (!automatic)
          if (!automatic)
            notice = `${track.title || "Track"} queued for download. Progress is available in Logs.`;
      }
      const downloadUrl = `${isTauri() ? await localAudioBase() : "/api/audio"}/download/${encodeURIComponent(trackId)}?code=solo`;
      if (!isTauri()) {
        if (!downloadDirectoryHandle) {
          notice = "Set a downloads folder in Settings before downloading.";
          navigate("/settings");
          return;
        }
        let permission = await downloadDirectoryHandle.queryPermission({
          mode: "readwrite",
        });
        if (permission !== "granted")
          permission = await downloadDirectoryHandle.requestPermission({
            mode: "readwrite",
          });
        if (permission !== "granted")
          throw new Error("Downloads folder permission was not granted.");
        const response = await fetch(downloadUrl);
        if (!response.ok)
          throw new Error(`Download failed (${response.status}).`);
        const contentDisposition =
          response.headers.get("content-disposition") || "";
        const filenameMatch = contentDisposition.match(
          /filename\*?=(?:UTF-8''|\")?([^;\"]+)/i,
        );
        const responseFilename = filenameMatch?.[1]
          ? decodeURIComponent(filenameMatch[1].trim())
          : `${trackId}.audio`;
        const extension =
          responseFilename.match(/\.[a-z0-9]{2,5}$/i)?.[0] || ".audio";
        const metadata = getDownloadMetadata(track);
        let targetDirectory = downloadDirectoryHandle;
        for (const segment of [
          metadata.source,
          metadata.artist,
          metadata.album,
        ]) {
          targetDirectory = await targetDirectory.getDirectoryHandle(segment, {
            create: true,
          });
        }
        const filename = `${metadata.artist} - ${metadata.title}${extension}`;
        const fileHandle = await targetDirectory.getFileHandle(filename, {
          create: true,
        });
        const writable = await fileHandle.createWritable();
        if (response.body) await response.body.pipeTo(writable);
        else {
          await writable.write(await response.blob());
          await writable.close();
        }
        if (!automatic)
          notice = `${filename} saved to ${metadata.source}/${metadata.artist}/${metadata.album}.`;
        return;
      }
      const link = document.createElement("a");
      link.href = downloadUrl;
      link.download = "";
      document.body.append(link);
      link.click();
      link.remove();
    } catch (error) {
      notice = error.message;
    }
  }
  function persistPartySession(code, role, name) {
    localStorage.setItem("wavelength-party-code", code);
    localStorage.setItem("wavelength-party-role", role);
    localStorage.setItem("wavelength-party-name", name);
  }

  function saveQueueBeforeParty() {
    const snapshotKey = "wavelength-pre-party-queue";
    if (localStorage.getItem(snapshotKey) !== null) return;
    localStorage.setItem(
      snapshotKey,
      localStorage.getItem("wavelength-queue") || JSON.stringify(queue),
    );
  }

  function restoreQueueAfterParty() {
    if (partyRole !== "guest") return;
    const snapshotKey = "wavelength-pre-party-queue";
    const snapshot = localStorage.getItem(snapshotKey);
    if (snapshot === null) return;
    try {
      const restoredQueue = JSON.parse(snapshot);
      queue = Array.isArray(restoredQueue) ? restoredQueue : [];
      localStorage.setItem("wavelength-queue", JSON.stringify(queue));
    } catch {
      localStorage.removeItem("wavelength-queue");
      queue = [];
    }
    localStorage.removeItem(snapshotKey);
  }

  function finishGuestParty(message) {
    if (partyRole !== "guest") return;
    playbackRequestId += 1;
    playbackRequestPending = false;
    pendingHistoryTrack = null;
    audioElement?.pause();
    if (audioUrl?.startsWith("blob:")) URL.revokeObjectURL(audioUrl);
    audioElement?.removeAttribute("src");
    audioElement?.load();
    audioUrl = "";
    playing = null;
    isPlaying = false;
    currentTime = 0;
    duration = 0;
    clearPartySession();
    partyDetailsVisible = false;
    queueVisible = true;
    notice = message;
  }

  function disconnectPartySocket() {
    if (partyProgressTimer) clearTimeout(partyProgressTimer);
    partyProgressTimer = null;
    partyProgressLastSentAt = 0;
    if (partySocket) {
      partySocket.removeAllListeners();
      partySocket.disconnect();
      partySocket = null;
    }
    pendingPartySocketState = null;
  }

  async function handlePartySocketState(party) {
    if (String(party?.code || "").trim().toUpperCase() !== joinedParty) return;
    const revision = Number(party.syncRevision) || 0;
    partyMembers = Array.isArray(party.members) ? party.members : [];
    partyHostName = party.host || party.members?.[0] || "";
    sharedPlaylists = Array.isArray(party.playlists) ? party.playlists : [];
    partyPermissions = {
      canAdd: false,
      canManageQueue: false,
      canControlPlayback: false,
      ...(party.permissions || {}),
    };
    if (!partyMembers.includes(partyDisplayName)) {
      finishGuestParty("You were removed from the room.");
      return;
    }
    if (partyRole === "host") return;
    if (revision <= lastAppliedRoomRevision) return;
    replaceQueueFromParty(party);
    if (partyPlaybackSyncInFlight) {
      if (
        !pendingPartySocketState ||
        revision >= (Number(pendingPartySocketState.syncRevision) || 0)
      ) {
        pendingPartySocketState = party;
      }
      return;
    }
    lastAppliedRoomRevision = revision;
    await applyPartyPlayback(party);
    const pending = pendingPartySocketState;
    pendingPartySocketState = null;
    if (pending) await handlePartySocketState(pending);
  }

  function connectPartySocket() {
    disconnectPartySocket();
    if (!joinedParty || !roomServerUrl || !partyDisplayName) return;
    const roomCode = joinedParty;
    partySocket = createSocket(roomServerUrl, {
      auth: { code: roomCode, username: partyDisplayName },
      transports: ["websocket", "polling"],
      reconnection: true,
    });
    partySocket.on("room-state", (party) => {
      if (joinedParty === roomCode) void handlePartySocketState(party);
    });
    partySocket.on("room-command", (envelope) => {
      if (joinedParty === roomCode) handlePartyCommand(envelope);
    });
    partySocket.on("playback-control", (control) => {
      if (
        joinedParty !== roomCode ||
        partyRole !== "guest" ||
        !playing ||
        toPartyTrack(playing).id !== control?.currentTrackId
      )
        return;
      const position = Number(control.currentTime);
      if (!Number.isFinite(position) || position < 0) return;
      currentTime = position;
      isPlaying = Boolean(control.isPlaying);
      updateActiveLyric();
      if (!audioElement) return;
      if (!isPlaying) {
        playbackRequestId += 1;
        playbackRequestPending = false;
        audioElement.pause();
        return;
      }
      if (Math.abs(audioElement.currentTime - position) > 0.35) {
        try {
          audioElement.currentTime = position;
        } catch {}
      }
      if (audioElement.paused && !playbackRequestPending) {
        const requestId = ++playbackRequestId;
        playbackRequestPending = true;
        void resumeEqualizer()
          .then(() =>
            requestId === playbackRequestId ? audioElement.play() : null,
          )
          .then(() => {
            if (requestId === playbackRequestId) {
              playbackRequestPending = false;
              isPlaying = true;
            }
          })
          .catch(() => {
            if (requestId === playbackRequestId)
              playbackRequestPending = false;
          });
      }
    });
    partySocket.on("playback-progress", (progress) => {
      if (
        joinedParty !== roomCode ||
        partyRole !== "guest" ||
        !playing ||
        toPartyTrack(playing).id !== progress?.currentTrackId
      )
        return;
      const position = Number(progress.currentTime);
      const nextDuration = Number(progress.duration);
      if (!Number.isFinite(position) || position < 0) return;
      currentTime = position;
      if (Number.isFinite(nextDuration) && nextDuration >= 0)
        duration = nextDuration;
      updateActiveLyric();
      if (
        audioElement &&
        Number.isFinite(audioElement.currentTime) &&
        Math.abs(audioElement.currentTime - position) > 1.5
      ) {
        try {
          audioElement.currentTime = position;
        } catch {}
      }
    });
    partySocket.on("room-closed", () => {
      if (joinedParty !== roomCode) return;
      finishGuestParty("Room closed by host.");
    });
    partySocket.on("disconnect", () => {
      if (joinedParty === roomCode && partyRole === "guest")
        void refreshPartyMembers();
    });
  }

  function clearPartySession() {
    restoreQueueAfterParty();
    disconnectPartySocket();
    localStorage.removeItem("wavelength-party-code");
    localStorage.removeItem("wavelength-party-role");
    localStorage.removeItem("wavelength-party-name");
    localStorage.removeItem("wavelength-party-host-token");
    joinedParty = "";
    partyHostToken = "";
    partyCode = "";
    partyRole = "";
    partyDisplayName = "";
    partyMembers = [];
    partyHostName = "";
    sharedPlaylists = [];
    roomSyncRevision = 0;
    lastAppliedRoomRevision = -1;
  }

  async function restorePartySession() {
    if (!joinedParty) return;
    if (!roomServerUrl) {
      roomSessionReady = true;
      return;
    }
    try {
      const party = await api(`/parties/${encodeURIComponent(joinedParty)}`);
      roomSyncRevision = Math.max(0, Number(party.syncRevision) || 0);
      lastAppliedRoomRevision = -1;
      const name = partyDisplayName || username;
      if (!name || !party.members?.includes(name))
        throw new Error("Room membership has expired.");
      if (party.host === name && !party.hostAuthorized) {
        clearPartySession();
        notice =
          "This room needs to be recreated before host controls can be used.";
        return;
      }
      partyDisplayName = name;
      partyRole = party.host === name ? "host" : "guest";
      partyMembers = Array.isArray(party.members) ? party.members : [];
      partyHostName = party.host || party.members?.[0] || "";
      partyPermissions = {
        canAdd: false,
        canManageQueue: false,
        canControlPlayback: false,
        ...(party.permissions || {}),
      };
      sharedPlaylists = Array.isArray(party.playlists) ? party.playlists : [];
      persistPartySession(party.code, partyRole, name);
      connectPartySocket();
      if (partyRole === "host") {
        await applyPartyPlayback(party, {
          allowHost: true,
          forcePlayback: true,
        });
        await uploadQueuedLocalTracks();
        if (!party.currentTrackId && queue.length)
          void playTrack(queue[0], { roomSession: true });
        else schedulePartyStatePublish();
      } else await applyPartyPlayback(party, { forcePlayback: true });
    } catch {
      clearPartySession();
    } finally {
      roomSessionReady = true;
    }
  }

  async function joinPartyAs(name, code = pendingInviteCode || partyCode) {
    if (!roomServerUrl) {
      notice = "Set a room server URL in Settings to join a room.";
      return false;
    }
    const cleanName = String(name || "").trim();
    const cleanCode = String(code || "")
      .trim()
      .toUpperCase();
    if (!cleanName) {
      notice = "Enter your name or connect Spotify to join.";
      return false;
    }
    if (!cleanCode) {
      notice = "Enter a room code to join.";
      return false;
    }
    if (joinedParty && joinedParty !== cleanCode) {
      notice = "Close your current room before joining another one.";
      return false;
    }
    try {
      const result = await api("/parties/join", {
        method: "POST",
        body: JSON.stringify({ code: cleanCode, username: cleanName }),
      });
      saveQueueBeforeParty();
      let roomState = result.party || {};
      joinedParty = result.code;
      partyCode = result.code;
      partyRole = "guest";
      partyDisplayName = cleanName;
      try {
        roomState = await api(`/parties/${encodeURIComponent(result.code)}`);
      } catch {}
      roomSyncRevision = Math.max(0, Number(roomState.syncRevision) || 0);
      lastAppliedRoomRevision = Number(roomState.syncRevision) || 0;
      partyMembers = roomState.members || [cleanName];
      partyHostName = roomState.host || roomState.members?.[0] || "";
      partyPermissions = {
        canAdd: false,
        canManageQueue: false,
        canControlPlayback: false,
        ...(roomState.permissions || {}),
      };
      sharedPlaylists = Array.isArray(roomState.playlists)
        ? roomState.playlists
        : [];
      username = cleanName;
      roomSessionReady = true;
      localStorage.setItem("wavelength-username", cleanName);
      persistPartySession(result.code, "guest", cleanName);
      localStorage.removeItem("wavelength-pending-invite");
      pendingInviteCode = "";
      connectPartySocket();
      await applyPartyPlayback(roomState, { forcePlayback: true });
      notice = `Joined room ${joinedParty}`;
      return true;
    } catch (error) {
      notice = error.message;
      return false;
    }
  }

  async function startParty() {
    if (joinedParty) {
      notice = "Leave your current room before joining another one.";
      return;
    }
    const cleanName = username.trim();
    const code = partyCode.trim().toUpperCase();
    if (!cleanName) {
      notice = "Enter your name to join a room.";
      return;
    }
    if (!code) {
      notice = "Enter a room code to join.";
      return;
    }
    await joinPartyAs(cleanName, code);
  }
  async function createRoom() {
    if (!roomServerUrl) {
      notice = "Set a room server URL in Settings to create a room.";
      return;
    }
    if (joinedParty) {
      notice = "End your current room before creating or joining another one.";
      return;
    }
    const spotifyName =
      spotifyProfile?.display_name || spotifyProfile?.id || "";
    const host = (
      roomNameMode === "spotify" && spotifyConnected ? spotifyName : username
    ).trim();
    if (!host) {
      notice = "Enter your name to create a room.";
      return;
    }
    try {
      const result = await api("/parties", {
        method: "POST",
        body: JSON.stringify({ host }),
      });
      joinedParty = result.code;
      partyHostToken = result.hostToken || "";
      if (partyHostToken)
        localStorage.setItem("wavelength-party-host-token", partyHostToken);
      partyCode = result.code;
      partyRole = "host";
      partyDisplayName = host;
      partyMembers = result.party?.members || [host];
      partyHostName = result.party?.host || host;
      partyPermissions = {
        canAdd: false,
        canManageQueue: false,
        canControlPlayback: false,
        ...(result.party?.permissions || {}),
      };
      localStorage.setItem("wavelength-username", host);
      persistPartySession(result.code, "host", host);
      connectPartySocket();
      notice = `Room ${joinedParty} is ready`;
      persistLastPlayback(true);
      audioElement?.pause();
      if (audioUrl?.startsWith("blob:")) URL.revokeObjectURL(audioUrl);
      audioElement?.removeAttribute("src");
      audioUrl = "";
      playing = null;
      isPlaying = false;
      currentTime = 0;
      duration = 0;
      await uploadQueuedLocalTracks();
      schedulePartyStatePublish();
      if (queue.length) void playTrack(queue[0], { roomSession: true });
    } catch (error) {
      notice = error.message;
    }
  }
  async function closeParty() {
    if (!joinedParty || partyRole !== "host") return;
    if (!roomServerUrl) {
      notice = "Set the room server URL in Settings to manage this room.";
      return;
    }
    try {
      await api(`/parties/${encodeURIComponent(joinedParty)}`, {
        method: "DELETE",
        body: JSON.stringify({ username: partyDisplayName }),
      });
      clearPartySession();
      partyDetailsVisible = false;
      queueVisible = true;
      notice = "Room closed.";
    } catch (error) {
      notice = error.message;
    }
  }
  async function leaveParty() {
    if (!joinedParty || partyRole !== "guest") return;
    const code = joinedParty;
    try {
      await api(`/parties/${encodeURIComponent(code)}/leave`, {
        method: "POST",
        body: JSON.stringify({ username: partyDisplayName }),
      });
      audioElement?.pause();
      if (audioUrl?.startsWith("blob:")) URL.revokeObjectURL(audioUrl);
      audioElement?.removeAttribute("src");
      audioUrl = "";
      playing = null;
      isPlaying = false;
      currentTime = 0;
      duration = 0;
      clearPartySession();
      partyDetailsVisible = false;
      queueVisible = true;
      notice = "You left the room.";
    } catch (error) {
      if (error.message === "Room not found") {
        finishGuestParty("The room was already closed.");
        return;
      }
      notice = error.message;
    }
  }
  async function savePartyPermissions() {
    if (!joinedParty || partyRole !== "host") return;
    if (!roomServerUrl) {
      notice = "Set the room server URL in Settings to manage this room.";
      return;
    }
    try {
      const party = await api(`/parties/${encodeURIComponent(joinedParty)}`, {
        method: "PUT",
        body: JSON.stringify({
          username: partyDisplayName,
          permissions: partyPermissions,
        }),
      });
      partyPermissions = {
        canAdd: false,
        canManageQueue: false,
        canControlPlayback: false,
        ...(party.permissions || {}),
      };
    } catch (error) {
      notice = error.message;
      refreshPartyMembers();
    }
  }
  async function copyPartyInvite() {
    if (!joinedParty) return;
    const invite = new URL("/player", window.location.origin);
    invite.searchParams.set("party", joinedParty);
    try {
      await navigator.clipboard.writeText(invite.toString());
      inviteCopied = true;
      notice = "Invite link copied.";
      if (inviteCopyTimer) clearTimeout(inviteCopyTimer);
      inviteCopyTimer = window.setTimeout(() => {
        inviteCopied = false;
        codeCopied = false;
      }, 1800);
    } catch {
      notice = `Unable to copy invite. Room code: ${joinedParty}`;
    }
  }
  async function copyPartyCode() {
    if (!joinedParty) return;
    try {
      await navigator.clipboard.writeText(joinedParty);
      codeCopied = true;
      notice = "Room code copied.";
      if (inviteCopyTimer) clearTimeout(inviteCopyTimer);
      inviteCopyTimer = window.setTimeout(() => {
        inviteCopied = false;
        codeCopied = false;
      }, 1800);
    } catch {
      notice = `Unable to copy room code: ${joinedParty}`;
    }
  }
  async function refreshPartyMembers() {
    if (!joinedParty || !roomServerUrl || partyRefreshInFlight) return;
    partyRefreshInFlight = true;
    try {
      const party = await api(`/parties/${encodeURIComponent(joinedParty)}`);
      partyMembers = Array.isArray(party.members) ? party.members : [];
      partyHostName = party.host || party.members?.[0] || "";
      sharedPlaylists = Array.isArray(party.playlists) ? party.playlists : [];
      if (partyRole === "host")
        partyPermissions = {
          canAdd: false,
          canManageQueue: false,
          canControlPlayback: false,
          ...(party.permissions || {}),
        };
      else {
        const revision = Number(party.syncRevision) || 0;
        if (revision > lastAppliedRoomRevision) {
          lastAppliedRoomRevision = revision;
          await applyPartyPlayback(party);
        }
      }
    } catch (error) {
      if (partyRole === "guest" && error.message === "Room not found")
        finishGuestParty("Room closed by host.");
    } finally {
      partyRefreshInFlight = false;
    }
  }
  function replaceQueueFromParty(party) {
    const sharedQueue = Array.isArray(party.queue) ? party.queue : [];
    const nextQueue = sharedQueue
      .map((item) => {
        const localTrack = tracks.find((candidate) => candidate.id === item.id);
        return localTrack
          ? {
              ...localTrack,
              serverAudioId: item.audioId || localTrack.serverAudioId,
            }
          : item.audioId
            ? { ...item, serverAudioId: item.audioId, source: "local" }
            : item;
      })
      .filter((item) => item?.id);
    const currentIds = JSON.stringify(
      queue.map((item) => [item.id, item.serverAudioId || ""]),
    );
    const nextIds = JSON.stringify(
      nextQueue.map((item) => [item.id, item.serverAudioId || ""]),
    );
    if (currentIds !== nextIds) {
      queue = nextQueue;
      localStorage.setItem("wavelength-queue", JSON.stringify(queue));
    }
    return nextQueue;
  }

  function resolveRoomCommandTrack(sharedTrack) {
    const localTrack = [...tracks, ...importedTracks, ...favorites].find(
      (candidate) =>
        candidate.id === sharedTrack.id ||
        getQueueAudioId(candidate) === sharedTrack.id,
    );
    if (localTrack) {
      return {
        ...localTrack,
        serverAudioId: sharedTrack.audioId || localTrack.serverAudioId,
      };
    }
    return {
      ...sharedTrack,
      id: sharedTrack.id,
      videoId: sharedTrack.videoId || sharedTrack.id.replace(/^yt-/, ""),
      source: sharedTrack.source || (sharedTrack.id.startsWith("yt-") ? "youtube" : "spotify"),
      serverAudioId: sharedTrack.audioId || "",
    };
  }

  function handlePartyCommand(envelope) {
    if (partyRole !== "host" || !partyMembers.includes(envelope?.username))
      return;
    const command = envelope.command;
    if (!command) return;
    switch (command.action) {
      case "add-track": {
        if (!partyPermissions.canAdd || !command.track) return;
        const track = resolveRoomCommandTrack(command.track);
        if (
          queue.some((item) => item.id === track.id) ||
          playing?.id === track.id
        )
          return;
        queue = [...queue, track];
        localStorage.setItem("wavelength-queue", JSON.stringify(queue));
        preloadQueueTrack(track, { scope: "room" });
        schedulePartyStatePublish();
        break;
      }
      case "remove-track":
        if (!partyPermissions.canManageQueue) return;
        queue = queue.filter(
          (item) => toPartyTrack(item).id !== command.trackId,
        );
        localStorage.setItem("wavelength-queue", JSON.stringify(queue));
        schedulePartyStatePublish();
        break;
      case "clear-queue":
        if (!partyPermissions.canManageQueue) return;
        queue = [];
        localStorage.setItem("wavelength-queue", "[]");
        schedulePartyStatePublish();
        break;
      case "reorder-queue": {
        if (!partyPermissions.canManageQueue) return;
        const sourceIndex = queue.findIndex(
          (item) => toPartyTrack(item).id === command.trackId,
        );
        const displayedQueue = [
          ...new Map(
            [...queue, playing].filter(Boolean).map((item) => {
              const sharedTrack = toPartyTrack(item);
              return [sharedTrack.id, item];
            }),
          ).values(),
        ];
        const visibleTargetIndex = Number(command.targetIndex);
        const targetIndex = displayedQueue
          .slice(0, visibleTargetIndex)
          .filter((item) =>
            queue.some((queued) => toPartyTrack(queued).id === toPartyTrack(item).id),
          ).length;
        if (
          sourceIndex < 0 ||
          !Number.isInteger(visibleTargetIndex) ||
          visibleTargetIndex < 0 ||
          visibleTargetIndex >= displayedQueue.length ||
          targetIndex < 0 ||
          targetIndex > queue.length ||
          sourceIndex === targetIndex
        )
          return;
        const reordered = [...queue];
        const [moved] = reordered.splice(sourceIndex, 1);
        reordered.splice(targetIndex, 0, moved);
        queue = reordered;
        localStorage.setItem("wavelength-queue", JSON.stringify(queue));
        schedulePartyStatePublish();
        break;
      }
      case "play-track":
        if (!partyPermissions.canControlPlayback || !command.track) return;
        void playTrack(resolveRoomCommandTrack(command.track), {
          roomSession: true,
        });
        break;
      case "toggle-playback":
        if (partyPermissions.canControlPlayback) togglePlayback();
        break;
      case "previous-track":
        if (partyPermissions.canControlPlayback) previousTrack();
        break;
      case "next-track":
        if (partyPermissions.canControlPlayback) nextTrack();
        break;
      case "toggle-shuffle":
        if (partyPermissions.canControlPlayback) toggleShuffle();
        break;
      case "toggle-repeat":
        if (partyPermissions.canControlPlayback) toggleRepeat();
        break;
      case "seek":
        if (
          !partyPermissions.canControlPlayback ||
          !Number.isFinite(Number(command.currentTime)) ||
          !audioElement
        )
          return;
        audioElement.currentTime = Math.max(0, Number(command.currentTime));
        handleAudioTimeUpdate();
        schedulePartyStatePublish();
        break;
      default:
        break;
    }
  }
  async function publishPartyState() {
    if (!joinedParty || !roomServerUrl || partyRole !== "host") return;
    if (partyPublishInFlight) {
      partyPublishPending = true;
      return;
    }
    partyPublishInFlight = true;
    const roomCode = joinedParty;
    try {
      do {
        partyPublishPending = false;
        if (joinedParty !== roomCode || partyRole !== "host") break;
        const sharedQueue = [
          ...new Map(
            [...queue, playing].filter(Boolean).map((track) => {
              const sharedTrack = toPartyTrack(track);
              return [sharedTrack.id, sharedTrack];
            }),
          ).values(),
        ];
        const sharedPlaying = playing ? toPartyTrack(playing) : null;
        const sharedPlayingIsLocal = Boolean(
          sharedPlaying?.id.startsWith("local-") && !sharedPlaying.audioId,
        );
        const timestamp = Date.now();
        const playbackTime = Number.isFinite(currentTime)
          ? Math.max(0, currentTime)
          : 0;
        const revision = ++roomSyncRevision;
        try {
          const updated = await api(`/parties/${encodeURIComponent(roomCode)}`, {
            method: "PUT",
            body: JSON.stringify({
              username: partyDisplayName,
              queue: sharedQueue,
              playlists: playlistReferencesForParty(),
              currentTrackId: sharedPlayingIsLocal
                ? ""
                : sharedPlaying?.id || "",
              isPlaying: Boolean(playing && !sharedPlayingIsLocal && isPlaying),
              startEpoch: Math.round(timestamp - playbackTime * 1000),
              pausedAt: playbackTime,
              repeatMode,
              shuffle: shuffleOn,
              syncRevision: revision,
            }),
          });
          if (joinedParty === roomCode) {
            roomSyncRevision = Math.max(
              roomSyncRevision,
              Number(updated.syncRevision) || revision,
            );
          }
        } catch (error) {
          if (joinedParty === roomCode)
            notice = `Room sync failed: ${error.message}`;
          break;
        }
      } while (partyPublishPending);
    } finally {
      partyPublishInFlight = false;
      if (partyPublishPending && joinedParty && partyRole === "host") {
        partyPublishPending = false;
        schedulePartyStatePublish();
      }
    }
  }
  async function applyPartyPlayback(
    party,
    { allowHost = false, forcePlayback = false } = {},
  ) {
    if ((partyRole === "host" && !allowHost) || partyPlaybackSyncInFlight)
      return;
    partyPlaybackSyncInFlight = true;
    try {
      if (forcePlayback) {
        playbackRequestId += 1;
        playbackRequestPending = false;
        audioElement?.pause();
        isPlaying = false;
        if (audioUrl?.startsWith("blob:")) URL.revokeObjectURL(audioUrl);
        audioUrl = "";
        audioElement?.removeAttribute("src");
      }
      const currentTrackId = party.currentTrackId || "";
      const sharedQueue = Array.isArray(party.queue) ? party.queue : [];
      const sharedTrack = sharedQueue.find(
        (item) => item.id === currentTrackId,
      );
      const localTrack = tracks.find((item) => item.id === currentTrackId);
      const track = localTrack
        ? {
            ...localTrack,
            serverAudioId: sharedTrack?.audioId || localTrack.serverAudioId,
          }
        : sharedTrack?.audioId
          ? {
              ...sharedTrack,
              serverAudioId: sharedTrack.audioId,
              source: "local",
            }
          : isQueueDownloadable(sharedTrack)
            ? sharedTrack
            : null;
      const nextQueue = replaceQueueFromParty(party);
      const tracksToPreload = nextQueue.filter((item) => {
        if (!isQueueDownloadable(item)) return false;
        const status = queueDownloadStatuses[getQueueAudioId(item)]?.status;
        return !status || status === "pending";
      });
      tracksToPreload.forEach((item) => {
        void preloadQueueTrack(item, { scope: "room" });
      });
      if (["off", "all", "one"].includes(party.repeatMode))
        repeatMode = party.repeatMode;
      shuffleOn = Boolean(party.shuffle);
      if (!track) {
        playbackRequestId += 1;
        playbackRequestPending = false;
        audioElement?.pause();
        isPlaying = false;
        pendingHistoryTrack = null;
        if (audioUrl?.startsWith("blob:")) URL.revokeObjectURL(audioUrl);
        audioUrl = "";
        audioElement?.removeAttribute("src");
        playing = null;
        currentTime = 0;
        duration = 0;
        return;
      }
      const targetTime = party.isPlaying
        ? Math.max(
            0,
            (Date.now() - Number(party.startEpoch || Date.now())) / 1000,
          )
        : Math.max(0, Number(party.pausedAt) || 0);
      if (forcePlayback || playing?.id !== track.id) {
        audioElement?.pause();
        playing = track;
        currentTime = targetTime;
        duration = Math.max(0, Number(track.duration) || 0);
        await playTrack(track, {
          broadcast: false,
          roomSession: true,
          startTime: targetTime,
        });
      }
      if (
        audioElement &&
        Number.isFinite(targetTime) &&
        Math.abs(audioElement.currentTime - targetTime) > 1.25
      ) {
        try {
          audioElement.currentTime = targetTime;
        } catch {}
      }
      if (
        party.isPlaying &&
        audioElement &&
        !isPlaying &&
        !playbackRequestPending
      ) {
        const requestId = ++playbackRequestId;
        playbackRequestPending = true;
        isPlaying = true;
        try {
          await resumeEqualizer();
          if (requestId !== playbackRequestId) return;
          await audioElement.play();
          if (requestId !== playbackRequestId) {
            audioElement.pause();
            return;
          }
          playbackRequestPending = false;
          isPlaying = true;
        } catch {
          if (requestId === playbackRequestId) {
            playbackRequestPending = false;
            isPlaying = false;
          }
        }
      } else if (
        !party.isPlaying &&
        audioElement &&
        (isPlaying || playbackRequestPending)
      ) {
        playbackRequestId += 1;
        playbackRequestPending = false;
        audioElement.pause();
        isPlaying = false;
      }
    } finally {
      partyPlaybackSyncInFlight = false;
      const pending = pendingPartySocketState;
      pendingPartySocketState = null;
      if (pending) void handlePartySocketState(pending).catch(() => {});
    }
  }
  function togglePlayback() {
    if (sendPartyCommand("toggle-playback", {}, "canControlPlayback")) return;
    if (!playing) return;
    if (isPlaying || playbackRequestPending) {
      playbackRequestId += 1;
      playbackRequestPending = false;
      audioElement?.pause();
      isPlaying = false;
      publishPartyPlaybackControl();
      schedulePartyStatePublish();
      return;
    }
    if (!audioElement || !audioUrl) {
      if (audioElement && playing) {
        void playTrack(playing, {
          startTime: currentTime || playing.resumeAt || 0,
        });
        return;
      }
      isPlaying = true;
      publishPartyPlaybackControl();
      schedulePartyStatePublish();
      return;
    }
    const requestId = ++playbackRequestId;
    playbackRequestPending = true;
    isPlaying = true;
    void resumeEqualizer().catch(() => {});
    audioElement
      .play()
      .then(() => {
        if (requestId !== playbackRequestId) {
          if (!isPlaying && !playbackRequestPending) audioElement.pause();
          return;
        }
        playbackRequestPending = false;
        isPlaying = true;
        publishPartyPlaybackControl();
        schedulePartyStatePublish();
      })
      .catch((error) => {
        if (requestId !== playbackRequestId) return;
        playbackRequestPending = false;
        isPlaying = false;
        notice =
          error?.name === "NotAllowedError"
            ? "Playback was blocked. Press Play again to start audio."
            : "Unable to start playback. Try pressing Play again.";
      });
  }
  function seek(event) {
    if (!audioElement || !Number.isFinite(duration) || duration <= 0) return;
    const requestedTime = Number(event.currentTarget.value);
    if (!Number.isFinite(requestedTime)) return;
    const targetTime = Math.min(duration, Math.max(0, requestedTime));
    if (
      sendPartyCommand(
        "seek",
        { currentTime: targetTime },
        "canControlPlayback",
      )
    )
      return;
    if (!audioUrl) {
      currentTime = targetTime;
      playing = { ...playing, resumeAt: targetTime };
      persistLastPlayback(true);
      return;
    }
    audioElement.currentTime = targetTime;
    handleAudioTimeUpdate();
    schedulePartyStatePublish();
  }
  function openLyricsFullscreen() {
    if (lyricsFullscreen) return;
    window.history.pushState(
      { lyricsFullscreen: true },
      "",
      window.location.href,
    );
    lyricsFullscreen = true;
    tick().then(() => document.querySelector(".lyrics-close")?.focus());
  }
  function closeLyricsFullscreen() {
    if (!lyricsFullscreen) return;
    lyricsFullscreen = false;
    if (window.history.state?.lyricsFullscreen) window.history.back();
  }
  function toggleLyricsFullscreen() {
    if (lyricsFullscreen) closeLyricsFullscreen();
    else openLyricsFullscreen();
  }
  function handleLyricsLinesKeydown(event) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      openLyricsFullscreen();
    }
  }
  function startRailResize(event) {
    resizingRail = true;
    event.preventDefault();
  }
  function resizeRail(event) {
    if (resizingRail)
      railWidth = Math.min(
        480,
        Math.max(300, window.innerWidth - event.clientX),
      );
  }
  function stopRailResize() {
    resizingRail = false;
  }
  function openNowPlaying() {
    if (mobileViewport && playing) navigate("/now-playing");
  }
  function toggleSidebar() {
    if (mobileViewport) mobileNav = !mobileNav;
    else sidebarCollapsed = !sidebarCollapsed;
  }
  function handleWindowPointerMove(event) {
    resizeRail(event);
  }
  function handlePlayerClick(event) {
    if (!mobileViewport || !event.target.closest?.(".player-bar")) return;
    if (event.target.closest(".player-center, .player-like-button")) return;
    openNowPlaying();
  }
  function handleWindowClick(event) {
    stopDraggedClick(event);
    handlePlayerClick(event);
    if (!event.target.closest(".topbar-search-control"))
      topbarSearchOpen = false;
    if (volumeSliderOpen && !event.target.closest(".volume-control"))
      volumeSliderOpen = false;
  }
  function switchRail(view) {
    const nextView = railView === view ? null : view;
    queueVisible = nextView === "queue" || nextView === "party";
    partyDetailsVisible = nextView === "party";
  }
  function openPartySettings() {
    if (!roomServerUrl) {
      notice = "Set the server URL in Settings before opening room settings.";
      return;
    }
    switchRail("party");
  }
  function logout() {
    localStorage.removeItem("wavelength-username");
    username = "";
    notice = "Logged out";
  }
</script>

<svelte:window
  on:pointermove={handleWindowPointerMove}
  on:pointerup={stopRailResize}
  on:click={handleWindowClick}
/>

<div
  class:queue-closed={!railView}
  class:sidebar-collapsed={sidebarCollapsed}
  class:now-playing-active={activeView === "now-playing"}
  class="app-shell"
  inert={inviteGateRequired}
  aria-hidden={inviteGateRequired}
  style={`--rail-width:${railWidth}px;--content-modal-left:${contentModalBounds.left}px;--content-modal-top:${contentModalBounds.top}px;--content-modal-width:${contentModalBounds.width}px;--content-modal-height:${contentModalBounds.height}px`}
>
  <audio
    bind:this={audioElement}
    src={audioUrl || undefined}
    crossorigin="anonymous"
    preload="metadata"
    on:timeupdate={handleAudioTimeUpdate}
    on:loadedmetadata={handleAudioTimeUpdate}
    on:durationchange={handleAudioTimeUpdate}
    on:seeked={handleAudioSeeked}
    on:ended={handleAudioEnded}
    on:play={handleAudioPlay}
    on:pause={handleAudioPause}
  ></audio>
  <div class="workspace-cluster">
    <aside
      class:open={mobileNav}
      class="sidebar"
    >
      <nav>
        {#each views as view}
          <button
            class:active={activeView === view.id}
            on:click={() => navigate(view.path)}
            ><svelte:component this={view.icon} size={17} /><span
              >{view.label}</span
            ></button
          >
        {/each}
        {#if !mobileViewport}<button
            class:active={activeView === "playlists"}
            on:click={() => navigate("/playlists")}
            ><ListMusic size={17} /><span>Playlists</span></button
          >{/if}
        <button
          class:active={activeView === "songs"}
          on:click={() => navigate("/library")}
          ><Library size={17} /><span>Local library</span></button
        >
        <button
          class:active={activeView === "history"}
          on:click={() => navigate("/history")}
          ><Clock3 size={17} /><span>Listening history</span></button
        >
      </nav>
      <button
        class="sidebar-collapse-hover-zone sidebar-collapse-toggle"
        on:click={toggleSidebar}
        aria-label={sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
        title={sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
      >
        {#if sidebarCollapsed}<ChevronRight size={17} />{:else}<ChevronLeft
            size={17}
          />{/if}
      </button>
    </aside>

    {#if mobileNav}<button
        class="scrim"
        aria-label="Close navigation"
        on:click={() => (mobileNav = false)}><X /></button
      >{/if}
    <main
      class="content"
      inert={previewMode && previewPage === "search"}
      on:scroll={handleSearchScroll}
    >
      {#if railView === "queue"}
        <aside class="right-rail" aria-label="Queue" inert={previewMode}>
          <button
            class="rail-resizer"
            aria-label="Resize queue panel"
            on:pointerdown={startRailResize}
          ></button>
          <div class="rail-heading">
            <div>
              <p class="eyebrow">Up next</p>
              <h3>Queue <span>{queue.length}</span></h3>
            </div>
            <div class="rail-actions">
              {#if queue.length}<button
                  class="icon-button danger"
                  aria-label="Clear queue"
                  title="Clear queue"
                  disabled={!guestCanManageQueue}
                  on:click={clearQueue}><Trash2 size={15} /></button
                >{/if}
              {#if mobileViewport}<button
                  class="icon-button"
                  aria-label="Close queue"
                  on:click={() => switchRail("queue")}><X size={17} /></button
                >{/if}
            </div>
          </div>
          {#if queue.length}
            <div class="rail-queue-list">
              {#each queue as item, index}
                {@const download =
                  queueDownloadStatuses[getQueueAudioId(item)] ||
                  (getTrackSource(item) === "Local"
                    ? { status: "ready", progress: 100 }
                    : { status: "pending", progress: null })}
                {@const rowState = getQueueRowState(item)}
                <div
                  class="queue-row queue-draggable"
                  data-queue-index={index}
                  class:now-playing={rowState.nowPlaying}
                  class:up-next={index === 0 && !rowState.nowPlaying}
                  class:dragging={draggedQueueIndex === index}
                  class:drop-target={dragOverQueueIndex === index}
                  role="listitem"
                  aria-roledescription="reorderable queue item"
                  draggable={guestCanManageQueue}
                  on:dragstart={(event) => handleQueueDragStart(event, index)}
                  on:dragover={(event) => handleQueueDragOver(event, index)}
                  on:drop={(event) => handleQueueDrop(event, index)}
                  on:dragend={clearQueueDrag}
                >
                  <span
                    class="queue-index queue-drag-handle"
                    role="button"
                    tabindex={guestCanManageQueue ? 0 : -1}
                    aria-disabled={!guestCanManageQueue}
                    title="Drag to reorder"
                    aria-label={`Reorder ${item.title}; use arrow keys to move`}
                    on:pointerdown={(event) =>
                      handleQueuePointerDown(event, index)}
                    on:keydown={(event) =>
                      handleQueueReorderKeydown(event, index)}
                    ><span
                      class="queue-source-dot"
                      class:spotify={getTrackSource(item) === "Spotify"}
                      class:youtube={getTrackSource(item) === "YouTube"}
                      aria-hidden="true"
                    ></span></span
                  >
                  <span class="queue-art mini-art"
                    >{#if getTrackArt(item)}<img
                        src={getTrackArt(item)}
                        alt=""
                      />{:else}<Disc3
                        size={16}
                      />{/if}{#if isQueueDownloadable(item) && (download.status !== "ready" || downloadCompletionMarks[getQueueAudioId(item)])}<span
                        class="queue-download-indicator"
                        class:ready={download.status === "ready"}
                        class:completion={downloadCompletionMarks[
                          getQueueAudioId(item)
                        ]}
                        class:fading={downloadCompletionMarks[
                          getQueueAudioId(item)
                        ] === "fading"}
                        class:error={download.status === "error"}
                        class:indeterminate={download.progress == null}
                        style={`--download-progress: ${Math.min(100, Math.max(0, Number(download.progress) || 0))}%`}
                        role="img"
                        aria-label={download.error ||
                          (download.progress == null
                            ? download.status
                            : `${download.status} ${download.progress}%`)}
                        title={download.error ||
                          (download.progress == null
                            ? download.status
                            : `${download.status} ${download.progress}%`)}
                        ><span class="queue-download-ring"
                        ></span>{#if download.status === "ready"}<svg
                            class="queue-download-mark"
                            viewBox="0 0 16 16"
                            aria-hidden="true"
                            ><path d="m3.5 8.5 3 3 6-7" /></svg
                          >{:else if download.status === "error"}<span
                            class="queue-download-mark">!</span
                          >{:else if download.progress == null}<span
                            class="queue-download-spinner"
                          ></span>{:else}<span class="queue-download-percent"
                            >{Math.round(download.progress)}%</span
                          >{/if}</span
                      >{/if}</span
                  >
                  <button
                    class="queue-track"
                    disabled={!guestCanControlPlayback}
                    on:click={() => playTrack(item)}
                  >
                    <strong
                      ><span class="queue-track-title" use:marquee
                        >{item.title}</span
                      ></strong
                    ><small use:marquee
                      >{item.artist}</small
                    >
                  </button>
                  <button
                    class="row-action"
                    aria-label={`Play ${item.title}`}
                    disabled={!guestCanControlPlayback}
                    on:click={() => playTrack(item)}><Play size={15} /></button
                  >
                  <button
                    class="row-action"
                    aria-label={`Remove ${item.title} from queue`}
                    disabled={!guestCanManageQueue}
                    on:click={() => removeFromQueue(item.id)}
                    ><X size={15} /></button
                  >
                </div>
              {/each}
            </div>
          {:else}
            <p class="rail-empty">Your queue is ready for its next song.</p>
          {/if}
        </aside>
      {:else if railView === "party"}
        <aside class="right-rail" aria-label="Party details">
          <button
            class="rail-resizer"
            aria-label="Resize party panel"
            on:pointerdown={startRailResize}
          ></button>
          <div class="party-room-dialog">
            <div class="rail-heading">
              <h3>Room settings</h3>
              <button
                class="icon-button"
                aria-label="Close party details"
                on:click={() => switchRail("party")}><X size={17} /></button
              >
            </div>
            {#if joinedParty}
              {#if !roomServerUrl}<p class="party-member-empty">
                  Set the room server URL in Settings to resume shared
                  listening.
                </p>{/if}
              <section
                class="party-room-section party-invite-section"
                aria-labelledby="room-invite-heading"
              >
                <h4 id="room-invite-heading" class="party-room-section-title">
                  Invite listeners
                </h4>
                <div class="party-invite">
                  <button
                    class="party-code"
                    type="button"
                    on:click={copyPartyCode}
                    aria-label={codeCopied
                      ? "Room code copied"
                      : "Copy room code"}
                    title={codeCopied ? "Room code copied" : "Copy room code"}
                    aria-live="polite"
                  >
                    <span class="party-code-value">{joinedParty}</span>
                    <span class="party-code-hint"
                      >{#if codeCopied}<Check
                          size={14}
                          aria-hidden="true"
                        />{:else}<Copy
                          size={14}
                          aria-hidden="true"
                        />{/if}{codeCopied ? "Copied" : "Press to copy"}</span
                    >
                  </button>
                  {#if !isLocalPlayer()}<div class="party-invite-actions">
                      <button
                        class="icon-button invite-copy"
                        aria-label={inviteCopied
                          ? "Invite link copied"
                          : "Copy invite link"}
                        title={inviteCopied
                          ? "Invite link copied"
                          : "Copy invite link"}
                        on:click={copyPartyInvite}
                        >{#if inviteCopied}<Check size={16} />{:else}<Link
                            size={16}
                          />{/if}</button
                      >
                    </div>{/if}
                </div>
              </section>
              {#if partyRole === "host"}
                <section
                  class="party-room-section"
                  aria-labelledby="guest-controls-heading"
                >
                  <h4
                    id="guest-controls-heading"
                    class="party-room-section-title"
                  >
                    Controls
                  </h4>
                  <div
                    class="guest-controls"
                    role="group"
                    aria-labelledby="guest-controls-heading"
                  >
                    {#each guestPermissionOptions as option}
                      <label class="guest-control-toggle"
                        ><span>{option.label}</span><input
                          type="checkbox"
                          bind:checked={partyPermissions[option.key]}
                          on:change={savePartyPermissions}
                          disabled={!roomServerUrl}
                        /><span class="guest-control-switch" aria-hidden="true"
                        ></span></label
                      >
                    {/each}
                  </div>
                </section>
              {/if}
              <section
                class="party-member-list party-room-section"
                aria-label="Room participants"
              >
                <div class="party-member-heading">
                  <strong>People in this room</strong><span
                    >{partyMembers.length}</span
                  >
                </div>
                {#if partyMembers.length}
                  <ul>
                    {#each partyMembers as member}<li>
                        <span class="party-member-avatar"
                          >{member.trim().slice(0, 1).toUpperCase() ||
                            "?"}</span
                        ><span class="party-member-name" use:marquee
                          >{member}</span
                        >{#if member === (partyHostName || partyMembers[0])}<small
                            >Host</small
                          >{:else if member === partyDisplayName}<small
                            >You</small
                          >{/if}
                      </li>{/each}
                  </ul>
                {:else}<p class="party-member-empty">
                    Waiting for listeners to join.
                  </p>{/if}
              </section>
              {#if partyRole === "host"}
                <div class="party-room-end-action">
                  <button
                    class="outline-button close-party-button"
                    on:click={closeParty}
                    disabled={!roomServerUrl}>End Session</button
                  >
                </div>
              {:else}
                <div class="party-room-end-action">
                  <button
                    class="outline-button leave-room-button"
                    on:click={leaveParty}
                    disabled={!roomServerUrl}>Leave Room</button
                  >
                </div>
              {/if}
            {:else}
              <form
                class="party-room-form party-room-section"
                on:submit|preventDefault={startParty}
              >
                {#if !roomServerUrl}<p class="party-member-empty">
                    Add a room server in Settings to use shared listening.
                  </p>{/if}
                {#if !isLocalPlayer() && spotifyConnected && spotifyProfile}
                  <fieldset class="room-name-options">
                    <legend>Create room as</legend><label
                      ><input
                        type="radio"
                        bind:group={roomNameMode}
                        value="spotify"
                      />Spotify · {spotifyProfile.display_name ||
                        spotifyProfile.id}</label
                    ><label
                      ><input
                        type="radio"
                        bind:group={roomNameMode}
                        value="custom"
                      />Custom name</label
                    >
                  </fieldset>
                {/if}
                <label
                  >Your name<input
                    bind:value={username}
                    maxlength="50"
                    placeholder="Listener"
                  /></label
                >
                <label class="room-code-field"
                  >Room code<input
                    bind:value={partyCode}
                    maxlength="6"
                    placeholder="Enter a code"
                  /></label
                >
                <div class="party-room-form-actions">
                  <button
                    class="solid-button"
                    type="submit"
                    disabled={!roomServerUrlLoaded || !roomServerUrl}
                    >Join room</button
                  >
                  <button
                    class="outline-button"
                    type="button"
                    on:click={createRoom}
                    disabled={!roomServerUrlLoaded || !roomServerUrl}
                    >Create room</button
                  >
                </div>
              </form>
            {/if}
          </div>
        </aside>
      {/if}
      <header
        class:has-back={appNavigationStack.length > 1}
        class="topbar"
        data-tauri-drag-region
      >
        <div class="topbar-start" data-tauri-drag-region>
          <img
            class="topbar-brand-mark"
            src={brandMark}
            alt="Wavelength"
            data-tauri-drag-region
          />
          <div class="topbar-navigation" aria-label="Page navigation">
            <button
              class="topbar-navigation-button"
              type="button"
              disabled={
                appNavigationStack.length < 2 && activeView !== "collection"
              }
              on:click={goBack}
              aria-label="Go back"
              title="Back"><ChevronLeft size={17} /></button
            ><button
              class="topbar-navigation-button"
              type="button"
              disabled={!forwardNavigationStack.length}
              on:click={goForward}
              aria-label="Go forward"
              title="Forward"><ChevronRight size={17} /></button
            >
          </div>
        </div>
        <div class="topbar-center" data-tauri-drag-region>
          <div class="topbar-mobile-crumb">
            <strong>{pageTitle}</strong>
          </div>
          <div class="topbar-search-control">
            <form
              class="topbar-search"
              role="search"
              on:submit|preventDefault={submitTopbarSearch}
            >
              <div
                class="topbar-source-switch"
                role="group"
                aria-label="Search source"
              >
                <button
                  type="button"
                  disabled={!downloadFolderReady}
                  class:active={searchSource === "spotify"}
                  aria-pressed={searchSource === "spotify"}
                  on:click={() => changeTopbarSearchSource("spotify")}
                  >Spotify</button
                >
                <button
                  type="button"
                  disabled={!downloadFolderReady}
                  class:active={searchSource === "youtube"}
                  aria-pressed={searchSource === "youtube"}
                  on:click={() => changeTopbarSearchSource("youtube")}
                  >YouTube</button
                >
              </div>
              <label class="topbar-search-input">
                <Search size={17} aria-hidden="true" />
                <input
                  bind:value={search}
                  disabled={!downloadFolderReady}
                  on:focus={() => {
                    if (activeView !== "search") topbarSearchOpen = true;
                  }}
                  on:input={handleTopbarSearchInput}
                  aria-label="Search songs"
                  aria-expanded={activeView !== "search" &&
                    topbarSearchOpen &&
                    Boolean(search.trim())}
                  aria-controls="topbar-search-results"
                  placeholder="Search songs, artists, albums..."
                  autocomplete="off"
                />
                {#if search}<button
                    class="topbar-search-clear"
                    type="button"
                  disabled={!downloadFolderReady}
                    on:click={() => {
                      search = "";
                      handleTopbarSearchInput();
                    }}
                    aria-label="Clear search"><X size={15} /></button
                  >{/if}
              </label>
              {#if activeView !== "search" && topbarSearchOpen && search.trim()}
                <div
                  id="topbar-search-results"
                  class="topbar-search-results"
                  role="region"
                  aria-label="Song suggestions"
                >
                  {#if topbarSuggestions.length}
                    {#each topbarSuggestions as track}
                      <div class="topbar-search-result">
                        <button
                          class="topbar-search-result-main"
                          type="button"
                          disabled={!downloadFolderReady}
                          on:click={() => chooseTopbarSuggestion(track)}
                        >
                          <span
                            class="mini-art"
                            style={`--art-color:${track.color || "#718979"}`}
                            >{#if getTrackArt(track)}<img
                                src={getTrackArt(track)}
                                alt=""
                              />{:else}<Disc3 size={16} />{/if}</span
                          >
                          <span class="topbar-search-copy"
                            ><strong>{track.title}</strong><small
                              >{track.artist} · {getTrackSource(track)}</small
                            ></span
                          >
                        </button>
                        <span class="topbar-search-length"
                          >{track.length || ""}</span
                        >
                        <button
                          class="icon-button topbar-search-queue"
                          type="button"
                          disabled={!downloadFolderReady}
                          on:click={(event) => {
                            event.stopPropagation();
                            void addToQueue(track);
                            topbarSearchOpen = false;
                          }}
                          aria-label={`Add ${track.title} to queue`}
                          title="Add to queue"><ListMusic size={16} /></button
                        >
                        <button
                          class="icon-button topbar-search-queue"
                          type="button"
                          disabled={!downloadFolderReady}
                          on:click={(event) => {
                            event.stopPropagation();
                            void downloadTrack(track);
                            topbarSearchOpen = false;
                          }}
                          aria-label={`Download ${track.title}`}
                          title="Download song"><Download size={16} /></button
                        >
                      </div>
                    {/each}
                  {:else}
                    <p class="topbar-search-empty">
                      {#if isTauri() && searchSource === "spotify" && !spotifyConnected}
                        Connect Spotify in Settings to search by text, or paste a Spotify link.
                      {:else}
                        No library matches. Press Enter to search {searchSource ===
                        "youtube"
                          ? "YouTube"
                          : "Spotify"}.
                      {/if}
                    </p>
                  {/if}
                </div>
              {/if}
            </form>
          </div>
        </div>
        <div class="topbar-end" data-tauri-drag-region>
          {#if mobileViewport}
          <button
            class="mobile-party-toggle"
            class:active={railView === "party"}
            on:click={openPartySettings}
            aria-label="Toggle party details"
            title="Party details"><Users size={18} /></button
          >
          {/if}
          <div class="top-actions profile-anchor">
            <button
              class="avatar"
              on:click={() => navigate("/settings")}
              aria-label="Open profile settings"
              >{username ? username[0].toUpperCase() : "W"}</button
            >
          </div>
          {#if appWindow}<div class="window-controls" aria-label="Window controls">
              <button
                class="window-control"
                type="button"
                on:click={() => appWindow.minimize()}
                aria-label="Minimize window"
                title="Minimize"><Minus size={15} /></button
              ><button
                class="window-control"
                type="button"
                on:click={() => appWindow.toggleMaximize()}
                aria-label="Maximize or restore window"
                title="Maximize or restore"><Square size={13} /></button
              ><button
                class="window-control window-close-control"
                type="button"
                on:click={() => appWindow.close()}
                aria-label="Close window"
                title="Close"><X size={15} /></button
              >
            </div>{/if}
        </div>
      </header>

      <section
        bind:this={contentInner}
        aria-label="Page content"
        class:scrollable={contentHasOverflow}
        class:drag-scrolling={isContentDragging}
        class:page-home={activeView === "overview"}
        class:page-search={activeView === "search"}
        class:page-library={activeView === "songs"}
        class:page-playlists={activeView === "playlists"}
        class:page-queue={activeView === "queue"}
        class:page-settings={activeView === "settings"}
        class:page-customization={activeView === "customization"}
        class:page-equalizer={activeView === "equalizer"}
        class:page-now-playing={activeView === "now-playing"}
        class="content-inner page-layout"
        on:scroll={handleSearchScroll}
        on:pointerdown={startContentDrag}
        on:pointermove={dragContent}
        on:pointerup={stopContentDrag}
        on:pointercancel={stopContentDrag}
      >
        {#if lyricsFullscreen && playing}
          <FullscreenLyrics
            {playing}
            {mobileViewport}
            {lyrics}
            {syncedLyricLines}
            {activeLyricIndex}
            bind:lyricsList
            {getTrackArt}
            onClose={closeLyricsFullscreen}
          />
        {:else}
          {#if notice}{#key notice}<div
                class="toast"
                role="status"
                aria-live="polite"
                style={`--toast-duration:${noticeRemaining}ms`}
                on:pointerenter={pauseNoticeDismiss}
                on:pointerleave={resumeNoticeDismiss}
                on:focusin={pauseNoticeDismiss}
                on:focusout={resumeNoticeDismiss}
              >
                <span>{notice}</span><button
                  class="toast-close"
                  aria-label="Dismiss notification"
                  on:click={dismissNotice}><X size={17} /></button
                ><span class="toast-progress" aria-hidden="true"></span>
              </div>{/key}{/if}
          {#if activeView === "now-playing"}
            <NowPlayingPage
              {playing}
              trackMetadata={nowPlayingMetadata}
              {isPlaying}
              canControlPlayback={guestCanControlPlayback}
              {lyricsFullscreen}
              {getTrackArt}
              {getTrackSource}
              {isFavorite}
              {duration}
              {currentTime}
              {formatTime}
              {shuffleOn}
              {repeatOn}
              {repeatMode}
              {appAccent}
              {contrastColor}
              {lyrics}
              {syncedLyricLines}
              {activeLyricIndex}
              {mobileViewport}
              bind:lyricsList
              onNavigate={navigate}
              onFavorite={toggleFavorite}
              onContextMenu={showContextMenu}
              onSeek={seek}
              onShuffle={toggleShuffle}
              onPrevious={previousTrack}
              onTogglePlayback={togglePlayback}
              onNext={nextTrack}
              onToggleRepeat={toggleRepeat}
              onCloseLyricsFullscreen={closeLyricsFullscreen}
              onOpenLyricsFullscreen={openLyricsFullscreen}
              onLyricsLinesKeydown={handleLyricsLinesKeydown}
            />
          {/if}
          {#if activeView === "songs"}
            <LibraryPage
              bind:libraryFilter
              bind:importedSearch
              bind:importedSortBy
              {mobileViewport}
              {libraryCollections}
              {filteredImportedTracks}
              {importedSortOptions}
              {importedFolderTree}
              {importedBrowseGroups}
              {getCollectionArt}
              {getTrackArt}
              {getTrackSource}
              {isFavorite}
              {isLikedSongsCollection}
              onOpenFolderManager={openFolderManager}
              onToggleImportedFolder={toggleImportedFolder}
              onPlay={playTrack}
              onQueue={addToQueue}
              onFavorite={toggleFavorite}
              onContextMenu={showContextMenu}
              onCreatePlaylist={createPlaylist}
              onOpenCollection={openCollection}
              onDuplicateSpotifyPlaylist={duplicateSpotifyPlaylist}
              onLikeTracks={likeTracks}
                          onAddSongs={addSongsToPlaylist}
            />
          {/if}
          {#if activeView === "playlists"}
            <PlaylistsPage
              {savedPlaylistCollections}
              {getCollectionArt}
              {isLikedSongsCollection}
              onCreatePlaylist={createPlaylist}
              onOpenCollection={openCollection}
              onDuplicateSpotifyPlaylist={duplicateSpotifyPlaylist}
              onQueueCollection={(collection) =>
                collection.tracks?.forEach(addToQueue)}
              onLikeTracks={likeTracks}
              onAddSongs={addSongsToPlaylist}
            />
          {/if}
          {#if activeView === "collection"}
            <CollectionDetailPage
              {selectedCollection}
              {getTrackArt}
              {isFavorite}
              {isLikedSongsCollection}
              onNavigate={navigate}
              onPlay={playTrack}
              onQueue={addToQueue}
              onFavorite={toggleFavorite}
              onContextMenu={showContextMenu}
              onAddSongs={addSongsToPlaylist}
            />
          {/if}
          {#if activeView === "overview"}
            <HomePage
              {libraryCollections}
              {playHistory}
              {importedTracks}
              {artistStats}
              {songStats}
              bind:statsPeriod
              {statsPeriodOptions}
              {recentlyAdded}
              {downloadFolderReady}
              {downloadDirectoryHandle}
              bind:recentAddedGrid
              {getCollectionArt}
              {getTrackArt}
              {getTrackSource}
              {isFavorite}
              onChooseDownloadDirectory={chooseDownloadDirectory}
              onNavigate={navigate}
              onOpenCollection={openCollection}
              onPlay={playTrack}
              onQueue={addToQueue}
              onFavorite={toggleFavorite}
              onScrollRecentlyAdded={scrollRecentlyAdded}
              onContextMenu={showContextMenu}
            />
          {/if}
          {#if activeView === "history"}
            <ListeningHistory
              {playHistory}
              {getTrackArt}
              {isFavorite}
              onPlay={playTrack}
              onQueue={addToQueue}
              onToggleFavorite={toggleFavorite}
              onClear={clearPlayHistory}
              onContextMenu={showContextMenu}
            />
          {/if}
          {#if activeView === "search"}
            <SearchResultsPage
              bind:search
              bind:searchSource
              bind:searchTable
              {searchSourceOptions}
              {searchResults}
              {displayedTracks}
              {searchLoading}
              {searchLoadingMore}
              {searchAtBottom}
              {searchHasMore}
              {mobileViewport}
              searchDisabled={!downloadFolderReady}
              {getTrackArt}
              onScroll={handleSearchScroll}
              onMobileSearchSourceChange={handleMobileSearchSourceChange}
              onMobileSearchInput={handleMobileSearchInput}
              onSearch={() => runSearch({ limit: 10 })}
              onPlay={playTrack}
              onContextMenu={showContextMenu}
              onPointerDown={handleSongPointerDown}
              onPointerUp={handleSongTouchEnd}
              onPointerLeave={clearContextMenuTimer}
              onClearContextMenuTimer={clearContextMenuTimer}
            />
          {/if}
          {#if activeView === "queue"}
            <QueuePage
              {queue}
              canManageQueue={guestCanManageQueue}
              canControlPlayback={guestCanControlPlayback}
              {queueDownloadStatuses}
              queueDownloadCompletionMarks={downloadCompletionMarks}
              {draggedQueueIndex}
              {dragOverQueueIndex}
              {getQueueAudioId}
              {getQueueRowState}
              {getTrackArt}
              {getTrackSource}
              {isQueueDownloadable}
              onClearQueue={clearQueue}
              onPlay={playTrack}
              onRemove={removeFromQueue}
              onDragStart={handleQueueDragStart}
              onDragOver={handleQueueDragOver}
              onDrop={handleQueueDrop}
              onDragEnd={clearQueueDrag}
              onPointerDown={handleQueuePointerDown}
              onReorderKeydown={handleQueueReorderKeydown}
              onContextMenu={showContextMenu}
            />
          {/if}
          {#if activeView === "logs"}
            <ActivityPage
              {downloadLogs}
              {downloadLogsError}
              {downloadLogsLoading}
              {notificationLogs}
              {formatLogTime}
            />
          {/if}
          {#if activeView === "settings"}
            <SettingsPage
              {spotifyConnected}
              {spotifyProfile}
              bind:spotifyClientId
              {downloadDirectoryName}
              tauriAvailable={isTauri()}
              {appVersion}
              {appUpdateChecking}
              {appUpdateStatus}
              qobuzConfigured={qobuzConfigured}
              {appTheme}
              {appAccent}
              {accentOptions}
              {roomServerSaving}
              {roomServerUrl}
              bind:roomServerInput
              {mobileViewport}
              onDisconnectSpotify={disconnectSpotify}
              onConnectSpotify={connectSpotify}
              onSaveSpotifyClientId={saveSpotifyClientId}
              onOpenFolderManager={openFolderManager}
              onChooseDownloadDirectory={chooseDownloadDirectory}
              onNavigate={navigate}
              onSaveRoomServerUrl={saveRoomServerUrl}
              onOpenQobuzCredentials={openQobuzCredentials}
              onCheckForAppUpdates={checkForAppUpdates}
            />
          {/if}
          {#if activeView === "customization"}
            <CustomizationPage
              {appTheme}
              {themeOptions}
              {appAccent}
              {appBackgroundColor}
              defaultBackgroundColor={resolvedTheme() === "light"
                ? "#f5f6f2"
                : "#10110f"}
              onThemeChange={setTheme}
              onBackgroundChange={setBackgroundColor}
              onOpenAccentPicker={openAccentPicker}
            />
          {/if}
          {#if activeView === "equalizer"}
            <EqualizerPage
              bind:equalizerSpectrumCanvas
              bind:activeEqualizerPresetId
              bind:equalizerPresetName
              {equalizerPresetOptions}
              {equalizerBands}
              {equalizerGains}
              {equalizerFrequencies}
              {equalizerQs}
              {equalizerTypeOptions}
              {equalizerTypes}
              {equalizerNodeColors}
              bind:selectedEqualizerBand
              {equalizerUnavailable}
              onSelectPreset={selectEqualizerPreset}
              onSavePreset={saveEqualizerPreset}
              onResetEqualizer={resetEqualizer}
              onResetBand={resetEqualizerBand}
              onSetGain={setEqualizerGain}
              onSetFrequency={setEqualizerFrequency}
              onSetQ={setEqualizerQ}
              onSetType={setEqualizerType}
              onGraphPointerDown={startEqualizerBandDrag}
              onGraphPointerMove={moveEqualizerBand}
              onGraphPointerUp={endEqualizerBandDrag}
              onGraphPointerCancel={endEqualizerBandDrag}
            />
          {/if}
        {/if}
        {#if showAccentPicker}
          <div
            class="modal-backdrop content-area-backdrop accent-picker-backdrop"
            role="presentation"
            style={`--content-modal-left:${contentModalBounds.contentLeft}px;--content-modal-top:${contentModalBounds.contentTop}px;--content-modal-center-x:${contentModalBounds.centerX}px;--content-modal-center-y:${contentModalBounds.centerY}px`}
            on:click={(event) => {
              if (event.target === event.currentTarget)
                showAccentPicker = false;
            }}
          >
            <div
              class="modal accent-picker-modal"
              role="dialog"
              aria-modal="true"
              aria-labelledby="accent-picker-heading"
            >
              <button
                type="button"
                class="modal-close"
                on:click={() => (showAccentPicker = false)}
                aria-label="Close accent picker"><X /></button
              >
              <p class="eyebrow">Customization</p>
              <h2 id="accent-picker-heading">Accent color</h2>
              <div
                class="accent-picker-list"
                role="group"
                aria-label="Accent color"
              >
                {#each accentOptions as accent}<button
                    type="button"
                    class:active={appAccent === accent.value}
                    on:click={() => {
                      setAccent(accent.value);
                      customAccentError = "";
                      showAccentPicker = false;
                    }}
                    aria-label={`${accent.name} accent`}
                    title={accent.name}
                    aria-pressed={appAccent === accent.value}
                    ><span
                      class="accent-option-swatch"
                      style={`background:${accent.value}`}
                    ></span>{#if appAccent === accent.value}<Check
                        size={15}
                      />{/if}</button
                  >{/each}
              </div>
              <form
                class="custom-accent-form"
                on:submit|preventDefault={applyCustomAccent}
              >
                <label for="custom-accent-hex">Custom hex color</label>
                <div>
                  <input
                    id="custom-accent-hex"
                    bind:value={customAccentInput}
                    maxlength="7"
                    placeholder="#D7C7A4"
                    aria-invalid={Boolean(customAccentError)}
                    aria-describedby="custom-accent-error"
                  /><button class="solid-button compact-action" type="submit"
                    >Apply</button
                  >
                </div>
                {#if customAccentError}<p id="custom-accent-error" role="alert">
                    {customAccentError}
                  </p>{/if}
              </form>
            </div>
          </div>
        {/if}
      </section>
      <PlayerBar
        {playing}
        {isPlaying}
        canControlPlayback={guestCanControlPlayback}
        {isFavorite}
        {getTrackArt}
        {getTrackSource}
        {shuffleOn}
        {repeatMode}
        {repeatOn}
        {appAccent}
        {contrastColor}
        {currentTime}
        {duration}
        {volume}
        {isMuted}
        {volumeSliderOpen}
        {mobileViewport}
        {lyricsFullscreen}
        {railView}
        {joinedParty}
        {partyMembers}
        {formatTime}
        onOpenNowPlaying={openNowPlaying}
        onContextMenu={showContextMenu}
        onFavorite={toggleFavorite}
        onShuffle={toggleShuffle}
        onPrevious={previousTrack}
        onTogglePlayback={togglePlayback}
        onNext={nextTrack}
        onRepeat={toggleRepeat}
        onSeek={seek}
        onToggleLyrics={toggleLyricsFullscreen}
        onOpenPartySettings={openPartySettings}
        onShowQueue={() => switchRail("queue")}
        onVolumeButtonClick={handleVolumeButtonClick}
        onVolumeInput={handleVolumeInput}
      />
    </main>
  </div>
  {#if !(mobileViewport && activeView === "now-playing")}
    <div
      class="mobile-playback-progress"
      role="progressbar"
      aria-label="Playback progress"
      aria-valuemin="0"
      aria-valuemax={duration || 1}
      aria-valuenow={currentTime}
    >
      <span
        style={`transform:scaleX(${duration > 0 ? Math.min(currentTime / duration, 1) : 0})`}
      ></span>
    </div>
  {/if}
  <nav
    class="mobile-bottom-nav"
    aria-label="Mobile navigation"
    inert={inviteGateRequired}
    aria-hidden={inviteGateRequired}
  >
    <button
      class:active={activeView === "overview"}
      on:click={() => navigate("/player")}
      aria-label="Home"><Home size={18} /><span>Home</span></button
    >
    <button
      class:active={activeView === "search"}
      on:click={() => navigate("/search")}
      aria-label="Search"><Search size={18} /><span>Search</span></button
    >
    <button
      class:active={activeView === "songs"}
      on:click={() => navigate("/library")}
      aria-label="Library"><Library size={18} /><span>Library</span></button
    >
    <div class="mobile-settings">
      <button
        class="avatar"
        on:click={() => navigate("/settings")}
        aria-label="Open profile settings"
        >{username ? username[0].toUpperCase() : "W"}</button
      >
    </div>
  </nav>

  {#if showFolderManager}
    <div
      class="modal-backdrop content-area-backdrop folder-manager-backdrop"
      role="presentation"
      style={`--content-modal-center-x:${contentModalBounds.centerX}px;--content-modal-center-y:${contentModalBounds.centerY}px`}
      on:click={(event) => {
        if (event.target === event.currentTarget) showFolderManager = false;
      }}
    >
      <div
        class="modal folder-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="folder-manager-title"
      >
        <button
          type="button"
          class="modal-close"
          on:click={() => (showFolderManager = false)}
          aria-label="Close music folders"><X /></button
        >
        <div class="folder-modal-heading">
          <div class="folder-modal-title">
            <h2 id="folder-manager-title">Music folders</h2>
            <button
              type="button"
              class="icon-button folder-add-button"
              on:click={chooseLocalFolder}
              aria-label="Add music folder"
              title="Add folder"><Plus size={18} /></button
            >
            <button
              type="button"
              class="icon-button folder-add-button"
              on:click={rescanLocalFolders}
              aria-label="Rescan saved music folders"
              title="Rescan folders"><RefreshCw size={18} /></button
            >
          </div>
          <p>Saved on this device for Wavelength</p>
        </div>
        <div class="folder-list">
          {#if localFolders.length}{#each localFolders as folder}<div
                class="folder-row"
              >
                <span
                  ><FolderOpen size={16} /><strong>{folder.name}</strong></span
                ><button
                  class="icon-button danger"
                  on:click={() => removeLocalFolder(folder)}
                  aria-label={`Remove ${folder.name}`}
                  ><Trash2 size={16} /></button
                >
              </div>{/each}{:else}<p class="rail-empty">
              No imported folders yet.
            </p>{/if}
        </div>
      </div>
    </div>
  {/if}
  {#if qobuzCredentialsOpen}
    <div
      class="modal-backdrop content-area-backdrop folder-manager-backdrop"
      role="presentation"
      style={`--content-modal-center-x:${contentModalBounds.centerX}px;--content-modal-center-y:${contentModalBounds.centerY}px`}
      on:click={(event) => {
        if (event.target === event.currentTarget) qobuzCredentialsOpen = false;
      }}
    >
      <div
        class="modal folder-modal qobuz-credentials-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="qobuz-credentials-title"
      >
        <button
          type="button"
          class="modal-close"
          on:click={() => (qobuzCredentialsOpen = false)}
          aria-label="Close Qobuz credentials"><X /></button
        >
        <div class="folder-modal-heading">
          <div class="folder-modal-title">
            <h2 id="qobuz-credentials-title">Qobuz credentials</h2>
          </div>
          <p>Spotify downloads use Qobuz when all three credentials are set</p>
        </div>
        <form class="qobuz-credentials-form" on:submit|preventDefault={saveQobuzCredentials}>
          <label>App ID<input bind:value={qobuzCredentials.appId} autocomplete="off" /></label>
          <label>App secret<input type="password" bind:value={qobuzCredentials.appSecret} autocomplete="new-password" /></label>
          <label>User token<input type="password" bind:value={qobuzCredentials.userToken} autocomplete="new-password" /></label>
          <div class="qobuz-credentials-actions">
            <button
              class="outline-button compact-action"
              type="button"
              on:click={() => (qobuzCredentialsOpen = false)}>Cancel</button
            >
            <button
              class="outline-button compact-action"
              type="submit"
              disabled={qobuzSaving}>{qobuzSaving ? "Saving" : "Save"}</button
            >
          </div>
        </form>
      </div>
    </div>
  {/if}
  {#if showAppUpdatePrompt && pendingDesktopUpdate}
    <div
      class="modal-backdrop content-area-backdrop folder-manager-backdrop"
      role="presentation"
      style={`--content-modal-center-x:${contentModalBounds.centerX}px;--content-modal-center-y:${contentModalBounds.centerY}px`}
      on:click={(event) => {
        if (event.target === event.currentTarget) dismissAppUpdatePrompt();
      }}
    >
      <div
        class="modal folder-modal app-update-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="app-update-title"
      >
        <button
          type="button"
          class="modal-close"
          on:click={dismissAppUpdatePrompt}
          aria-label="Postpone Wavelength update"><X /></button
        >
        <div class="folder-modal-heading">
          <div class="folder-modal-title">
            <h2 id="app-update-title">Wavelength {pendingDesktopUpdate.version} is available</h2>
          </div>
          <p>{pendingDesktopUpdate.body || "A signed update is ready to install."}</p>
        </div>
        <div class="qobuz-credentials-actions">
          <button
            class="outline-button compact-action"
            type="button"
            on:click={dismissAppUpdatePrompt}>Not now</button
          >
          <button
            class="outline-button compact-action"
            type="button"
            disabled={appUpdateInstalling}
            on:click={installPendingAppUpdate}
            >{appUpdateInstalling ? "Installing" : "Install update"}</button
          >
        </div>
      </div>
    </div>
  {/if}
  {#if showCreatePlaylist}
    <div
      class="modal-backdrop content-area-backdrop playlist-create-backdrop"
      role="presentation"
      style={`--content-modal-center-x:${contentModalBounds.centerX}px;--content-modal-center-y:${contentModalBounds.centerY}px`}
      on:click={(event) => {
        if (event.target === event.currentTarget) closeCreatePlaylist();
      }}
    >
      <div
        class="modal playlist-create-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="create-playlist-title"
      >
        <button
          type="button"
          class="modal-close"
          on:click={closeCreatePlaylist}
          aria-label="Close create playlist"><X /></button
        >
        <h2 id="create-playlist-title">Create playlist</h2>
        <form
          class="playlist-create-form"
          on:submit|preventDefault={submitCreatePlaylist}
        >
          <input
            id="create-playlist-name"
            aria-label="Playlist name"
            bind:this={createPlaylistInput}
            bind:value={createPlaylistName}
            maxlength="60"
            autocomplete="off"
            placeholder="Name this playlist"
          />
          <div class="playlist-create-actions">
            <button
              class="outline-button compact-action"
              type="button"
              on:click={closeCreatePlaylist}>Cancel</button
            ><button
              class="solid-button compact-action"
              type="submit"
              disabled={!createPlaylistName.trim()}>Create</button
            >
          </div>
        </form>
      </div>
    </div>
  {/if}
  <TrackContextMenu
    {contextMenu}
    {playlists}
    {spotifyPlaylists}
    preferredPlaylist={playlistAddTarget}
    artistUrl={contextMenu?.track?.spotify_artist_url || ""}
    albumUrl={contextMenu?.track?.spotify_album_url || ""}
    songUrl={contextMenu?.track?.spotify_url || ""}
    isFavorite={contextMenu?.track ? isFavorite(contextMenu.track) : false}
    downloadDisabled={!downloadFolderReady}
    onAddToPlaylist={addTrackToPlaylist}
    onCreatePlaylist={createPlaylistFromTrack}
    onQueue={(track) => {
      addToQueue(track);
      contextMenu = null;
    }}
    onToggleFavorite={(track) => {
      toggleFavorite(track);
      contextMenu = null;
    }}
    onDownload={(track) => {
      downloadTrack(track);
      contextMenu = null;
    }}
    onOpenPage={openExternalPage}
    onSongDetails={openSongDetails}
  />
  <SongDetailsModal
    track={songDetailsTrack}
    {getTrackArt}
    {getTrackSource}
    onClose={() => (songDetailsTrack = null)}
    onOpenPage={openExternalPage}
  />
</div>

{#if inviteGateRequired}
  <div class="invite-gate" role="presentation">
    <div
      class="invite-gate-panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="invite-gate-title"
    >
      <p class="eyebrow">Shared listening</p>
      <h1 id="invite-gate-title">Join room {pendingInviteCode}</h1>
      <p>Enter a name or connect Spotify to join this session.</p>
      {#if joinedParty && joinedParty !== pendingInviteCode}
        <p class="invite-gate-warning">
          You are already in room {joinedParty}. End that room before joining
          this one.
        </p>
        {#if partyRole === "host"}<button
            class="outline-button"
            on:click={closeParty}>End current room</button
          >{/if}
      {/if}
      <form
        class="invite-gate-form"
        on:submit|preventDefault={() =>
          joinPartyAs(inviteGuestName, pendingInviteCode)}
      >
        <label
          >Your name<input
            bind:value={inviteGuestName}
            maxlength="50"
            autocomplete="name"
            placeholder="Listener"
          /></label
        >
        <button class="solid-button" type="submit">Join as guest</button>
      </form>
      {#if !isLocalPlayer() && spotifyConnected && spotifyProfile}
        <button
          class="outline-button invite-spotify-join"
          on:click={() =>
            joinPartyAs(
              spotifyProfile.display_name || spotifyProfile.id,
              pendingInviteCode,
            )}
          >Join as {spotifyProfile.display_name || spotifyProfile.id}</button
        >
      {:else if !isLocalPlayer()}
        <button
          class="outline-button invite-spotify-join"
          on:click={connectSpotify}>Connect Spotify</button
        >
      {/if}
      {#if notice}<p class="invite-gate-notice" role="status">{notice}</p>{/if}
    </div>
  </div>
{/if}
