<script context="module">
    let selectMenuCounter = 0;
</script>

<script>
    import { createEventDispatcher, tick } from "svelte";
    import { Check, ChevronDown } from "lucide-svelte";

    export let value = "";
    export let options = [];
    export let ariaLabel = "Select option";
    export let className = "";
    export let disabled = false;

    const dispatch = createEventDispatcher();
    const menuId = `wavelength-select-menu-${++selectMenuCounter}`;
    let root;
    let trigger;
    let isOpen = false;

    $: selectedOption = options.find((option) => option.value === value) ||
        options[0] || { label: "", value: "" };

    function closeOnOutsidePointer(event) {
        if (!root?.contains(event.target)) isOpen = false;
    }

    async function focusSelectedOption() {
        await tick();
        const optionButtons = [
            ...(root?.querySelectorAll('[role="option"]') || []),
        ];
        const selectedIndex = optionButtons.findIndex(
            (optionButton) => optionButton.dataset.value === String(value),
        );
        optionButtons[selectedIndex >= 0 ? selectedIndex : 0]?.focus();
    }

    function openMenu() {
        if (disabled) return;
        isOpen = true;
        focusSelectedOption();
    }

    function toggleMenu() {
        if (isOpen) isOpen = false;
        else openMenu();
    }

    function handleTriggerKeydown(event) {
        if (disabled || !["ArrowDown", "ArrowUp"].includes(event.key)) return;
        event.preventDefault();
        openMenu();
    }

    function handleListKeydown(event) {
        const optionButtons = [
            ...(root?.querySelectorAll('[role="option"]') || []),
        ];
        const activeIndex = optionButtons.indexOf(document.activeElement);
        let nextIndex = activeIndex;

        if (event.key === "Escape") {
            event.preventDefault();
            isOpen = false;
            trigger?.focus();
            return;
        }
        if (event.key === "ArrowDown")
            nextIndex = Math.min(optionButtons.length - 1, activeIndex + 1);
        else if (event.key === "ArrowUp")
            nextIndex = Math.max(0, activeIndex - 1);
        else if (event.key === "Home") nextIndex = 0;
        else if (event.key === "End") nextIndex = optionButtons.length - 1;
        else return;

        event.preventDefault();
        optionButtons[nextIndex]?.focus();
    }

    function chooseOption(option) {
        value = option.value;
        isOpen = false;
        dispatch("change", value);
        trigger?.focus();
    }
</script>

<svelte:window on:pointerdown={closeOnOutsidePointer} />

<div
    bind:this={root}
    class={`select-menu ${className}`}
    class:open={isOpen}
    class:disabled
>
    <button
        bind:this={trigger}
        type="button"
        class="select-menu-trigger"
        role="combobox"
        aria-label={ariaLabel}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        aria-controls={menuId}
        {disabled}
        on:click={toggleMenu}
        on:keydown={handleTriggerKeydown}
    >
        <span class="select-menu-value">{selectedOption.label}</span>
        <ChevronDown size={15} aria-hidden="true" />
    </button>
    {#if isOpen}
        <div
            id={menuId}
            class="select-menu-options"
            role="listbox"
            aria-label={ariaLabel}
            tabindex="-1"
            on:keydown={handleListKeydown}
        >
            {#each options as option (option.value)}
                <button
                    type="button"
                    class="select-menu-option"
                    class:selected={option.value === value}
                    role="option"
                    aria-selected={option.value === value}
                    data-value={option.value}
                    tabindex={option.value === value ? 0 : -1}
                    on:click={() => chooseOption(option)}
                >
                    <span>{option.label}</span>
                    {#if option.value === value}<Check
                            size={14}
                            aria-hidden="true"
                        />{/if}
                </button>
            {/each}
        </div>
    {/if}
</div>
