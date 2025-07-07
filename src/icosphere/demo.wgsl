struct Time {
    time: f32,
}

@group(0) @binding(0) var<uniform> time: Time;

struct Vertex {
    @location(0) pos: vec3f,
    @location(1) normal: vec3f,
}

struct VsOut {
    @builtin(position) pos: vec4f,
    @location(0) normal: vec3f,
}


@vertex
fn vs_main(
    v: Vertex
) -> VsOut {
    let c = cos(time.time * 0.5);
    let s = sin(time.time * 0.5);

    let pos = vec3f(
        c * v.pos.x - s * v.pos.z,
        v.pos.y,
        s * v.pos.x + c * v.pos.z,
    );

    let normal = vec3f(
        c * v.normal.x - s * v.normal.z,
        v.normal.y,
        s * v.normal.x + c * v.normal.z,
    );

    return VsOut(
        vec4f(pos.xy, (pos.z + 1.0)/2.0, 1.0),
        normal
    );

    // return VsOut(
    //     vec4f(v.pos.xyz, 1.0),
    //     v.normal
    // );
}

struct FsOut {
    @location(0) color: vec4f,
}

@fragment
fn fs_main(f: VsOut) -> FsOut {
    return FsOut (vec4f(1.0, 1.0, 1.0, 1.0));
}