import("./js.js")

const organizer = import("./pkg/organizer");

organizer.then(code=>{
    let f = code.lucas_game();
    function x() {
        requestAnimationFrame(x)
        f()
    }
    requestAnimationFrame(x)
}).catch(console.error);