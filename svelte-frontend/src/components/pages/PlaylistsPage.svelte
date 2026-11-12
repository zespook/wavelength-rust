<script>
    import { ListMusic, Plus } from "lucide-svelte";
    import CollectionGrid from "../CollectionGrid.svelte";

    export let savedPlaylistCollections = [];
    export let getCollectionArt;
    export let isLikedSongsCollection;
    export let onCreatePlaylist;
    export let onOpenCollection;
    export let onDuplicateSpotifyPlaylist;
    export let onQueueCollection;
    export let onLikeTracks;
    export let onAddSongs;
</script>

<section class="playlists-page" aria-labelledby="playlists-page-heading">
    <header class="library-page-heading">
        <div><h1 id="playlists-page-heading">Playlists</h1></div>
        <button
            class="icon-button create-playlist-button"
            on:click={onCreatePlaylist}
            aria-label="Create playlist"
            title="Create playlist"><Plus size={18} /></button
        >
    </header>
    {#if savedPlaylistCollections.length}
        <CollectionGrid
            collections={savedPlaylistCollections}
            {getCollectionArt}
            {isLikedSongsCollection}
            onOpen={onOpenCollection}
            onDuplicate={onDuplicateSpotifyPlaylist}
            onQueue={onQueueCollection}
            onLike={(collection) => onLikeTracks(collection.tracks || [])}
            onAddSongs={onAddSongs}
        />
    {:else}
        <div class="imported-empty">
            <ListMusic size={20} />
            <p>No playlists yet.</p>
            <button class="outline-button" on:click={onCreatePlaylist}
                >Create playlist</button
            >
        </div>
    {/if}
</section>
