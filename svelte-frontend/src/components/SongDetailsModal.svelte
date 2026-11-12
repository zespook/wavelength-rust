<script>
    import { Disc3, Link, X } from "lucide-svelte";

    export let track = null;
    export let getTrackArt;
    export let getTrackSource;
    export let onClose;
    export let onOpenPage;

    $: durationLabel =
        track?.length ||
        (track?.duration
            ? `${Math.floor(track.duration / 60)}:${String(Math.floor(track.duration % 60)).padStart(2, "0")}`
            : "—");
</script>

{#if track}
    <div
        class="modal-backdrop content-area-backdrop song-details-backdrop"
        role="presentation"
        on:click={(event) => {
            if (event.target === event.currentTarget) onClose();
        }}
    >
        <div
            class="modal song-details-modal"
            role="dialog"
            aria-modal="true"
            aria-labelledby="song-details-title"
        >
            <button
                type="button"
                class="modal-close"
                on:click={onClose}
                aria-label="Close song details"><X size={18} /></button
            >
            <div class="song-details-identity">
                <div class="song-details-art mini-art">
                    {#if getTrackArt(track)}<img
                            src={getTrackArt(track)}
                            alt=""
                        />{:else}<Disc3 size={24} />{/if}
                </div>
                <div class="song-details-copy">
                    <p class="eyebrow">{getTrackSource(track)}</p>
                    <h2 id="song-details-title">
                        {track.title || "Unknown track"}
                    </h2>
                    <p>{track.artist || "Unknown artist"}</p>
                </div>
            </div>
            <dl class="song-details-list">
                <div>
                    <dt>Album</dt>
                    <dd>{track.album || "Single"}</dd>
                </div>
                <div>
                    <dt>Duration</dt>
                    <dd>{durationLabel}</dd>
                </div>
            </dl>
            {#if track.spotify_url}<button
                    class="outline-button compact-action song-details-open"
                    on:click={() => onOpenPage(track.spotify_url)}
                    ><Link size={15} />Open on Spotify</button
                >{/if}
        </div>
    </div>
{/if}
