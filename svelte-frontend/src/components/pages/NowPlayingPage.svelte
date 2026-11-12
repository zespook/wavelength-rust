<script>
    import {
        ChevronDown,
        ChevronLeft,
        Disc3,
        Heart,
        ListMusic,
        Maximize2,
        Pause,
        Play,
        Plus,
        Repeat2,
        Shuffle,
        SkipBack,
        SkipForward,
        X,
    } from "lucide-svelte";
    import { marquee } from "../../marquee.js";
    import { averageArtworkColor } from "../../artwork-color.js";

    export let playing = null;
    export let trackMetadata = {};
    export let isPlaying = false;
    export let canControlPlayback = true;
    export let lyricsFullscreen = false;
    export let getTrackArt;
    export let getTrackSource;
    export let isFavorite;
    export let duration = 0;
    export let currentTime = 0;
    export let formatTime;
    export let shuffleOn = false;
    export let repeatOn = false;
    export let repeatMode = "off";
    export let appAccent = "";
    export let contrastColor;
    export let lyrics = "";
    export let syncedLyricLines = [];
    export let activeLyricIndex = -1;
    export let mobileViewport = false;
    export let lyricsList = null;
    export let onNavigate;
    export let onFavorite;
    export let onContextMenu;
    export let onSeek;
    export let onShuffle;
    export let onPrevious;
    export let onTogglePlayback;
    export let onNext;
    export let onToggleRepeat;
    export let onCloseLyricsFullscreen;
    export let onOpenLyricsFullscreen;
    export let onLyricsLinesKeydown;

    let actionsOpen = false;
    let actionMenuRoot;
    let actionMenuTrigger;
    let lyricsBackgroundColor = "#101311";
    let lyricsArtworkRequest = 0;
    let requestedLyricsArtwork = "";

    $: lyricsArtworkUrl = playing && getTrackArt ? getTrackArt(playing) : "";
    $: if (lyricsArtworkUrl !== requestedLyricsArtwork) {
        requestedLyricsArtwork = lyricsArtworkUrl;
        updateLyricsBackground(lyricsArtworkUrl);
    }

    async function updateLyricsBackground(source) {
        const request = ++lyricsArtworkRequest;
        const fallbackColor = playing?.color || "#101311";
        lyricsBackgroundColor = fallbackColor;
        const color = await averageArtworkColor(source, fallbackColor);
        if (request === lyricsArtworkRequest) lyricsBackgroundColor = color;
    }

    function closeActionMenuOnOutsidePointer(event) {
        if (!actionMenuRoot?.contains(event.target)) actionsOpen = false;
    }

    function handleActionMenuKeydown(event) {
        if (!actionsOpen || event.key !== "Escape") return;
        event.preventDefault();
        actionsOpen = false;
        actionMenuTrigger?.focus();
    }

    function formatBpm(value) {
        const bpm = Number(value);
        return Number.isFinite(bpm) && bpm > 0 ? `${Math.round(bpm)} BPM` : "—";
    }

    function formatReleaseDate(value) {
        const releaseDate = String(value || "");
        if (!releaseDate) return "—";
        if (/^\d{4}$/.test(releaseDate)) return releaseDate;
        const hasDay = /^\d{4}-\d{2}-\d{2}$/.test(releaseDate);
        const dateValue = hasDay || /^\d{4}-\d{2}$/.test(releaseDate)
            ? new Date(`${releaseDate}${hasDay ? "" : "-01"}T00:00:00`)
            : null;
        if (!dateValue || Number.isNaN(dateValue.getTime())) return releaseDate;
        return new Intl.DateTimeFormat(undefined, {
            year: "numeric",
            month: "short",
            ...(hasDay ? { day: "numeric" } : {}),
        }).format(dateValue);
    }
</script>

<svelte:window on:pointerdown={closeActionMenuOnOutsidePointer} on:keydown={handleActionMenuKeydown} />

<section class="now-playing-page">
    {#if playing}
        <section
            class="now-playing-track-section"
            aria-label="Now playing"
            on:contextmenu={(event) => onContextMenu(event, playing)}
        >
            <div class="now-playing-toolbar">
                <button
                    class="now-playing-back"
                    on:click={() => onNavigate("/player")}
                    aria-label="Back to home"><X size={18} /></button
                >
                {#if mobileViewport}
                    <div
                        bind:this={actionMenuRoot}
                        class="now-playing-actions-menu"
                        class:open={actionsOpen}
                        role="group"
                        aria-label="Song actions"
                    >
                        <button
                            bind:this={actionMenuTrigger}
                            type="button"
                            class="now-playing-actions-trigger"
                            aria-label="Song actions"
                            aria-haspopup="menu"
                            aria-expanded={actionsOpen}
                            aria-controls="now-playing-actions-menu"
                            title="Song actions"
                            on:click={() => (actionsOpen = !actionsOpen)}
                        ><ChevronDown size={18} /></button>
                        {#if actionsOpen}
                            <div
                                id="now-playing-actions-menu"
                                class="now-playing-actions-options"
                                role="menu"
                                aria-label="Song actions"
                            >
                                <button
                                    type="button"
                                    role="menuitem"
                                    on:click={() => {
                                        actionsOpen = false;
                                        onFavorite(playing);
                                    }}
                                >{isFavorite(playing) ? "Remove from liked songs" : "Add to liked songs"}</button>
                                <button
                                    type="button"
                                    role="menuitem"
                                    on:click={(event) => {
                                        actionsOpen = false;
                                        onContextMenu(event, playing);
                                    }}
                                >Add to playlist</button>
                            </div>
                        {/if}
                    </div>
                {:else}
                    <div class="now-playing-actions">
                        <button
                            class="player-tool"
                            class:active={isFavorite(playing)}
                            on:click={() => onFavorite(playing)}
                            aria-label={isFavorite(playing)
                                ? "Remove from liked songs"
                                : "Add to liked songs"}
                            aria-pressed={isFavorite(playing)}
                            title={isFavorite(playing)
                                ? "Remove from liked songs"
                                : "Add to liked songs"}
                            ><Heart
                                size={17}
                                fill={isFavorite(playing) ? "currentColor" : "none"}
                            /></button
                        >
                        <button
                            class="player-tool"
                            on:click={(event) => onContextMenu(event, playing)}
                            aria-label="Add to playlist"
                            title="Add to playlist"><Plus size={17} /></button
                        >
                    </div>
                {/if}
            </div>
            <div
                class:playing={isPlaying}
                class="now-playing-art"
                style={`--art-color:${playing.color || "#718979"}`}
            >
                <span class="vinyl-grooves" aria-hidden="true"></span>
                <span class="vinyl-label"
                    >{#if getTrackArt(playing)}<img
                            src={getTrackArt(playing)}
                            alt=""
                        />{:else}<Disc3 size={30} />{/if}</span
                >
                <span class="vinyl-spindle" aria-hidden="true"></span>
            </div>
            <div class="now-playing-copy">
                <h1 use:marquee={true}><span>{playing.title}</span></h1>
                <p>{playing.artist}</p>
                <small
                    >{playing.album || "Single"} · {getTrackSource(
                        playing,
                    )}</small
                >
            </div>
            <div class="now-playing-progress">
                <input
                    class="progress-slider"
                    type="range"
                    min="0"
                    max={duration || 1}
                    step="0.1"
                    value={currentTime}
                    style={`--progress:${duration ? Math.min(100, Math.max(0, (currentTime / duration) * 100)) : 0}%`}
                    on:input={onSeek}
                    aria-label="Track progress"
                    disabled={!canControlPlayback}
                />
                <div>
                    <span>{formatTime(currentTime)}</span><span
                        >{formatTime(duration || playing.length)}</span
                    >
                </div>
            </div>
            <div class="now-playing-controls">
                <button
                    class="player-tool"
                    class:active={shuffleOn}
                    on:click={onShuffle}
                    disabled={!canControlPlayback}
                    aria-label="Toggle shuffle"><Shuffle size={18} /></button
                >
                <button
                    class="icon-button"
                    on:click={onPrevious}
                    disabled={!canControlPlayback}
                    aria-label="Previous track"><SkipBack size={20} /></button
                >
                <button
                    class="play-main"
                    style={`--play-icon-color:${contrastColor(appAccent)}`}
                    on:click={onTogglePlayback}
                    disabled={!canControlPlayback}
                    aria-label="Play or pause"
                    ><svelte:component
                        this={isPlaying ? Pause : Play}
                        size={21}
                        fill="currentColor"
                    /></button
                >
                <button
                    class="icon-button"
                    on:click={onNext}
                    disabled={!canControlPlayback}
                    aria-label="Next track"><SkipForward size={20} /></button
                >
                <button
                    class="player-tool repeat-control"
                    class:active={repeatOn}
                    on:click={onToggleRepeat}
                    disabled={!canControlPlayback}
                    aria-label={`Repeat: ${repeatMode}`}
                    title={repeatMode === "off"
                        ? "Repeat all"
                        : repeatMode === "all"
                          ? "Repeat current song"
                          : "Turn repeat off"}
                    ><Repeat2 size={18} />{#if repeatMode === "one"}<span
                            class="repeat-one-mark">1</span
                        >{/if}</button
                >
            </div>
                    {#if mobileViewport}
                        <div class="mobile-now-playing-queue-action">
                            <button
                                type="button"
                                on:click={() => onNavigate("/queue")}
                                aria-label="Open queue"
                                ><ListMusic size={16} /><span>Queue</span></button
                            >
                        </div>
                    {/if}
        </section>
        <section
            class:lyrics-fullscreen={lyricsFullscreen}
            class="now-playing-extra-panel lyrics-panel"
            aria-labelledby="now-playing-lyrics-heading"
            style={`--lyrics-art-average:${lyricsBackgroundColor};background-color:var(--lyrics-art-average)`}
        >
            {#if !lyricsFullscreen}<button
                    type="button"
                    class="lyrics-preview-open"
                    on:click={onOpenLyricsFullscreen}
                    aria-label="Open fullscreen lyrics"
                    title="Open fullscreen lyrics"
                ></button>{/if}
            <header class="lyrics-heading">
                <div>
                    <h2 id="now-playing-lyrics-heading" use:marquee>
                        Lyrics
                    </h2>
                </div>
                {#if lyricsFullscreen}<button
                        class="lyrics-close"
                        on:click={onCloseLyricsFullscreen}
                        aria-label="Back to now playing"
                        title="Back to now playing"
                        ><ChevronLeft size={20} /></button
                    >{:else}<button
                        class="lyrics-expand"
                        on:click={onOpenLyricsFullscreen}
                        aria-label="Fullscreen lyrics"
                        title="Fullscreen lyrics"
                        ><Maximize2 size={17} /></button
                    >{/if}
            </header>
            <div
                class="lyrics-lines"
                bind:this={lyricsList}
                aria-live="off"
                role="button"
                tabindex="0"
                aria-label="Open fullscreen lyrics"
                title={mobileViewport
                    ? "Tap to open fullscreen lyrics"
                    : "Open fullscreen lyrics"}
                on:click={onOpenLyricsFullscreen}
                on:keydown={onLyricsLinesKeydown}
            >
                {#if syncedLyricLines.length}
                    {#each syncedLyricLines as line, index}<p
                            class:passed={activeLyricIndex >= 0 &&
                                index < activeLyricIndex}
                            class:active={index === activeLyricIndex}
                            class:upcoming={line.time !== null &&
                                (activeLyricIndex < 0 ||
                                    index > activeLyricIndex)}
                            class:untimed={line.time === null}
                        >
                            {line.text}
                        </p>{/each}
                {:else}
                    <p>{lyrics || "Loading lyrics…"}</p>
                {/if}
            </div>
        </section>
        <section
            class="now-playing-extra-panel song-info-panel"
            aria-labelledby="song-info-heading"
        >
            <div class="now-playing-extra-heading">
                <div>
                    <p class="eyebrow">Track details</p>
                    <h2 id="song-info-heading">Song information</h2>
                </div>
            </div>
            <dl class="song-info-list">
                <div>
                    <dt>Artist</dt>
                    <dd>{playing.artist || "Unknown artist"}</dd>
                </div>
                <div>
                    <dt>Album</dt>
                    <dd>{playing.album || "Single"}</dd>
                </div>
                <div>
                    <dt>BPM</dt>
                    <dd>{formatBpm(trackMetadata.bpm ?? playing.bpm ?? playing.tempo)}</dd>
                </div>
                <div>
                    <dt>Scale</dt>
                    <dd>{trackMetadata.scale || playing.scale || "—"}</dd>
                </div>
                <div>
                    <dt>Release date</dt>
                    <dd>{formatReleaseDate(trackMetadata.releaseDate || playing.releaseDate || playing.release_date)}</dd>
                </div>
                <div>
                    <dt>Duration</dt>
                    <dd>{formatTime(duration || playing.length)}</dd>
                </div>
            </dl>
        </section>
    {:else}
        <div class="now-playing-copy">
            <h1>Nothing playing</h1>
            <p>Choose a track from your library or queue.</p>
        </div>
    {/if}
</section>
