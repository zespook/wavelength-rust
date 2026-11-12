const MARQUEE_SPEED_MULTIPLIER = 0.85;

export function marquee(node, automatic = false) {
    node.dataset.marquee = 'true';
    let animation;
    let originalStyle;
    let mirror;
    let resizeObserver;
    let mutationObserver;
    let animationFrame;
    const start = () => {
        const content = automatic ? node.firstElementChild : node;
        if (!content) return;
        if (automatic) {
            if (animation || window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
            const fullWidth = content.scrollWidth;
            const distance = fullWidth - node.clientWidth;
            if (distance < 4) {
                mirror?.remove();
                mirror = null;
                return;
            }
            const gap = 24;
            if (!mirror) {
                mirror = content.cloneNode(true);
                mirror.setAttribute('aria-hidden', 'true');
                mirror.setAttribute('data-marquee-copy', 'true');
                node.appendChild(mirror);
            } else if (mirror.textContent !== content.textContent) {
                mirror.textContent = content.textContent;
            }
            originalStyle = {
                display: node.style.display,
                width: node.style.width,
                maxWidth: node.style.maxWidth,
                flex: node.style.flex,
                gap: node.style.gap,
                overflow: node.style.overflow,
                whiteSpace: node.style.whiteSpace,
                textOverflow: node.style.textOverflow,
            };
            node.style.display = 'flex';
            node.style.width = `${fullWidth * 2 + gap}px`;
            node.style.maxWidth = 'none';
            node.style.flex = '0 0 auto';
            node.style.gap = `${gap}px`;
            node.style.overflow = 'visible';
            node.style.whiteSpace = 'nowrap';
            node.style.textOverflow = 'clip';
            animation = node.animate(
                [{ transform: 'translateX(0)' }, { transform: `translateX(-${fullWidth + gap}px)` }],
                {
                    duration: Math.max(5000, (fullWidth + gap) * 28) / MARQUEE_SPEED_MULTIPLIER,
                    iterations: Infinity,
                    easing: 'linear',
                },
            );
            return;
        }
        const fullWidth = node.scrollWidth;
        const distance = fullWidth - node.clientWidth;
        if (animation || distance < 4 || window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
        originalStyle = {
            width: node.style.width,
            maxWidth: node.style.maxWidth,
            flex: node.style.flex,
            overflow: node.style.overflow,
            textOverflow: node.style.textOverflow,
        };
        node.style.width = `${fullWidth}px`;
        node.style.maxWidth = 'none';
        node.style.flex = '0 0 auto';
        node.style.overflow = 'visible';
        node.style.textOverflow = 'clip';
        animation = node.animate([{ transform: 'translateX(0)' }, { transform: `translateX(-${distance}px)` }], {
            duration: Math.max(2400, distance * 28) / MARQUEE_SPEED_MULTIPLIER,
            direction: 'alternate',
            iterations: Infinity,
            easing: 'linear',
        });
    };
    const stop = () => {
        animation?.cancel();
        animation = null;
        if (!originalStyle) return;
        Object.assign(node.style, originalStyle);
        originalStyle = null;
    };
    const restartAutomatic = () => {
        stop();
        cancelAnimationFrame(animationFrame);
        animationFrame = requestAnimationFrame(start);
    };
    if (automatic) {
        resizeObserver = new ResizeObserver(restartAutomatic);
        resizeObserver.observe(node.parentElement || node);
        mutationObserver = new MutationObserver((records) => {
            const contentChanged = records.some((record) => {
                if (record.target === node) {
                    return [...record.addedNodes, ...record.removedNodes].some(
                        (changedNode) => changedNode !== mirror,
                    );
                }
                return !mirror?.contains(record.target);
            });
            if (contentChanged) restartAutomatic();
        });
        mutationObserver.observe(node, { childList: true, characterData: true, subtree: true });
        animationFrame = requestAnimationFrame(start);
    } else {
        node.addEventListener('mouseenter', start);
        node.addEventListener('mouseleave', stop);
        node.addEventListener('focus', start);
        node.addEventListener('blur', stop);
    }
    return {
        destroy() {
            stop();
            cancelAnimationFrame(animationFrame);
            resizeObserver?.disconnect();
            mutationObserver?.disconnect();
            mirror?.remove();
            if (!automatic) {
                node.removeEventListener('mouseenter', start);
                node.removeEventListener('mouseleave', stop);
                node.removeEventListener('focus', start);
                node.removeEventListener('blur', stop);
            }
        },
    };
}