const organizer = import("./pkg/organizer");

export function loadImageAndAwaitLoad(url, callback) {
    let img = new Image();
    img.onload = () => {
        callback(img)
    }
    img.src = url;
}

organizer.then(code=>{
    code.texture_test();
}).catch(console.error);