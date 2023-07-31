const organizer = import("./pkg/organizer");

organizer.then(code=>{
    code.line_test();
}).catch(console.error);