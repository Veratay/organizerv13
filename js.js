export function log_json_string(x) {
    console.log(JSON.parse(x));
}

export function log_u8_as_f32_arr(x) {
    // Create a Float32Array with the same length as the encoded bytes
    const float32Array = new Float32Array(x.length / Float32Array.BYTES_PER_ELEMENT);

    // Iterate over the encoded bytes and populate the Float32Array
    for (let i = 0; i < float32Array.length; i++) {
        // Create a DataView to extract the bytes for each Float32 value
        const dataView = new DataView(x.buffer, i * Float32Array.BYTES_PER_ELEMENT, Float32Array.BYTES_PER_ELEMENT);
        
        // Get the Float32 value from the DataView
        const floatValue = dataView.getFloat32(0, true);
        
        // Store the Float32 value in the Float32Array
        float32Array[i] = floatValue;
    }
    console.log(float32Array);
    console.log(x);
}

export function window_resize_listener(f) {
    window.onresize = () => {
        f(window.innerWidth, window.innerHeight)
    }
}