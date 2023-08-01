const organizer = import("./pkg/organizer");

organizer.then(code=>{
    code.texture_update_test();
}).catch(console.error);