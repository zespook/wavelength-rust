<script>
    import {
        Disc3,
        Heart,
        ListMusic,
        Mic2,
        Pause,
        Play,
        Repeat2,
        Shuffle,
        SkipBack,
        SkipForward,
        Users,
        Volume2,
        VolumeX,
    } from "lucide-svelte";
    import { marquee } from "../marquee.js";

    export let playing = null;
    export let isPlaying = false;
    export let isFavorite;
    export let getTrackArt;
    export let getTrackSource;
    export let shuffleOn = false;
    export let repeatMode = "off";
    export let repeatOn = false;
    export let appAccent = "";
    export let contrastColor;
    export let currentTime = 0;
    export let duration = 0;
    export let volume = 0;
    export let isMuted = false;
    export let volumeSliderOpen = false;
    export let mobileViewport = false;
    export let lyricsFullscreen = false;
    export let railView = null;
    export let joinedParty = "";
    export let partyMembers = [];
    export let canControlPlayback = true;
    export let formatTime;
    export let onOpenNowPlaying;
    export let onFavorite;
    export let onContextMenu;
    export let onShuffle;
    export let onPrevious;
    export let onTogglePlayback;
    export let onNext;
    export let onRepeat;
    export let onSeek;
    export let onToggleLyrics;
    export let onOpenPartySettings;
    export let onShowQueue;
    export let onVolumeButtonClick;
    export let onVolumeInput;
</script>

<footer class="player-bar">
    <div
        class="player-identity"
        role="group"
        on:contextmenu={(event) => playing && onContextMenu(event, playing)}
    >
        <div
            class:empty={!playing}
            class="vinyl-player"
            role="button"
            tabindex={playing ? 0 : -1}
            aria-disabled={!playing}
            on:click={onOpenNowPlaying}
            on:keydown={(event) => event.key === "Enter" && onOpenNowPlaying()}
            aria-label={playing ? "Open now playing" : "Nothing playing"}
        >
            <div
                class="player-art"
                style={`--art-color:${playing?.color || "#718979"}`}
                aria-hidden="true"
            >
                {#if playing && getTrackArt(playing)}<img
                        src={getTrackArt(playing)}
                        alt=""
                    />{:else}<Disc3 size={19} />{/if}
            </div>
            <div class="player-track">
                {#if playing}<strong use:marquee={true}
                        ><span>{playing.title}</span></strong
                    ><small use:marquee={true}
                        ><span>{playing.artist} · {getTrackSource(playing)}</span></small
                    >{:else}<strong>Nothing playing</strong><small
                        >Select music to begin</small
                    >{/if}
            </div>
        </div>
        <button
            class="player-like-button"
            class:active={playing && isFavorite(playing)}
            style={`--player-control-color:${playing && isFavorite(playing) ? "var(--lime)" : "var(--muted)"}`}
            on:click={() => onFavorite(playing)}
            aria-label={playing
                ? isFavorite(playing)
                    ? "Remove from liked songs"
                    : "Add to liked songs"
                : "Like now playing song"}
            aria-pressed={Boolean(playing && isFavorite(playing))}
            title={playing
                ? isFavorite(playing)
                    ? "Remove from liked songs"
                    : "Add to liked songs"
                : "Nothing playing"}
            disabled={!playing}
            ><Heart
                size={17}
                fill={playing && isFavorite(playing) ? "currentColor" : "none"}
            /></button
        >
    </div>
    <div class="player-center">
        <div class="transport">
            <button
                class="player-tool"
                class:active={shuffleOn}
                on:click={onShuffle}
                aria-label="Toggle shuffle"
                title="Shuffle"
                disabled={!playing || !canControlPlayback}><Shuffle size={16} /></button
            >
            <button
                class="icon-button"
                on:click={onPrevious}
                aria-label="Previous track"
                disabled={!playing || !canControlPlayback}><SkipBack size={16} /></button
            >
            <button
                class="play-main"
                style={`--play-icon-color:${contrastColor(appAccent)}`}
                on:click={onTogglePlayback}
                aria-label="Play or pause"
                disabled={!playing || !canControlPlayback}
                ><svelte:component
                    this={isPlaying ? Pause : Play}
                    size={17}
                    fill="currentColor"
                /></button
            >
            <button
                class="icon-button"
                on:click={onNext}
                aria-label="Next track"
                disabled={!playing || !canControlPlayback}><SkipForward size={16} /></button
            >
            <button
                class="player-tool repeat-control"
                class:active={repeatOn}
                on:click={onRepeat}
                aria-label={`Repeat: ${repeatMode}`}
                title={repeatMode === "off"
                    ? "Repeat all"
                    : repeatMode === "all"
                      ? "Repeat current song"
                      : "Turn repeat off"}
                disabled={!playing || !canControlPlayback}
                ><Repeat2 size={16} />{#if repeatMode === "one"}<span
                        class="repeat-one-mark">1</span
                    >{/if}</button
            >
        </div>
        <div class="progress-row">
            <span>{formatTime(playing ? currentTime : 0)}</span><input
                class="progress-slider"
                type="range"
                min="0"
                max={duration || 1}
                step="0.1"
                value={playing ? currentTime : 0}
                style={`--progress:${playing && duration ? Math.min(100, Math.max(0, (currentTime / duration) * 100)) : 0}%`}
                on:input={onSeek}
                aria-label="Track progress"
                disabled={!playing || !canControlPlayback}
            /><span>{formatTime(playing ? duration || playing.length : 0)}</span
            >
        </div>
    </div>
    {#if !mobileViewport}
        <div class="player-tools">
            <button
                class="player-tool"
                class:active={lyricsFullscreen}
                style={`--player-control-color:${lyricsFullscreen ? "var(--lime)" : "var(--muted)"}`}
                on:click={onToggleLyrics}
                aria-label="Toggle lyrics"
                aria-pressed={lyricsFullscreen}
                title={lyricsFullscreen ? "Hide lyrics" : "Show lyrics"}
                disabled={!playing}><Mic2 size={17} /></button
            >
            <button
                class="party-roster-button"
                class:active={railView === "party"}
                style={`--player-control-color:${railView === "party" ? "var(--lime)" : "var(--muted)"}`}
                on:click={onOpenPartySettings}
                aria-label={joinedParty
                    ? `Party ${joinedParty}, ${partyMembers.length} listeners`
                    : "Open party details"}
                aria-pressed={railView === "party"}
                title={joinedParty ? `Room ${joinedParty}` : "Party details"}
                ><Users size={17} /></button
            >
            <button
                class="player-tool"
                class:active={railView === "queue"}
                style={`--player-control-color:${railView === "queue" ? "var(--lime)" : "var(--muted)"}`}
                on:click={onShowQueue}
                aria-label="Show queue"
                aria-pressed={railView === "queue"}
                title="Queue"><ListMusic size={17} /></button
            >
            <div class:open={volumeSliderOpen} class="volume-control">
                <button
                    class="volume-button"
                    style={`color:${isMuted || volume === 0 ? "var(--muted)" : "var(--lime)"}`}
                    on:click={onVolumeButtonClick}
                    aria-label={isMuted || volume === 0 ? "Unmute" : "Mute"}
                    title={isMuted || volume === 0 ? "Unmute" : "Mute"}
                    ><svelte:component
                        this={isMuted || volume === 0 ? VolumeX : Volume2}
                        size={16}
                    /></button
                >
                <input
                    class="volume-slider"
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    value={volume}
                    style={`--volume:${volume * 100}%`}
                    on:input={onVolumeInput}
                    aria-label="Volume"
                />
            </div>
        </div>
    {/if}
</footer>
