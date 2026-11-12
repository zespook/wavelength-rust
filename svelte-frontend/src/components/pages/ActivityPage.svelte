<script>
    export let downloadLogs = [];
    export let downloadLogsError = "";
    export let downloadLogsLoading = false;
    export let notificationLogs = [];
    export let formatLogTime;

    let activeTab = "downloads";
</script>

<section class="download-logs-page" aria-labelledby="activity-heading">
    <header class="download-logs-heading">
        <h1 id="activity-heading">Activity</h1>
    </header>
    <div class="activity-tabs" role="tablist" aria-label="Activity type">
        <button
            type="button"
            role="tab"
            id="downloads-tab"
            aria-controls="downloads-panel"
            aria-selected={activeTab === "downloads"}
            class:active={activeTab === "downloads"}
            on:click={() => (activeTab = "downloads")}>Downloads</button
        >
        <button
            type="button"
            role="tab"
            id="notifications-tab"
            aria-controls="notifications-panel"
            aria-selected={activeTab === "notifications"}
            class:active={activeTab === "notifications"}
            on:click={() => (activeTab = "notifications")}>Notifications</button
        >
    </div>
    {#if activeTab === "downloads"}
        <div
            id="downloads-panel"
            class="activity-tab-panel"
            role="tabpanel"
            aria-labelledby="downloads-tab"
            tabindex="0"
        >
            {#if downloadLogsError}
                <p class="download-logs-error" role="alert">{downloadLogsError}</p>
            {:else if downloadLogs.length}
                <div
                    class="download-log-list"
                    role="table"
                    aria-label="Download activity"
                >
                    <div class="download-log-columns" aria-hidden="true">
                        <span>Time</span><span>Level</span><span>Source</span><span
                            >Event</span
                        ><span>Progress</span>
                    </div>
                    {#each downloadLogs as entry, index (`${entry.timestamp}-${entry.trackId || ""}-${index}`)}
                        <article
                            class:log-error={entry.level === "error"}
                            class:log-warning={entry.level === "warning"}
                            class="download-log-entry"
                        >
                            <time datetime={new Date(entry.timestamp).toISOString()}
                                >{formatLogTime(entry.timestamp)}</time
                            >
                            <span class="download-log-level">{entry.level}</span>
                            <span class="download-log-source">{entry.source || "—"}</span>
                            <div class="download-log-message">
                                <strong>{entry.message}</strong
                                >{#if entry.trackId}<small>{entry.trackId}</small>{/if}
                            </div>
                            {#if entry.progress != null}<progress
                                    max="100"
                                    value={entry.progress}
                                    aria-label={`Download progress ${entry.progress}%`}
                                ></progress>{/if}
                        </article>
                    {/each}
                </div>
            {:else}
                <p class="download-log-empty">
                    {downloadLogsLoading
                        ? "Loading activity…"
                        : "No download activity recorded."}
                </p>
            {/if}
        </div>
    {:else}
        <div
            id="notifications-panel"
            class="activity-tab-panel"
            role="tabpanel"
            aria-labelledby="notifications-tab"
            tabindex="0"
        >
            {#if notificationLogs.length}
                <div class="notification-log-list" role="list" aria-label="Popup notifications">
                    {#each notificationLogs as entry, index (`${entry.timestamp}-${index}`)}
                        <article class="notification-log-entry" role="listitem">
                            <time datetime={new Date(entry.timestamp).toISOString()}>{formatLogTime(entry.timestamp)}</time>
                            <p>{entry.message}</p>
                        </article>
                    {/each}
                </div>
            {:else}
                <p class="download-log-empty">No popup notifications recorded.</p>
            {/if}
        </div>
    {/if}
</section>
