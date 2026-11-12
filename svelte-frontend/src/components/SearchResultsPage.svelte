<script>
    import { Check, Disc3, Play, Search } from "lucide-svelte";
    import SelectMenu from "../SelectMenu.svelte";
    import { marquee } from "../marquee.js";

    export let search = "";
    export let searchSource = "spotify";
    export let searchSourceOptions = [];
    export let searchResults = [];
    export let displayedTracks = [];
    export let searchLoading = false;
    export let searchLoadingMore = false;
    export let searchAtBottom = false;
    export let searchHasMore = false;
    export let mobileViewport = false;
    export let searchDisabled = false;
    export let searchTable = null;
    export let getTrackArt;
    export let onScroll;
    export let onMobileSearchSourceChange;
    export let onMobileSearchInput;
    export let onSearch;
    export let onPlay;
    export let onContextMenu;
    export let onPointerDown;
    export let onPointerUp;
    export let onPointerLeave;
    export let onClearContextMenuTimer;
</script>

{#if mobileViewport}
    <div class="search-source-select">
        <span>Search with</span>
        <SelectMenu
            bind:value={searchSource}
            options={searchSourceOptions}
            ariaLabel="Search with"
            className="mobile-search-source"
            disabled={searchDisabled}
            on:change={onMobileSearchSourceChange}
        />
    </div>
    <form
        class="search-page-form sticky-search"
        on:submit|preventDefault={onSearch}
    >
        <Search size={18} />
        <input
            type="search"
            bind:value={search}
            disabled={searchDisabled}
            on:input={onMobileSearchInput}
            placeholder={`Search ${searchSource} or paste a link...`}
            aria-label="Search songs"
        />
    </form>
{/if}

<header class="search-results-heading">
    <h1>Search results</h1>
    {#if search}<p>{searchSource === "youtube" ? "YouTube" : "Spotify"}</p>{/if}
</header>

{#if searchResults.length}
    <section
        bind:this={searchTable}
        on:scroll={onScroll}
        class="track-table"
        class:youtube-results={searchSource === "youtube"}
        class:spotify-results={searchSource === "spotify"}
        aria-label="Search results"
    >
        {#each displayedTracks as track (track.id)}
            <article
                class="track-row"
                aria-label={`Track ${track.title}`}
                on:contextmenu={(event) => onContextMenu(event, track)}
                on:pointerdown={(event) => onPointerDown(event, track)}
                on:pointerup={(event) => {
                    onClearContextMenuTimer();
                    onPointerUp(event, track);
                }}
                on:pointerleave={onPointerLeave}
            >
                <button
                    class="track-select"
                    disabled={searchDisabled}
                    on:click={() => onPlay(track)}
                    aria-label={`Play ${track.title}`}
                >
                    <span class="track-name">
                        <span class="mini-art"
                            >{#if getTrackArt(track)}<img
                                    src={getTrackArt(track)}
                                    alt=""
                                />{:else}<span class="mini-art-fallback"
                                    ><Disc3 size={16} /></span
                                >{/if}</span
                        >
                        <span class="track-copy"
                            ><strong use:marquee>{track.title}</strong><small
                                use:marquee>{track.artist}</small
                            ></span
                        >
                    </span>
                    <span class="album-name">{track.album || "Single"}</span>
                    <span class="track-length">{track.length || "—"}</span>
                </button>
                <span class="track-actions">
                    <button
                        class="row-action play-row-action"
                        disabled={searchDisabled}
                        on:click={(event) => {
                            event.stopPropagation();
                            onPlay(track);
                        }}
                        aria-label={`Play ${track.title} now`}
                        title="Play now"><Play size={15} /></button
                    >
                </span>
            </article>
        {/each}
        {#if searchAtBottom}
            {#if searchLoading || searchLoadingMore}
                <p class="search-results-status is-loading" role="status">
                    Loading more results…
                </p>
            {:else if searchResults.length && !searchHasMore}
                <p class="search-results-status">
                    <span class="search-results-status-rule" aria-hidden="true"
                    ></span><Check size={14} aria-hidden="true" /><span
                        >You’re all caught up.</span
                    ><span class="search-results-status-rule" aria-hidden="true"
                    ></span>
                </p>
            {/if}
        {/if}
    </section>
{:else if search.trim()}
    <p class="search-results-empty">
        {searchLoading || searchLoadingMore
            ? "Loading results…"
            : "No results found."}
    </p>
{:else}
    <p class="search-results-empty">
        {mobileViewport
            ? "Search songs, artists, or albums above."
            : "Search songs, artists, or albums from the top search bar."}
    </p>
{/if}
