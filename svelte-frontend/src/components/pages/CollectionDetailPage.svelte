<script>
    import { ChevronLeft, Disc3, Heart, ListMusic, Plus } from "lucide-svelte";
    import { marquee } from "../../marquee.js";

    export let selectedCollection = null;
    export let getTrackArt;
    export let isFavorite;
    export let isLikedSongsCollection;
    export let onNavigate;
    export let onPlay;
    export let onQueue;
    export let onFavorite;
    export let onContextMenu;
    export let onAddSongs;
</script>

{#if selectedCollection}
    <section class="collection-detail-page">
        <header
            class="collection-detail-heading"
            class:liked-songs-detail={isLikedSongsCollection(
                selectedCollection,
            )}
        >
            {#if !isLikedSongsCollection(selectedCollection)}<button
                    class="icon-button"
                    on:click={() => onNavigate("/library")}
                    aria-label="Back to library"
                    title="Back to library"><ChevronLeft size={18} /></button
                >{/if}
            <div class="collection-detail-title">
                <h1>{selectedCollection.name}</h1>
                {#if !isLikedSongsCollection(selectedCollection)}<p
                        class="eyebrow"
                    >
                        {selectedCollection.kind} · {selectedCollection.tracks
                            ?.length || 0} songs
                    </p>{/if}
            </div>
            {#if selectedCollection.kind !== "Album" && !isLikedSongsCollection(selectedCollection)}<button
                    class="outline-button compact-action collection-add-songs"
                    on:click={() => onAddSongs(selectedCollection)}
                    ><Plus size={15} /> Add songs</button
                >{/if}
            {#if isLikedSongsCollection(selectedCollection)}<p
                    class="collection-detail-meta"
                >
                    {selectedCollection.kind} · {selectedCollection.tracks
                        ?.length || 0} songs
                </p>{/if}
        </header>
        {#if selectedCollection.tracks?.length}
            <div class="imported-track-list">
                {#each selectedCollection.tracks as track (track.id)}
                    <article
                        class="imported-track-row"
                        on:contextmenu={(event) => onContextMenu(event, track)}
                    >
                        <button
                            class="imported-track-select"
                            on:click={() => onPlay(track)}
                            aria-label={`Play ${track.title}`}
                            ><span
                                class="mini-art"
                                style={`--art-color:${track.color || "#718979"}`}
                                >{#if getTrackArt(track)}<img
                                        src={getTrackArt(track)}
                                        alt=""
                                    />{:else}<Disc3 size={18} />{/if}</span
                            ><span class="imported-track-copy"
                                ><strong use:marquee>{track.title}</strong
                                ><small use:marquee
                                    >{track.artist} · {track.album ||
                                        selectedCollection.name}</small
                                ></span
                            ></button
                        >
                        <span class="imported-track-duration"
                            >{track.length || "—"}</span
                        >
                        <button
                            class="row-action imported-queue-action"
                            on:click={() => onQueue(track)}
                            aria-label={`Add ${track.title} to queue`}
                            title="Add to queue"><ListMusic size={15} /></button
                        >
                        <button
                            class="row-action imported-favorite-action"
                            class:active={isFavorite(track)}
                            on:click={() => onFavorite(track)}
                            aria-label={isFavorite(track)
                                ? `Remove ${track.title} from liked songs`
                                : `Like ${track.title}`}
                            aria-pressed={isFavorite(track)}
                            title={isFavorite(track)
                                ? "Remove from liked songs"
                                : "Add to liked songs"}
                            ><Heart
                                size={15}
                                fill={isFavorite(track)
                                    ? "currentColor"
                                    : "none"}
                            /></button
                        >
                    </article>
                {/each}
            </div>
        {:else}
            <p class="settings-empty">
                This collection has no available tracks.
            </p>
        {/if}
    </section>
{:else}
    <section class="collection-detail-page">
        <header class="collection-detail-heading">
            <button
                class="icon-button"
                on:click={() => onNavigate("/library")}
                aria-label="Back to library"
                title="Back to library"><ChevronLeft size={18} /></button
            >
            <h1>Collection not found</h1>
        </header>
    </section>
{/if}
