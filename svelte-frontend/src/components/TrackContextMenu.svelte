<script>
    export let contextMenu = null;
    export let playlists = [];
    export let spotifyPlaylists = [];
    export let preferredPlaylist = null;
    export let artistUrl = "";
    export let albumUrl = "";
    export let songUrl = "";
    export let isFavorite = false;
    export let downloadDisabled = false;
    export let onAddToPlaylist;
    export let onCreatePlaylist;
    export let onQueue;
    export let onToggleFavorite;
    export let onDownload;
    export let onOpenPage;
    export let onSongDetails;

    $: track = contextMenu?.track;
</script>

{#if contextMenu && track}
    <div
        class="context-menu"
        role="menu"
        aria-label={`Actions for ${track.title}`}
        style={`left:${contextMenu.x}px; top:${contextMenu.y}px; max-height:${contextMenu.maxHeight}px`}
    >
        <button on:click={() => onSongDetails(track)}>Song details</button>
        {#if songUrl}<button on:click={() => onOpenPage(songUrl)}
                >Open song on Spotify</button
            >{/if}
        {#if artistUrl}<button on:click={() => onOpenPage(artistUrl)}
                >Go to artist page</button
            >{/if}
        {#if albumUrl}<button on:click={() => onOpenPage(albumUrl)}
                >Go to album page</button
            >{/if}
        {#if preferredPlaylist || playlists.length || spotifyPlaylists.length}
            {#if preferredPlaylist}<button
                        class="context-menu-preferred"
                        on:click={() => onAddToPlaylist(track, preferredPlaylist)}
                        >Add to {preferredPlaylist.name}</button
                    >{/if}
            {#each [...playlists, ...spotifyPlaylists].filter((playlist) => playlist.id !== preferredPlaylist?.id) as playlist (playlist.id)}
                <button on:click={() => onAddToPlaylist(track, playlist)}
                    >{playlist.name}</button
                >
            {/each}
        {/if}
        <button on:click={() => onCreatePlaylist(track)}
            >Create new playlist</button
        >
        <button on:click={() => onQueue(track)}>Add to queue</button>
        <button on:click={() => onToggleFavorite(track)}
            >{isFavorite
                ? "Remove from liked songs"
                : "Add to liked songs"}</button
        >
        <button
            disabled={downloadDisabled}
            on:click={() => onDownload(track)}>Download song</button
        >
    </div>
{/if}
