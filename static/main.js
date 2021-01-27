import discriminord_init, * as discriminord from './discriminord.js'

const darkBackgroundInput = document.getElementById('dark-background');
const lightBackgroundInput = document.getElementById('light-background');
const darkImageInput = document.getElementById('dark-image');
const lightImageInput = document.getElementById('light-image');

const darkPreviewImage = document.getElementById('dark-preview');
const lightPreviewImage = document.getElementById('light-preview');

let imageBuffer = null;
let imageBlob = null;
let imageURL = null;

function setPreview(element, background) {
    element.style.backgroundColor = background;
    element.src = imageURL;
}

async function update() {
    if (darkImageInput.files.length > 0 && lightImageInput.files.length > 0) {
        const [darkImageBuffer, lightImageBuffer] = await Promise.all([
            darkImageInput.files[0].arrayBuffer(),
            lightImageInput.files[0].arrayBuffer(),
        ]);

        imageBuffer = discriminord.convert(
            new Uint8Array(darkImageBuffer),
            new Uint8Array(lightImageBuffer),
            darkBackgroundInput.value,
            lightBackgroundInput.value,
        );
        imageBlob = new Blob([imageBuffer], {type: "image/png"});

        if (imageURL !== null) {
            URL.revokeObjectURL(imageURL);
        }
        imageURL = URL.createObjectURL(imageBlob);
    }
    
    if (imageURL !== null) {
        setPreview(darkPreviewImage, darkBackgroundInput.value);
        setPreview(lightPreviewImage, lightBackgroundInput.value);
    }
}

discriminord_init().then(() => {
    update();
});

darkBackgroundInput.addEventListener('change', update);
lightBackgroundInput.addEventListener('change', update);
darkImageInput.addEventListener('change', update);
lightImageInput.addEventListener('change', update);
