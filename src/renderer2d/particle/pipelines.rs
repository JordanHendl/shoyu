use dashi::utils::*;
use dashi::*;

use crate::utils::Canvas;

pub struct ParticlePipelineInfo {
    pub bg_layout: Handle<BindGroupLayout>,
    pub pipeline_layout: Handle<GraphicsPipelineLayout>,
    pub pipeline: Handle<GraphicsPipeline>,

    pub compute_bg_layout: Handle<BindGroupLayout>,
    pub compute_layout: Handle<ComputePipelineLayout>,
    pub compute_pipeline: Handle<ComputePipeline>,
}

pub fn make_pipelines(ctx: &mut Context, canvas: &Canvas) -> ParticlePipelineInfo {
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
                    spirv: inline_spirv::include_spirv!("src/renderer2d/particle/glsl/vert.glsl", vert, I "src/renderer2d/particle/glsl/"),
                    specialization: &[],
                },
                PipelineShaderInfo {
                    stage: ShaderType::Fragment,
                    spirv: inline_spirv::include_spirv!("src/renderer2d/particle/glsl/frag.glsl", glsl, frag, I "src/renderer2d/particle/glsl/"),
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
    let compute_bg_layout = ctx
        .make_bind_group_layout(&BindGroupLayoutInfo {
            shaders: &[ShaderInfo {
                shader_type: ShaderType::Compute,
                variables: &[
                    BindGroupVariable {
                        var_type: BindGroupVariableType::Storage,
                        binding: 0,
                    },
                    BindGroupVariable {
                        var_type: BindGroupVariableType::DynamicUniform,
                        binding: 1,
                    },
                ],
            }],
        })
        .unwrap();

    // Make a pipeline layout. This describes a graphics pipeline's state.
    let compute_layout = ctx
        .make_compute_pipeline_layout(&ComputePipelineLayoutInfo {
            bg_layout: compute_bg_layout,
            shader: 
                &PipelineShaderInfo {
                    stage: ShaderType::Compute,
                    spirv: inline_spirv::include_spirv!("src/renderer2d/particle/glsl/compute.glsl", glsl, comp, I "src/renderer2d/particle/glsl/"),
                    specialization: &[],
                },
        })
        .expect("Unable to create Compute Pipeline Layout!");

    // Make a graphics pipeline. This matches a pipeline layout to a render pass.
    let compute_pipeline = ctx
        .make_compute_pipeline(&dashi::ComputePipelineInfo {
            layout: compute_layout,
        })
        .unwrap();

    ParticlePipelineInfo {
        bg_layout,
        pipeline_layout,
        pipeline,
        compute_bg_layout,
        compute_layout,
        compute_pipeline,
    }
}
