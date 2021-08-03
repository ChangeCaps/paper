struct VertexInput {
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] color: vec4<f32>;
};

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] color: vec4<f32>;
};

[[block]]
struct Uniforms {
	transform: mat4x4<f32>;	
	view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;

	out.position = uniforms.view_proj * uniforms.transform * vec4<f32>(in.position, 1.0);
	out.color = in.color;

	return out;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	return in.color;
}
