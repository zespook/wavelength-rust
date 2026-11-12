<script>
    import { Disc3, ListMusic, Play, Trash2, X } from "lucide-svelte";

    export let queue = [];
    export let queueDownloadStatuses = {};
    export let queueDownloadCompletionMarks = {};
    export let draggedQueueIndex = null;
    export let dragOverQueueIndex = null;
    export let getQueueAudioId;
    export let getQueueRowState;
    export let getTrackArt;
    export let getTrackSource;
    export let isQueueDownloadable;
    export let canManageQueue = true;
    export let canControlPlayback = true;
    export let onClearQueue;
    export let onPlay;
    export let onRemove;
    export let onDragStart;
    export let onDragOver;
    export let onDrop;
    export let onDragEnd;
    export let onPointerDown;
    export let onReorderKeydown;
    export let onContextMenu;
</script>

<section class="mobile-queue-page">
    <section class="queue-list" aria-label="Queue">
        {#if queue.length}<div class="mobile-queue-actions">
                <button
                    class="icon-button danger"
                    aria-label="Clear queue"
                    title="Clear queue"
                    disabled={!canManageQueue}
                        on:click={onClearQueue}><Trash2 size={15} /></button>
                    </div>
                {/if}
        {#if queue.length}
            {#each queue as item, index (item.id)}
                {@const download =
                    queueDownloadStatuses[getQueueAudioId(item)] ||
                    (getTrackSource(item) === "Local"
                        ? { status: "ready", progress: 100 }
                        : { status: "pending", progress: null })}
                {@const rowState = getQueueRowState(item)}
                <div
                    class="mobile-queue-row queue-draggable"
                    on:contextmenu={(event) => onContextMenu(event, item)}
                    data-queue-index={index}
                    class:now-playing={rowState.nowPlaying}
                    class:up-next={index === 0 && !rowState.nowPlaying}
                    class:dragging={draggedQueueIndex === index}
                    class:drop-target={dragOverQueueIndex === index}
                    role="listitem"
                    aria-roledescription="reorderable queue item"
                    draggable={canManageQueue}
                    on:dragstart={(event) => canManageQueue && onDragStart(event, index)}
                    on:dragover={(event) => canManageQueue && onDragOver(event, index)}
                    on:drop={(event) => canManageQueue && onDrop(event, index)}
                    on:dragend={onDragEnd}
                >
                    <span
                        class="queue-index queue-drag-handle"
                        role="button"
                        tabindex={canManageQueue ? 0 : -1}
                        aria-disabled={!canManageQueue}
                        title="Long-press and drag to reorder"
                        aria-label={`Reorder ${item.title}; use arrow keys to move`}
                        on:pointerdown={(event) => canManageQueue && onPointerDown(event, index)}
                        on:keydown={(event) => canManageQueue && onReorderKeydown(event, index)}
                        ><span
                            class="queue-source-dot"
                            class:spotify={getTrackSource(item) === "Spotify"}
                            class:youtube={getTrackSource(item) === "YouTube"}
                            aria-hidden="true"
                        ></span></span
                    >
                    <span class="queue-art mini-art"
                        >{#if getTrackArt(item)}<img
                                src={getTrackArt(item)}
                                alt=""
                            />{:else}<Disc3
                                size={16}
                            />{/if}{#if isQueueDownloadable(item) && (download.status !== "ready" || queueDownloadCompletionMarks[getQueueAudioId(item)])}<span
                                class="queue-download-indicator"
                                class:ready={download.status === "ready"}
                                class:completion={queueDownloadCompletionMarks[
                                    getQueueAudioId(item)
                                ]}
                                class:error={download.status === "error"}
                                class:indeterminate={download.progress == null}
                                style={`--download-progress: ${Math.min(100, Math.max(0, Number(download.progress) || 0))}%`}
                                role="img"
                                aria-label={download.error ||
                                    (download.progress == null
                                        ? download.status
                                        : `${download.status} ${download.progress}%`)}
                                title={download.error ||
                                    (download.progress == null
                                        ? download.status
                                        : `${download.status} ${download.progress}%`)}
                                ><span class="queue-download-ring"
                                ></span>{#if download.status === "ready"}<svg
                                        class="queue-download-mark"
                                        viewBox="0 0 16 16"
                                        aria-hidden="true"
                                        ><path d="m3.5 8.5 3 3 6-7" /></svg
                                    >{:else if download.status === "error"}<span
                                        class="queue-download-mark">!</span
                                    >{:else if download.progress == null}<span
                                        class="queue-download-spinner"
                                    ></span>{:else}<span
                                        class="queue-download-percent"
                                        >{Math.round(download.progress)}%</span
                                    >{/if}</span
                            >{/if}</span
                    >
                    <span class="mobile-queue-track-copy"
                        ><strong>{item.title}</strong><small
                            >{item.artist} · {getTrackSource(item)}</small
                        ></span
                    >
                    <button
                        class="row-action play-row-action"
                        disabled={!canControlPlayback}
                        on:click={() => onPlay(item)}
                        aria-label={`Play ${item.title}`}
                        ><Play size={15} /></button
                    >
                    <button
                        class="row-action"
                        disabled={!canManageQueue}
                        on:click={() => onRemove(item.id)}
                        aria-label={`Remove ${item.title}`}
                        ><X size={15} /></button
                    >
                </div>
            {/each}
        {:else}
            <div class="queue-empty-state">
                <span class="queue-empty-mark"><ListMusic size={20} /></span>
                <div>
                    <h3>Nothing queued</h3>
                    <p>Add a song from your library and it will appear here.</p>
                </div>
            </div>
        {/if}
    </section>
</section>
