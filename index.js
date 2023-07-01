const organizer = import("./pkg/organizer");

organizer.then(code=>{
    code.rust_hello_old();
}).catch(console.error);