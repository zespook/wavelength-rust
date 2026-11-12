<script>
    import { onMount } from "svelte";
    import { X } from "lucide-svelte";
    import { averageArtworkColor } from "../artwork-color.js";

    export let playing = null;
    export let mobileViewport = false;
    export let lyrics = "";
    export let syncedLyricLines = [];
    export let activeLyricIndex = -1;
    export let lyricsList = null;
    export let getTrackArt;
    export let onClose;

    let artworkAverageColor = "#101311";
    let artworkRequest = 0;
    let requestedArtwork = "";
    let lyricsPanel;
    let desktopPanelLeft = "0px";
    let desktopPanelWidth = "100%";

    $: artworkUrl = playing && getTrackArt ? getTrackArt(playing) : "";
    $: if (artworkUrl !== requestedArtwork) {
        requestedArtwork = artworkUrl;
        updateArtworkAverage(artworkUrl);
    }

    async function updateArtworkAverage(source) {
        const request = ++artworkRequest;
        const fallbackColor = playing?.color || "#101311";
        artworkAverageColor = fallbackColor;
        const color = await averageArtworkColor(source, fallbackColor);
        if (request === artworkRequest) artworkAverageColor = color;
    }

    onMount(() => {
        const content = lyricsPanel?.closest(".workspace-cluster > .content");
        const contentInner = lyricsPanel?.parentElement;
        if (!content || !contentInner) return;

        const updateDesktopBounds = () => {
            const contentBounds = content.getBoundingClientRect();
            const innerBounds = contentInner.getBoundingClientRect();
            const appZoom = Number.parseFloat(
                getComputedStyle(document.getElementById("app")).zoom,
            ) || 1;
            const rails = [...content.querySelectorAll(".right-rail, .details-rail")]
                .filter((rail) => {
                    const bounds = rail.getBoundingClientRect();
                    return getComputedStyle(rail).display !== "none" && bounds.width > 0;
                });
            const rightEdge = rails.length
                ? Math.min(...rails.map((rail) => rail.getBoundingClientRect().left))
                : contentBounds.right;
            desktopPanelLeft = `${(contentBounds.left - innerBounds.left) / appZoom}px`;
            desktopPanelWidth = `${(rightEdge - contentBounds.left) / appZoom}px`;
        };

        const observer = new ResizeObserver(updateDesktopBounds);
        observer.observe(content);
        observer.observe(contentInner);
        content.querySelectorAll(".right-rail, .details-rail").forEach((rail) =>
            observer.observe(rail),
        );
        window.addEventListener("resize", updateDesktopBounds);
        updateDesktopBounds();

        return () => {
            observer.disconnect();
            window.removeEventListener("resize", updateDesktopBounds);
        };
    });
</script>

<section
    class:desktop-lyrics={!mobileViewport}
    class="now-playing-extra-panel lyrics-panel lyrics-fullscreen"
    aria-label="Fullscreen lyrics"
    bind:this={lyricsPanel}
    style={`--lyrics-art-average:${artworkAverageColor};--lyrics-panel-left:${desktopPanelLeft};--lyrics-panel-width:${desktopPanelWidth};background-color:color-mix(in srgb,var(--lyrics-art-average) var(--lyrics-art-alpha),transparent)`}
>
    {#if mobileViewport}
        <header class="lyrics-heading">
            <button
                class="lyrics-close"
                on:click={onClose}
                aria-label="Close lyrics"
                title="Close lyrics"><X size={20} /></button
            >
        </header>
    {/if}
    <div
        class="lyrics-lines"
        bind:this={lyricsList}
        aria-live="off"
        aria-label="Lyrics"
    >
        {#if syncedLyricLines.length}
            {#each syncedLyricLines as line, index}<p
                    class:passed={activeLyricIndex >= 0 &&
                        index < activeLyricIndex}
                    class:active={index === activeLyricIndex}
                    class:upcoming={line.time !== null &&
                        (activeLyricIndex < 0 || index > activeLyricIndex)}
                    class:untimed={line.time === null}
                >
                    {line.text}
                </p>{/each}
        {:else}
            <p>{lyrics || "Loading lyrics…"}</p>
        {/if}
    </div>
</section>
