
@group(0) @binding(0) var<uniform> transform: mat4x4<f32>;

struct Vertex {
    @location(0) pos: vec3f,
    @location(1) normal: vec3f,
}

struct VsOut {
    @builtin(position) pos: vec4f,
    @location(0) normal: vec3f
}

@vertex
fn vs_main(v: Vertex) -> VsOut {
    var pos = vec4f(v.pos.xyz, 1.0);
    var normal = vec4f(v.normal, 0.0) ;

    pos = transform * pos;
    normal = transform * normal;
    pos = vec4f(pos.xy, (pos.z + 1.0)/2.0, 1.0);
    
    return VsOut(
        pos,
        normal.xyz,
    );
}

struct FsOut {
    @location(0) color: vec4f,
}

@fragment
fn fs_main(f: VsOut) -> FsOut {
    let diffuse = dot(f.normal, vec3f(0.7, 0.7, 0.7));
    let ambient = 0.2;
    return FsOut (vec4f(diffuse + ambient, diffuse + ambient, diffuse + ambient, 1.0));
}