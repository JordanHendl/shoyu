#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

layout(location = 0) in vec2 inPosition; // vertex position for a quad (-0.5, -0.5) to (0.5, 0.5)
layout(location = 0) out vec2 fragTexCoord; // Output texture coordinates for the fragment shader

#include <particle.glsl>

layout(binding = 0) readonly buffer particle_animation {
    ParticleAnimation animations[];
};

layout(binding = 1) buffer particle_buffer {
    Particle particles[];
};

layout(binding = 2) uniform position_offset {
    mat4 transform;
};

layout(binding = 3) uniform camera_offset {
    vec2 camera;
    float delta_time;
};


mat4 build_transform(vec2 position, vec2 size, float rotation) {
    // Convert the rotation angle from degrees to radians
    float rad = radians(rotation);
    float cosAngle = cos(rad);
    float sinAngle = sin(rad);

    // Create the scaling matrix
    mat4 scale = mat4(
        size.x, 0.0,    0.0, 0.0,
        0.0,    size.y, 0.0, 0.0,
        0.0,    0.0,    1.0, 0.0,
        0.0,    0.0,    0.0, 1.0
    );

    // Create the rotation matrix around the Z-axis
    mat4 rotationZ = mat4(
        cosAngle, -sinAngle, 0.0, 0.0,
        sinAngle,  cosAngle, 0.0, 0.0,
        0.0,       0.0,      1.0, 0.0,
        0.0,       0.0,      0.0, 1.0
    );

    // Create the translation matrix
    mat4 translation = mat4(
        1.0, 0.0, 0.0, position.x,
        0.0, 1.0, 0.0, position.y,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    // Combine translation, rotation, and scale matrices
    return translation * rotationZ * scale;
}


void main() {
    // Retrieve the current particle instance
    Particle particle = particles[gl_InstanceIndex];

    // If the particle is not active, discard the vertex
    if (!particle.is_active) {
        gl_Position = vec4(0.0); // Degenerate vertex
        return;
    }
    
    mat4 t = build_transform(vec2(0.0, 0.0), particle.size, particle.rotation);

    gl_Position = t * vec4(inPosition, 0.0, 1.0);
    gl_Position.xy += particle.position;

    // Calculate texture coordinates based on the particle's tex_coords
    // tex_coords.xy -> bottom-left corner of the texture in the atlas
    // tex_coords.zw -> top-right corner of the texture in the atlas
    vec2 texCoordMin = animations[particle.type].regions[particle.current_frame].region.xy; // Bottom-left UV
    vec2 texCoordMax = animations[particle.type].regions[particle.current_frame].region.zw; // Top-right UV
    
    texCoordMax += texCoordMin;
    // Interpolate between the bottom-left and top-right based on vertex position
    // Map inPosition (-0.5, -0.5) to (0.5, 0.5) to texCoordMin and texCoordMax
    fragTexCoord = mix(texCoordMin, texCoordMax, inPosition * 0.5 + 0.5);
}

