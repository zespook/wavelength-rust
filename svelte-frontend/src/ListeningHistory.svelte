<script>
    import { Clock3, Disc3, Heart, ListMusic } from "lucide-svelte";

    export let playHistory = [];
    export let getTrackArt = () => "";
    export let getTrackSource = () => "Local track";
    export let isFavorite = () => false;
    export let onPlay = () => {};
    export let onQueue = () => {};
    export let onToggleFavorite = () => {};
    export let onContextMenu = () => {};
    export let onClear = () => {};
</script>

<section class="history-page" aria-label="Listening history">
    <header class="library-page-heading history-page-heading">
        <h1>Listening history</h1>
        {#if playHistory.length}<button
                class="outline-button compact-action"
                on:click={onClear}>Clear history</button
            >{/if}
    </header>

    {#if playHistory.length}
        <div
            class="history-list collection-grid playlist-collection-grid list-view"
        >
            {#each playHistory as item}
                <article
                    class="collection-card history-card"
                    on:contextmenu={(event) => onContextMenu(event, item)}
                >
                    <button
                        class="collection-art history-art"
                        style={`--art-color:${item.color || "#718979"}`}
                        on:click={() => onPlay(item)}
                        aria-label={`Play ${item.title || "track"}`}
                    >
                        {#if getTrackArt(item)}<img
                                src={getTrackArt(item)}
                                alt=""
                            />{:else}<Disc3 size={19} />{/if}
                    </button>
                    <button
                        class="collection-copy collection-open history-track-copy"
                        on:click={() => onPlay(item)}
                    >
                        <strong>{item.title || "Unknown track"}</strong>
                        <small
                            >{item.artist || "Unknown artist"} · {item.album ||
                                getTrackSource(item)}{item.playedAt
                                ? ` · ${new Date(item.playedAt).toLocaleDateString()}`
                                : ""}</small
                        >
                    </button>
                    <div class="collection-actions">
                        <button
                            class="row-action"
                            on:click={() => onQueue(item)}
                            aria-label={`Add ${item.title || "track"} to queue`}
                            title="Add to queue"><ListMusic size={15} /></button
                        >
                        <button
                            class="row-action"
                            class:active={isFavorite(item)}
                            on:click={() => onToggleFavorite(item)}
                            aria-label={isFavorite(item)
                                ? `Remove ${item.title || "track"} from liked songs`
                                : `Like ${item.title || "track"}`}
                            aria-pressed={isFavorite(item)}
                            title={isFavorite(item)
                                ? "Remove from liked songs"
                                : "Add to liked songs"}
                        >
                            <Heart
                                size={15}
                                fill={isFavorite(item)
                                    ? "currentColor"
                                    : "none"}
                            />
                        </button>
                    </div>
                </article>
            {/each}
        </div>
    {:else}
        <div class="imported-empty">
            <Clock3 size={20} />
            <p>No listening history yet.</p>
        </div>
    {/if}
</section>
