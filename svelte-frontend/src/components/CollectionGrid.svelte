<script>
    import { Check, Copy, Disc3, Heart, ListMusic, Plus } from "lucide-svelte";
    import { marquee } from "../marquee.js";

    export let collections = [];
    export let albumGrid = false;
    export let getCollectionArt;
    export let isLikedSongsCollection;
    export let onOpen;
    export let onDuplicate;
    export let onQueue;
    export let onLike;
    export let onAddSongs = () => {};
</script>

<div
    class="collection-grid list-view"
    class:playlist-collection-grid={!albumGrid}
    class:album-collection-grid={albumGrid}
>
    {#each collections as collection (collection.id)}
        <article class="collection-card">
            <button
                class="collection-art"
                class:liked-songs-art={isLikedSongsCollection(collection)}
                style={`--art-color:${collection.color || "#718979"}`}
                on:click={() => onOpen(collection.id)}
                aria-label={`Open ${collection.name}`}
                title={`Open ${collection.name}`}
            >
                {#if isLikedSongsCollection(collection)}<Heart
                        size={20}
                        fill="currentColor"
                    />{:else if getCollectionArt(collection)}<img
                        src={getCollectionArt(collection)}
                        alt=""
                    />{:else}<Disc3 size={20} />{/if}
            </button>
            <button
                class="collection-copy collection-open"
                on:click={() => onOpen(collection.id)}
            >
                <strong use:marquee>{collection.name}</strong>
                <small use:marquee
                    >{collection.kind === "Spotify playlist"
                        ? "Spotify playlist"
                        : collection.kind === "Album"
                          ? "Album"
                          : "Wavelength playlist"} · {collection.tracks
                        ?.length || 0} songs</small
                >
            </button>
            {#if collection.kind !== "Album"}
                <div class="collection-actions">
                    {#if !isLikedSongsCollection(collection)}<button
                                class="row-action"
                                on:click={() => onAddSongs(collection)}
                                aria-label={`Add songs to ${collection.name}`}
                                title={`Add songs to ${collection.name}`}
                                ><Plus size={15} /></button
                            >{/if}
                        {#if collection.kind === "Spotify playlist" && collection.spotifyId}
                        <button
                            class="row-action"
                            disabled={collection.copiedToWavelength ||
                                !collection.tracks?.length}
                            on:click={() => onDuplicate(collection)}
                            aria-label={collection.copiedToWavelength
                                ? `Already copied ${collection.name} to Wavelength`
                                : `Copy ${collection.name} to Wavelength`}
                            title={collection.copiedToWavelength
                                ? "Already copied to Wavelength"
                                : "Copy to Wavelength"}
                        >
                            {#if collection.copiedToWavelength}<Check
                                    size={15}
                                />{:else}<Copy size={15} />{/if}
                        </button>
                    {/if}
                    <button
                        class="row-action"
                        on:click={() => onQueue(collection)}
                        aria-label={`Add ${collection.name} to queue`}
                        title={`Add ${collection.name} to queue`}
                        ><ListMusic size={15} /></button
                    >
                    {#if !isLikedSongsCollection(collection)}<button
                            class="row-action"
                            on:click={() => onLike(collection)}
                            aria-label={`Like ${collection.name}`}
                            title={`Like ${collection.name}`}
                            ><Heart size={15} /></button
                        >{/if}
                </div>
            {/if}
        </article>
    {/each}
</div>
