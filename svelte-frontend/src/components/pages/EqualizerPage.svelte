<script>
    import { Plus, RotateCcw } from "lucide-svelte";
    import SelectMenu from "../../SelectMenu.svelte";

    export let equalizerSpectrumCanvas = null;
    export let equalizerPresetOptions = [];
    export let activeEqualizerPresetId = "flat";
    export let equalizerPresetName = "";
    export let equalizerBands = [];
    export let equalizerGains = [];
    export let equalizerFrequencies = [];
    export let equalizerQs = [];
    export let equalizerTypeOptions = [];
    export let equalizerTypes = [];
    export let equalizerNodeColors = [];
    export let selectedEqualizerBand = 0;
    export let equalizerUnavailable = false;
    export let onSelectPreset;
    export let onSavePreset;
    export let onResetEqualizer;
    export let onResetBand;
    export let onSetGain;
    export let onSetFrequency;
    export let onSetQ;
    export let onSetType;
    export let onGraphPointerDown;
    export let onGraphPointerMove;
    export let onGraphPointerUp;
    export let onGraphPointerCancel;

    $: selectedFrequency = equalizerFrequencies[selectedEqualizerBand] || equalizerBands[selectedEqualizerBand] || 60;
    $: selectedGain = equalizerGains[selectedEqualizerBand] || 0;
    $: selectedQ = equalizerQs[selectedEqualizerBand] || 1.1;
    $: selectedType = equalizerTypes[selectedEqualizerBand] || "peaking";
    $: selectedTypeLabel = equalizerTypeOptions.find((option) => option.value === selectedType)?.label || "Bell";
    $: qIsAvailable = selectedType === "peaking";

    function formatFrequency(value) {
        return value >= 1000
            ? `${(value / 1000).toFixed(value >= 10000 ? 0 : 1).replace(/\.0$/, "")} kHz`
            : `${Math.round(value)} Hz`;
    }

    function shortFrequency(value) {
        return value >= 1000
            ? `${(value / 1000).toFixed(value >= 10000 ? 0 : 1).replace(/\.0$/, "")}k`
            : `${Math.round(value)}`;
    }

    function sliderFrequency(value) {
        return Math.round((Math.log(Math.max(20, value) / 20) / Math.log(1000)) * 1000);
    }

    function gainLabel(value) {
        const rounded = Math.round(value * 10) / 10;
        return `${rounded > 0 ? "+" : ""}${Number.isInteger(rounded) ? rounded : rounded.toFixed(1)} dB`;
    }
</script>

<section class="equalizer-page">
    <header class="equalizer-page-heading">
        <div>
            <h1>Equalizer</h1>
            <p>10-band parametric EQ</p>
        </div>
    </header>
    <div class="equalizer-workbench-scroll">
        <div class="equalizer-workbench">
            <div class="equalizer-toolbar">
                <label class="equalizer-preset-control">
                    <span>Preset</span>
                    <SelectMenu
                        bind:value={activeEqualizerPresetId}
                        options={equalizerPresetOptions}
                        ariaLabel="Equalizer preset"
                        on:change={onSelectPreset}
                    />
                </label>
                <div class="equalizer-toolbar-actions">
                    <button
                        class="icon-button equalizer-reset"
                        type="button"
                        aria-label="Reset equalizer"
                        title="Reset equalizer"
                        on:click={onResetEqualizer}><RotateCcw size={16} /></button
                    >
                    <form
                        class="equalizer-save-form"
                        on:submit|preventDefault={onSavePreset}
                    >
                        <input
                            bind:value={equalizerPresetName}
                            maxlength="40"
                            placeholder="New preset name"
                            aria-label="New equalizer preset name"
                        /><button
                            class="outline-button compact-action"
                            type="submit"><Plus size={14} /> Save</button
                        >
                    </form>
                </div>
            </div>
            <div class="equalizer-graph-panel">
                <canvas
                    bind:this={equalizerSpectrumCanvas}
                    class="equalizer-spectrum-canvas"
                    aria-label="Equalizer response curve and live audio spectrum. Drag a node to adjust its frequency and gain."
                    on:pointerdown={onGraphPointerDown}
                    on:pointermove={onGraphPointerMove}
                    on:pointerup={onGraphPointerUp}
                    on:pointercancel={onGraphPointerCancel}
                ></canvas>
            </div>
            <div class="equalizer-band-strip" role="group" aria-label="Equalizer bands">
                {#each equalizerBands as defaultFrequency, index (index)}
                    <button
                        type="button"
                        class:selected={selectedEqualizerBand === index}
                        class="equalizer-band-tab"
                        style={`--band-color:${equalizerNodeColors[index] || "var(--lime)"}`}
                        aria-pressed={selectedEqualizerBand === index}
                        aria-label={`Select band ${index + 1}, ${formatFrequency(equalizerFrequencies[index] || defaultFrequency)}, ${gainLabel(equalizerGains[index] || 0)}`}
                        on:click={() => (selectedEqualizerBand = index)}
                    >
                        <span class="equalizer-band-dot"></span>
                        <span class="equalizer-band-number">{String(index + 1).padStart(2, "0")}</span>
                        <strong>{shortFrequency(equalizerFrequencies[index] || defaultFrequency)}</strong>
                        <small>{gainLabel(equalizerGains[index] || 0)}</small>
                    </button>
                {/each}
            </div>
            <section class="equalizer-band-editor" aria-label={`Band ${selectedEqualizerBand + 1} controls`}>
                <header class="equalizer-band-editor-heading">
                    <div class="equalizer-band-title">
                        <span
                            class="equalizer-band-dot"
                            style={`--band-color:${equalizerNodeColors[selectedEqualizerBand] || "var(--lime)"}`}
                        ></span>
                        <strong>Band {String(selectedEqualizerBand + 1).padStart(2, "0")}</strong>
                        <span class="equalizer-filter-type">{selectedTypeLabel}</span>
                    </div>
                    <button
                        class="text-button equalizer-band-reset"
                        type="button"
                        on:click={() => onResetBand(selectedEqualizerBand)}
                    >Reset band</button>
                </header>
                <label class="equalizer-type-control">
                    <span>Shape</span>
                    <SelectMenu
                        value={selectedType}
                        options={equalizerTypeOptions}
                        ariaLabel={`Band ${selectedEqualizerBand + 1} shape`}
                        on:change={(event) => onSetType(selectedEqualizerBand, event)}
                    />
                </label>
                <label class="equalizer-parameter">
                    <span>Frequency</span>
                    <output>{formatFrequency(selectedFrequency)}</output>
                    <input
                        type="range"
                        min="0"
                        max="1000"
                        step="1"
                        value={sliderFrequency(selectedFrequency)}
                        aria-label={`Band ${selectedEqualizerBand + 1} frequency`}
                        disabled={equalizerUnavailable}
                        on:input={(event) => onSetFrequency(selectedEqualizerBand, event)}
                    />
                </label>
                <label class="equalizer-parameter">
                    <span>Gain</span>
                    <output>{gainLabel(selectedGain)}</output>
                    <input
                        type="range"
                        min="-18"
                        max="18"
                        step="0.1"
                        value={selectedGain}
                        aria-label={`Band ${selectedEqualizerBand + 1} gain`}
                        disabled={equalizerUnavailable}
                        on:input={(event) => onSetGain(selectedEqualizerBand, event)}
                    />
                </label>
                <label class="equalizer-parameter">
                    <span>Q</span>
                    <output>{qIsAvailable ? selectedQ.toFixed(1) : "Not used"}</output>
                    <input
                        type="range"
                        min="0.3"
                        max="10"
                        step="0.1"
                        value={selectedQ}
                        aria-label={`Band ${selectedEqualizerBand + 1} Q`}
                        disabled={equalizerUnavailable || !qIsAvailable}
                        on:input={(event) => onSetQ(selectedEqualizerBand, event)}
                    />
                </label>
            </section>
        </div>
    </div>
    {#if equalizerUnavailable}<p class="equalizer-unavailable" role="status">
            Audio equalization is unavailable in this browser.
        </p>{/if}
</section>
