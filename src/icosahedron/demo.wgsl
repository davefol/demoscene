struct Vertex {
    @location(0) pos: vec3f,
    @location(1) normal: vec3f,
}

struct VsOut {
    @builtin(position) pos: vec4f,
    @location(0) normal: vec3f,
}

struct FsIn {
    @location(0) normal: vec3f,
}

struct FsOut {
    @location(0) color: vec4f,
}

@vertex
fn vs_main(
    v: Vertex
) -> VsOut {
    return VsOut(
        vec4f(v.pos.xyz, 1.0),
        v.normal
    );
}

@fragment
fn fs_main(f: FsIn) -> FsOut {
    return FsOut(
        vec4f(f.normal.xyz, 1.0)
    );
}