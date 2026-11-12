<script>
    import {
        ArrowRight,
        ChevronLeft,
        ChevronRight,
        Disc3,
        FolderOpen,
        Heart,
        ListMusic,
        Search,
    } from "lucide-svelte";
    import { marquee } from "../../marquee.js";
    import SelectMenu from "../../SelectMenu.svelte";

    export let libraryCollections = [];
    export let playHistory = [];
    export let importedTracks = [];
    export let artistStats = [];
    export let songStats = [];
    export let statsPeriod = 'all';
    export let statsPeriodOptions = [];
    export let recentlyAdded = [];
    export let downloadFolderReady = false;
    export let downloadDirectoryHandle = null;
    export let recentAddedGrid = null;
    export let getCollectionArt;
    export let getTrackArt;
    export let getTrackSource;
    export let isFavorite;
    export let onChooseDownloadDirectory;
    export let onNavigate;
    export let onOpenCollection;
    export let onPlay;
    export let onQueue;
    export let onFavorite;
    export let onContextMenu;
    export let onScrollRecentlyAdded;
</script>

<section class="home-greeting">
    <h1>Your music, within reach.</h1>
    <p>Search, play, and shape what comes next from your local library.</p>
</section>

{#if !downloadFolderReady}
    <aside class="home-download-folder-reminder" role="note">
        <p>Choose a download destination to enable search and downloads.</p>
        <button type="button" on:click={onChooseDownloadDirectory}>
            <FolderOpen size={16} />Choose download destination
        </button>
    </aside>
{/if}

{#if !playHistory.length && !importedTracks.length && downloadFolderReady}
    <button
        class="home-explore-button"
        on:click={() => onNavigate("/search")}
        ><Search size={18} /><span class="home-explore-copy"
            >Find something new to play<span
                class="animated-dots"
                aria-hidden="true"><i>.</i><i>.</i><i>.</i></span
            ></span
        ><ChevronRight size={17} /></button
    >
{/if}

{#if libraryCollections.some((collection) => collection.name !== "Liked Songs" && collection.tracks?.length && ["Playlist", "Spotify playlist", "Webapp playlist", "Album"].includes(collection.kind))}
    <section class="home-collections home-section">
        <div class="section-head">
            <div>
                <p class="eyebrow">Your library</p>
                <h2>Recent playlists and albums</h2>
            </div>
            <button class="text-button" on:click={() => onNavigate("/library")}
                >View library</button
            >
        </div>
        <div class="home-collection-grid">
            {#each libraryCollections
                .filter((collection) => collection.name !== "Liked Songs" && ["Playlist", "Spotify playlist", "Webapp playlist", "Album"].includes(collection.kind))
                .slice(0, 8) as collection (collection.id)}
                <button
                    class="home-collection-card"
                    on:click={() => onOpenCollection(collection.id)}
                    ><span
                        class="mini-art"
                        style={`--art-color:${collection.color || "#718979"}`}
                        >{#if getCollectionArt(collection)}<img
                                src={getCollectionArt(collection)}
                                alt=""
                            />{:else if collection.kind === "Album"}<Disc3
                                size={20}
                            />{:else}<ListMusic size={20} />{/if}</span
                    ><span
                        ><strong>{collection.name}</strong><small
                            >{collection.kind} · {collection.tracks?.length ||
                                0} songs</small
                        ></span
                    ></button
                >
            {/each}
        </div>
    </section>
{/if}

{#if recentlyAdded.length}
    <section class="home-section">
        <div class="section-head">
            <div class="recent-added-title">
                <h2>Recently added</h2>
                <button
                    class="icon-button recent-added-library-link"
                    on:click={() => onNavigate("/library")}
                    aria-label="Open local library"
                    title="Open local library"><ArrowRight size={17} /></button
                >
            </div>
            <div class="recent-added-actions">
                <button
                    class="icon-button recent-added-step"
                    on:click={() => onScrollRecentlyAdded(-1)}
                    aria-label="Scroll recently added left"
                    title="Previous items"><ChevronLeft size={17} /></button
                ><button
                    class="icon-button recent-added-step"
                    on:click={() => onScrollRecentlyAdded(1)}
                    aria-label="Scroll recently added right"
                    title="Next items"><ChevronRight size={17} /></button
                >
            </div>
        </div>
        <div bind:this={recentAddedGrid} class="recent-added-grid">
            {#each recentlyAdded as track (track.id)}
                <article
                    class="recent-added-card"
                    on:contextmenu={(event) => onContextMenu(event, track)}
                >
                    <button
                        class="recent-added-play"
                        on:click={() => onPlay(track)}
                        aria-label={`Play ${track.title}`}
                        title={`Play ${track.title}`}
                    >
                        <span
                            class="mini-art"
                            style={`--art-color:${track.color || "#718979"}`}
                            >{#if getTrackArt(track)}<img
                                    src={getTrackArt(track)}
                                    alt=""
                                />{:else}<Disc3 size={22} />{/if}</span
                        >
                        <span class="recent-added-copy"
                            ><strong use:marquee>{track.title}</strong><small
                                use:marquee
                                >{track.artist} · {getTrackSource(track)}</small
                            ></span
                        >
                    </button>
                    <button
                        class="recent-added-queue"
                        on:click={() => onQueue(track)}
                        aria-label={`Add ${track.title} to queue`}
                        title="Add to queue"><ListMusic size={16} /></button
                    >
                    <button
                        class="recent-added-like"
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
                            size={16}
                            fill={isFavorite(track) ? "currentColor" : "none"}
                        /></button
                    >
                </article>
            {/each}
        </div>
    </section>
{/if}

{#if playHistory.length}
    <section class="home-stats" aria-labelledby="home-stats-title">
        <div class="section-head">
            <div><h2 id="home-stats-title">Listening Stats</h2></div>
               <SelectMenu bind:value={statsPeriod} options={statsPeriodOptions} ariaLabel="Listening stats period" className="stats-period-select" />
        </div>
        <div class="stats-grid">
            <section class="stat-panel">
                <h3>Top artists</h3>
                {#if artistStats.length}{#each artistStats as item, index}<div
                            class="stat-row"
                        >
                            <span>{String(index + 1).padStart(2, "0")}</span
                            ><strong>{item[0]}</strong><b>{item[1]}</b>
                        </div>{/each}{:else}<p class="settings-empty">
                        Play music to build your stats.
                    </p>{/if}
            </section>
            <section class="stat-panel">
                <h3>Top tracks</h3>
                {#if songStats.length}{#each songStats as item, index}<div
                            class="stat-row"
                        >
                            <span>{String(index + 1).padStart(2, "0")}</span
                            ><strong>{item.track.title}</strong><b
                                >{item.listens}</b
                            >
                        </div>{/each}{:else}<p class="settings-empty">
                        Your most-played tracks will appear here.
                    </p>{/if}
            </section>
        </div>
    </section>
{/if}
