<script>
    import SelectMenu from "../../SelectMenu.svelte";

    export let appTheme = "dark";
    export let themeOptions = [];
    export let appAccent = "";
    export let onThemeChange;
    export let onOpenAccentPicker;
</script>

<section class="customization-page">
    <header class="customization-page-heading"><h1>Customization</h1></header>
    <div class="customization-panel">
        <div class="customization-control theme-setting-row">
            <div class="settings-row-copy"><strong>Appearance</strong></div>
            <SelectMenu
                value={appTheme}
                options={themeOptions}
                ariaLabel="Appearance"
                className="appearance-select"
                on:change={(event) => onThemeChange(event.detail)}
            />
        </div>
        <div class="customization-accent-control">
            <div class="settings-row-copy"><strong>Accent color</strong></div>
            <div class="accent-actions">
                <div class="accent-current">
                    <span
                        class="accent-current-swatch"
                        style={`background:${appAccent}`}
                    ></span><code>{appAccent.toUpperCase()}</code>
                </div>
                <button
                    class="outline-button compact-action"
                    on:click={onOpenAccentPicker}>Change</button
                >
            </div>
        </div>
    </div>
</section>
