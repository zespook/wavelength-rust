<script>
    import {
        ChevronDown,
        ChevronRight,
        Disc3,
        FolderOpen,
        Heart,
        Plus,
        Search,
        Users,
        X,
        ListMusic,
    } from "lucide-svelte";
    import SelectMenu from "../../SelectMenu.svelte";
    import CollectionGrid from "../CollectionGrid.svelte";
    import { marquee } from "../../marquee.js";

    export let mobileViewport = false;
    export let libraryFilter = "imported";
    export let libraryCollections = [];
    export let filteredImportedTracks = [];
    export let importedSearch = "";
    export let importedSortBy = "folders";
    export let importedSortOptions = [];
    export let importedFolderTree = [];
    export let importedBrowseGroups = [];
    export let getCollectionArt;
    export let getTrackArt;
    export let getTrackSource;
    export let isFavorite;
    export let isLikedSongsCollection;
    export let onOpenFolderManager;
    export let onToggleImportedFolder;
    export let onPlay;
    export let onQueue;
    export let onFavorite;
    export let onContextMenu;
    export let onCreatePlaylist;
    export let onOpenCollection;
    export let onDuplicateSpotifyPlaylist;
    export let onLikeTracks;
    export let onAddSongs;
</script>

<header class="library-page-heading">
    <h1>
        {mobileViewport && libraryFilter === "online"
            ? "Online library"
            : "Local library"}
    </h1>
    {#if !mobileViewport}<button
            class="outline-button compact-action"
            on:click={onOpenFolderManager}
            ><FolderOpen size={15} /> Manage folders</button
        >{/if}
</header>

{#if mobileViewport}
    <div class="mobile-library-toolbar">
        <div
            class="library-page-filters mobile-library-source"
            role="tablist"
            aria-label="Library source"
        >
            <button
                class:active={libraryFilter === "online"}
                on:click={() => (libraryFilter = "online")}
                role="tab"
                aria-selected={libraryFilter === "online"}>Online</button
            >
            <button
                class:active={libraryFilter === "imported"}
                on:click={() => (libraryFilter = "imported")}
                role="tab"
                aria-selected={libraryFilter === "imported"}>Offline</button
            >
        </div>
        {#if libraryFilter !== "online"}<button
                class="outline-button compact-action mobile-library-manage-folders"
                on:click={onOpenFolderManager}
                ><FolderOpen size={15} /> Manage folders</button
            >{/if}
    </div>
{/if}

{#if libraryFilter === "imported"}
    <section
        class="imported-library"
        aria-labelledby="imported-library-heading"
    >
        <header class="imported-library-heading">
            <h2 id="imported-library-heading">Imported music</h2>
            <span class="imported-track-count"
                >{filteredImportedTracks.length} tracks</span
            >
        </header>
        <div class="imported-library-tools">
            <label class="imported-search"
                ><Search size={16} /><input
                    bind:value={importedSearch}
                    placeholder="Search title, artist, or album"
                    aria-label="Search imported music"
                />{#if importedSearch}<button
                        type="button"
                        class="imported-search-clear"
                        aria-label="Clear imported search"
                        on:click={() => (importedSearch = "")}
                        ><X size={15} /></button
                    >{/if}</label
            >
            <div class="imported-sort-label">
                <SelectMenu
                    bind:value={importedSortBy}
                    options={importedSortOptions}
                    ariaLabel="Sort imported tracks by"
                    className="imported-sort-select"
                />
            </div>
        </div>
        {#if importedSortBy === "folders"}
            {#if importedFolderTree.length}
                <div class="imported-folder-list">
                    {#each importedFolderTree as folder (folder.path)}
                        <section
                            class="imported-folder-section"
                            style={`--folder-indent:${folder.depth * 16}px`}
                        >
                            <button
                                class="imported-folder-heading"
                                on:click={() =>
                                    onToggleImportedFolder(
                                        folder.path,
                                        folder.depth,
                                    )}
                                aria-expanded={folder.expanded}
                            >
                                {#if folder.expanded}<ChevronDown
                                        size={15}
                                    />{:else}<ChevronRight
                                        size={15}
                                    />{/if}<FolderOpen size={16} /><span
                                    class="imported-folder-name"
                                    >{folder.name}</span
                                ><span>{folder.trackCount}</span>
                            </button>
                            {#if folder.expanded}
                                <div class="imported-track-list">
                                    {#each folder.groups as group (group.name)}
                                        {#each group.tracks as track (track.id)}
                                            <article
                                                class="imported-track-row"
                                                on:contextmenu={(event) => onContextMenu(event, track)}
                                            >
                                                <button
                                                    class="imported-track-select"
                                                    on:click={() =>
                                                        onPlay(track)}
                                                    aria-label={`Play ${track.title}`}
                                                >
                                                    <span
                                                        class="mini-art"
                                                        style={`--art-color:${track.color || "#718979"}`}
                                                        >{#if getTrackArt(track)}<img
                                                                src={getTrackArt(
                                                                    track,
                                                                )}
                                                                alt=""
                                                            />{:else}<Disc3
                                                                size={18}
                                                            />{/if}</span
                                                    >
                                                    <span
                                                        class="imported-track-copy"
                                                        ><strong use:marquee
                                                            >{track.title}</strong
                                                        ><small use:marquee
                                                            >{track.artist ||
                                                                "Unknown artist"}
                                                            · {getTrackSource(
                                                                track,
                                                            )}</small
                                                        ></span
                                                    >
                                                </button>
                                                <span
                                                    class="imported-track-duration"
                                                    >{track.length || "—"}</span
                                                >
                                                <button
                                                    class="row-action imported-queue-action"
                                                    on:click={() =>
                                                        onQueue(track)}
                                                    aria-label={`Add ${track.title} to queue`}
                                                    title="Add to queue"
                                                    ><ListMusic
                                                        size={15}
                                                    /></button
                                                >
                                                <button
                                                    class="row-action imported-favorite-action"
                                                    class:active={isFavorite(
                                                        track,
                                                    )}
                                                    on:click={() =>
                                                        onFavorite(track)}
                                                    aria-label={isFavorite(
                                                        track,
                                                    )
                                                        ? `Remove ${track.title} from liked songs`
                                                        : `Like ${track.title}`}
                                                    aria-pressed={isFavorite(
                                                        track,
                                                    )}
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
                                    {/each}
                                </div>
                            {/if}
                        </section>
                    {/each}
                </div>
            {:else}
                <div class="imported-empty">
                    <FolderOpen size={20} />
                    <p>
                        {importedSearch
                            ? "No imported tracks match this search."
                            : "No imported tracks detected yet."}
                    </p>
                </div>
            {/if}
        {:else if importedBrowseGroups.length}
            <div class="imported-folder-list">
                {#each importedBrowseGroups as group (group.path)}
                    <section class="imported-folder-section">
                        <button
                            class="imported-folder-heading"
                            on:click={() =>
                                onToggleImportedFolder(group.path, 0)}
                            aria-expanded={group.expanded}
                        >
                            {#if group.expanded}<ChevronDown
                                    size={15}
                                />{:else}<ChevronRight
                                    size={15}
                                />{/if}{#if importedSortBy === "album"}<Disc3
                                    size={16}
                                />{:else}<Users size={16} />{/if}<span
                                class="imported-folder-name">{group.name}</span
                            ><span>{group.trackCount}</span>
                        </button>
                        {#if group.expanded}
                            <div class="imported-track-list">
                                {#each group.tracks as track (track.id)}
                                    <article
                                        class="imported-track-row"
                                        on:contextmenu={(event) => onContextMenu(event, track)}
                                    >
                                        <button
                                            class="imported-track-select"
                                            on:click={() => onPlay(track)}
                                            aria-label={`Play ${track.title}`}
                                        >
                                            <span
                                                class="mini-art"
                                                style={`--art-color:${track.color || "#718979"}`}
                                                >{#if getTrackArt(track)}<img
                                                        src={getTrackArt(track)}
                                                        alt=""
                                                    />{:else}<Disc3
                                                        size={18}
                                                    />{/if}</span
                                            >
                                            <span class="imported-track-copy"
                                                ><strong use:marquee
                                                    >{track.title}</strong
                                                ><small use:marquee
                                                    >{importedSortBy === "album"
                                                        ? track.artist ||
                                                          "Unknown artist"
                                                        : track.album ||
                                                          "Single"} · {getTrackSource(
                                                        track,
                                                    )}</small
                                                ></span
                                            >
                                        </button>
                                        <span class="imported-track-duration"
                                            >{track.length || "—"}</span
                                        >
                                        <button
                                            class="row-action imported-queue-action"
                                            on:click={() => onQueue(track)}
                                            aria-label={`Add ${track.title} to queue`}
                                            title="Add to queue"
                                            ><ListMusic size={15} /></button
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
                        {/if}
                    </section>
                {/each}
            </div>
        {:else}
            <div class="imported-empty">
                <FolderOpen size={20} />
                <p>
                    {importedSearch
                        ? "No imported tracks match this search."
                        : "No imported tracks detected yet."}
                </p>
            </div>
        {/if}
    </section>
{:else}
    <section class="library-collections">
        <div class="section-head library-collection-heading">
            <div>
                <h2>Playlists and albums</h2>
            </div>
            <button
                class="icon-button create-playlist-button"
                on:click={onCreatePlaylist}
                aria-label="Create playlist"
                title="Create playlist"><Plus size={18} /></button
            >
        </div>
        <CollectionGrid
            collections={libraryCollections.filter(
                (collection) =>
                    [
                        "Playlist",
                        "Spotify playlist",
                        "Webapp playlist",
                    ].includes(collection.kind) ||
                    collection.name === "Liked Songs",
            )}
            {getCollectionArt}
            {isLikedSongsCollection}
            onOpen={onOpenCollection}
            onDuplicate={onDuplicateSpotifyPlaylist}
            onQueue={(collection) => collection.tracks?.forEach(onQueue)}
            onLike={(collection) => onLikeTracks(collection.tracks || [])}
            onAddSongs={onAddSongs}
        />
        <div class="section-head library-albums-heading">
            <div>
                <p class="eyebrow">Albums</p>
                <h2>Saved albums</h2>
            </div>
        </div>
        <CollectionGrid
            albumGrid={true}
            collections={libraryCollections.filter(
                (collection) => collection.kind === "Album",
            )}
            {getCollectionArt}
            {isLikedSongsCollection}
            onOpen={onOpenCollection}
            onDuplicate={onDuplicateSpotifyPlaylist}
            onQueue={() => {}}
            onLike={() => {}}
        />
    </section>
{/if}
