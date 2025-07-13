struct Shape {
    w: u32,
    h: u32,
    r: u32,
}

fn unpackRGBA(p: u32) -> vec4<u32> {
    return vec4<u32>(
        (p >>  0) & 0xFFu,
        (p >>  8) & 0xFFu,
        (p >> 16) & 0xFFu,
        (p >> 24) & 0xFFu
    );
}

fn packRGBA(p: vec4<u32>) -> u32 {
    return (p.x & 0xFFu)
         | ((p.y & 0xFFu) <<  8)
         | ((p.z & 0xFFu) << 16)
         | ((p.w & 0xFFu) << 24);
}

@group(0) @binding(0) var<uniform> shape: Shape;
@group(0) @binding(1) var<storage, read> img_in: array<u32>;
@group(0) @binding(2) var<storage, read_write> img_out: array<u32>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3u) {
    if (id.x >= shape.w || id.y >= shape.h) {
        return;
    }

    let idx = id.y * shape.w + id.x;

    // average the values of the neighboring pixels within the radius
    let r = shape.r;
    var accum = vec4f(0.0, 0.0, 0.0, 0.0);
    var n = 0;
    for (var x = id.x - r; x <= id.x + r; x++) {
        for (var y = id.y - r; y <= id.y + r; y++) {
            if (x > 0 && x < shape.w && y > 0 && y < shape.h) {
                let sample_idx = y * shape.w + x;
                // let c = unpackRGBA(img_in[sample_idx]);
                // accum += vec4f(f32(c.r), f32(c.g), f32(c.b), f32(c.a));
                accum += unpack4x8unorm(img_in[sample_idx]);
                n += 1;
            }
        }
    }
    // let c_out = vec4<u32>(u32(accum.r / f32(n)), u32(accum.g / f32(n)), u32(accum.b / f32(n)), u32(accum.a / f32(n)));
    // img_out[idx] = packRGBA(c_out);
    img_out[idx] = pack4x8unorm(accum / f32(n));

}