<script>
    import { ChevronRight } from "lucide-svelte";

    export let spotifyConnected = false;
    export let spotifyProfile = null;
    export let spotifyClientId = "";
    export let downloadDirectoryName = "";
    export let tauriAvailable = false;
    export let appVersion = "Web app";
    export let appUpdateChecking = false;
    export let appUpdateStatus = "";
    export let qobuzConfigured = false;
    export let appTheme = "dark";
    export let appAccent = "";
    export let accentOptions = [];
    export let roomServerSaving = false;
    export let roomServerUrl = "";
    export let roomServerInput = "";
    export let roomServerPasscodeInput = "";
    export let roomServerConnected = false;
    export let mobileViewport = false;
    export let onDisconnectSpotify;
    export let onConnectSpotify;
    export let onSaveSpotifyClientId;
    export let onOpenFolderManager;
    export let onChooseDownloadDirectory;
    export let onNavigate;
    export let onSaveRoomServerUrl;
    export let onOpenQobuzCredentials;
    export let onCheckForAppUpdates;
</script>

<section class="settings-page">
    <header class="settings-page-heading"><h1>Settings</h1></header>
    <section class="settings-list" aria-label="Settings">
        <div class="settings-row">
            <div class="settings-row-copy">
                <strong>Spotify</strong>{#if spotifyConnected}<small
                        >{spotifyProfile?.display_name ||
                            spotifyProfile?.id ||
                            "Connected Spotify account"}</small
                    >{/if}
            </div>
            <button
                class="outline-button compact-action"
                on:click={spotifyConnected
                    ? onDisconnectSpotify
                    : onConnectSpotify}
                >{spotifyConnected ? "Disconnect" : "Connect"}</button
            >
        </div>
        <div class="settings-row spotify-client-id-row">
            <div class="settings-row-copy">
                <strong>Spotify Client ID</strong>
                <small
                    >Required for direct Spotify OAuth; no client secret is needed.
                    Use a Spotify Developer app ID and register
                    http://127.0.0.1:43821/callback as a Redirect URI.
                    Web apps also need their exact app URL.</small
                >
            </div>
            <input
                class="room-server-input spotify-client-id-input"
                type="text"
                bind:value={spotifyClientId}
                on:change={onSaveSpotifyClientId}
                placeholder="Spotify Client ID"
                aria-label="Spotify Client ID"
                autocomplete="off"
                maxlength="64"
            />
        </div>
        <div class="settings-row">
            <div class="settings-row-copy">
                <strong>Qobuz</strong><small
                    >{qobuzConfigured ? "Configured for Spotify downloads" : "Not configured"}</small
                >
            </div>
            <button
                class="outline-button compact-action"
                on:click={onOpenQobuzCredentials}
                >Set credentials</button
            >
        </div>
        <div class="settings-row">
            <div class="settings-row-copy"><strong>Local Folders</strong></div>
            <button
                class="outline-button compact-action"
                on:click={onOpenFolderManager}>Manage</button
            >
        </div>
        <div class="settings-row">
                <div class="settings-row-copy">
                    <strong>Download Folder</strong><small
                        >{downloadDirectoryName
                            ? `${downloadDirectoryName} · played songs save here automatically`
                            : "Choose a destination for played songs and downloads."}</small
                    >
                </div>
                <button
                    class="outline-button compact-action"
                    on:click={onChooseDownloadDirectory}
                    >{downloadDirectoryName ? "Change" : "Choose"}</button
                >
            </div>
        <button
            class="settings-row settings-nav-row"
            on:click={() => onNavigate("/customization")}
            ><div class="settings-row-copy"><strong>Customization</strong></div>
            <span class="settings-nav-summary"
                >{appTheme === "system"
                    ? "System"
                    : appTheme === "dark"
                      ? "Dark"
                      : "Light"} · {accentOptions.find(
                    (accent) => accent.value === appAccent,
                )?.name || "Custom"}</span
            ><ChevronRight size={17} aria-hidden="true" /></button
        >
        <button
            class="settings-row settings-nav-row"
            on:click={() => onNavigate("/equalizer")}
            ><div class="settings-row-copy"><strong>Equalizer</strong></div>
            <ChevronRight size={17} aria-hidden="true" /></button
        >
        {#if mobileViewport}<button
                class="settings-row settings-nav-row"
                on:click={() => onNavigate("/history")}
                ><div class="settings-row-copy">
                    <strong>Listening history</strong>
                </div>
                <ChevronRight size={17} aria-hidden="true" /></button
            >{/if}
        <div class="settings-row room-server-settings-row">
            <div class="room-server-heading">
                <strong>Self-hosted server</strong><small
                class:connected={roomServerConnected}
                    >{roomServerSaving
                    ? "Connecting"
                        : roomServerUrl
                      ? roomServerConnected
                          ? "Connected"
                          : "Passcode required"
                          : "Not connected"}</small
                >
            </div>
            <div class="room-server-controls">
                <input
                    class="room-server-input"
                    type="url"
                    bind:value={roomServerInput}
                    on:change={onSaveRoomServerUrl}
                    placeholder="https://music.example.com"
                    aria-label="Room server URL"
                />
                <input
                    class="room-server-input"
                    type="password"
                    bind:value={roomServerPasscodeInput}
                    on:change={onSaveRoomServerUrl}
                    placeholder="Server passcode"
                    aria-label="Room server passcode"
                    autocomplete="off"
                    maxlength="128"
                />
            </div>
            <small>{tauriAvailable
                ? "Passcode is saved in the system credential vault."
                : "Passcode is saved in this browser's local storage."}</small>
        </div>
        <div class="settings-row">
            <div class="settings-row-copy"><strong>Logging</strong></div>
            <button
                class="outline-button compact-action"
                on:click={() => onNavigate("/logs")}>View logs</button
            >
        </div>
    </section>
    <section class="settings-section" aria-label="About">
        <header class="settings-section-heading"><h2>About</h2></header>
        <div class="settings-list">
            <div class="settings-row">
                <div class="settings-row-copy">
                    <strong>Wavelength</strong>
                    <small>{tauriAvailable ? `Version ${appVersion}` : "Web app"}</small>
                </div>
            </div>
            <div class="settings-row">
                <div class="settings-row-copy">
                    <strong>Updates</strong><small
                        >{appUpdateStatus || (tauriAvailable ? "Checks on launch and every six hours" : "Available in the desktop app")}</small
                    >
                </div>
                <button
                    class="outline-button compact-action"
                    disabled={!tauriAvailable || appUpdateChecking}
                    on:click={() => onCheckForAppUpdates()}
                    >{!tauriAvailable ? "Desktop app only" : appUpdateChecking ? "Checking" : "Check now"}</button
                >
            </div>
        </div>
    </section>
</section>
