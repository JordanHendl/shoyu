use dashi::utils::*;
use dashi::*;

use crate::utils::Canvas;

pub struct GraphicsPipelineInfo {
    pub bg_layout: Handle<BindGroupLayout>,
    pub pipeline_layout: Handle<GraphicsPipelineLayout>,
    pub pipeline: Handle<GraphicsPipeline>,

    pub text_bg_layout: Handle<BindGroupLayout>,
    pub text_layout: Handle<GraphicsPipelineLayout>,
    pub text_pipeline: Handle<GraphicsPipeline>,
}

pub fn make_graphics_pipeline(ctx: &mut Context, canvas: &Canvas) -> GraphicsPipelineInfo {
    // Make the bind group layout. This describes the bindings into a shader.
    let bg_layout = ctx
        .make_bind_group_layout(&BindGroupLayoutInfo {
            shaders: &[
                ShaderInfo {
                    shader_type: ShaderType::Vertex,
                    variables: &[
                        BindGroupVariable {
                            var_type: BindGroupVariableType::DynamicUniform,
                            binding: 0,
                        },
                        BindGroupVariable {
                            var_type: BindGroupVariableType::DynamicUniform,
                            binding: 1,
                        },
                    ],
                },
                ShaderInfo {
                    shader_type: ShaderType::Fragment,
                    variables: &[BindGroupVariable {
                        var_type: BindGroupVariableType::SampledImage,
                        binding: 2,
                    }],
                },
            ],
        })
        .unwrap();

    // Make a pipeline layout. This describes a graphics pipeline's state.
    let pipeline_layout = ctx
        .make_graphics_pipeline_layout(&GraphicsPipelineLayoutInfo {
            vertex_info: VertexDescriptionInfo {
                entries: &[
                    VertexEntryInfo {
                        format: ShaderPrimitiveType::Vec2,
                        location: 0,
                        offset: 0,
                    },
                    VertexEntryInfo {
                        format: ShaderPrimitiveType::Vec2,
                        location: 1,
                        offset: 8,
                    },
                ],
                stride: 16,
                rate: VertexRate::Vertex,
            },
            bg_layout,
            shaders: &[
                PipelineShaderInfo {
                    stage: ShaderType::Vertex,
                    spirv: inline_spirv::inline_spirv!(
                        r#"
#version 450
layout(location = 0) in vec2 in_position;
layout(location = 1) in vec2 in_tex;
layout(location = 0) out vec2 frag_coords;

layout(binding = 0) uniform position_offset {
    mat4 transform;
};

layout(binding = 1) uniform camera_offset {
    vec2 camera;
};

void main() {
    vec4 position = transform * vec4(in_position, 0.0, 1.0);
    position -= vec4(camera.xy, 0.0, 0.0);
    gl_Position = position;
    frag_coords = in_tex;
}
"#,
                        vert
                    ),
                    specialization: &[],
                },
                PipelineShaderInfo {
                    stage: ShaderType::Fragment,
                    spirv: inline_spirv::inline_spirv!(
                        r#"
    #version 450 core
    layout(location = 0) in vec2 frag_coords;
    layout(location = 0) out vec4 out_color;
    layout(binding = 2) uniform sampler2D in_image;

    void main() { 
        out_color = texture(in_image, frag_coords); 
//        if(out_color.a < 0.9) 
//            discard;
    }
"#,
                        frag
                    ),
                    specialization: &[],
                },
            ],
            details: GraphicsPipelineDetails {
                topology: Topology::TriangleList,
                culling: CullMode::None,
                front_face: VertexOrdering::CounterClockwise,
                depth_test: false,
            },
        })
        .expect("Unable to create GFX Pipeline Layout!");

    // Make a graphics pipeline. This matches a pipeline layout to a render pass.
    let pipeline = ctx
        .make_graphics_pipeline(&dashi::GraphicsPipelineInfo {
            layout: pipeline_layout,
            render_pass: canvas.render_pass(),
        })
        .unwrap();

    ///////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////////

    // Make the bind group layout. This describes the bindings into a shader.
    let text_bg_layout = ctx
        .make_bind_group_layout(&BindGroupLayoutInfo {
            shaders: &[
                ShaderInfo {
                    shader_type: ShaderType::Vertex,
                    variables: &[],
                },
                ShaderInfo {
                    shader_type: ShaderType::Fragment,
                    variables: &[
                        BindGroupVariable {
                            var_type: BindGroupVariableType::DynamicUniform,
                            binding: 1,
                        },
                        BindGroupVariable {
                            var_type: BindGroupVariableType::SampledImage,
                            binding: 2,
                        },
                    ],
                },
            ],
        })
        .unwrap();

    // Make a pipeline layout. This describes a graphics pipeline's state.
    let text_layout = ctx
        .make_graphics_pipeline_layout(&GraphicsPipelineLayoutInfo {
            vertex_info: VertexDescriptionInfo {
                entries: &[
                    VertexEntryInfo {
                        format: ShaderPrimitiveType::Vec2,
                        location: 0,
                        offset: 0,
                    },
                    VertexEntryInfo {
                        format: ShaderPrimitiveType::Vec2,
                        location: 1,
                        offset: 8,
                    },
                ],
                stride: 16,
                rate: VertexRate::Vertex,
            },
            bg_layout: text_bg_layout,
            shaders: &[
                PipelineShaderInfo {
                    stage: ShaderType::Vertex,
                    spirv: inline_spirv::inline_spirv!(
                        r#"
#version 450
layout(location = 0) in vec2 in_position;
layout(location = 1) in vec2 in_tex;

layout(location = 0) out vec2 frag_coords;

void main() {
    gl_Position = vec4(in_position.xy, 0.0, 1.0);
    frag_coords = in_tex;
}
"#,
                        vert
                    ),
                    specialization: &[],
                },
                PipelineShaderInfo {
                    stage: ShaderType::Fragment,
                    spirv: inline_spirv::inline_spirv!(
                        r#"
    #version 450 core
    layout(location = 0) in vec2 frag_coords;
    layout(location = 0) out vec4 out_color;
    layout(binding = 2) uniform isampler2D in_image;
    layout(binding = 1) uniform camera_offset {
        vec4 color;
    };

    void main() { 
        out_color = texture(in_image, frag_coords); 
        if(out_color.r < 0.1)
            discard;
        else 
            out_color = vec4(color.xyz, 1.0);
        
    }
"#,
                        frag
                    ),
                    specialization: &[],
                },
            ],
            details: Default::default(),
        })
        .expect("Unable to create GFX Pipeline Layout!");

    // Make a graphics pipeline. This matches a pipeline layout to a render pass.
    let text_pipeline = ctx
        .make_graphics_pipeline(&dashi::GraphicsPipelineInfo {
            layout: text_layout,
            render_pass: canvas.render_pass(),
        })
        .unwrap();

    GraphicsPipelineInfo {
        bg_layout,
        pipeline_layout,
        pipeline,
        text_bg_layout,
        text_layout,
        text_pipeline,
    }
}
