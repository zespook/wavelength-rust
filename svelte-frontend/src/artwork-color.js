export function averageArtworkColor(source, fallbackColor = "#101311") {
    if (!source) return Promise.resolve(fallbackColor);

    return new Promise((resolve) => {
        const image = new Image();
        image.crossOrigin = "anonymous";
        image.onload = () => {
            const canvas = document.createElement("canvas");
            canvas.width = 32;
            canvas.height = 32;
            const context = canvas.getContext("2d", { willReadFrequently: true });
            if (!context) {
                resolve(fallbackColor);
                return;
            }

            try {
                context.imageSmoothingQuality = "high";
                context.drawImage(image, 0, 0, canvas.width, canvas.height);
                const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data;
                let red = 0;
                let green = 0;
                let blue = 0;
                let alphaTotal = 0;

                for (let pixelIndex = 0; pixelIndex < pixels.length; pixelIndex += 4) {
                    const alpha = pixels[pixelIndex + 3] / 255;
                    red += pixels[pixelIndex] * alpha;
                    green += pixels[pixelIndex + 1] * alpha;
                    blue += pixels[pixelIndex + 2] * alpha;
                    alphaTotal += alpha;
                }

                resolve(alphaTotal > 0
                    ? `rgb(${Math.round(red / alphaTotal)} ${Math.round(green / alphaTotal)} ${Math.round(blue / alphaTotal)})`
                    : fallbackColor);
            } catch {
                resolve(fallbackColor);
            }
        };
        image.onerror = () => resolve(fallbackColor);
        image.src = source;
    });
}