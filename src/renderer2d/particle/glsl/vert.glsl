#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

layout(location = 0) in vec2 inPosition; // vertex position for a quad (-0.5, -0.5) to (0.5, 0.5)
layout(location = 0) out vec2 fragTexCoord; // Output texture coordinates for the fragment shader

#include <particle.glsl>

layout(binding = 0) readonly buffer particle_animation {
    ParticleAnimation animations[];
};

layout(binding = 1) buffer readonly particle_buffer {
    Particle particles[];
};

layout(binding = 2) uniform position_offset {
    mat4 transform;
};

layout(binding = 3) uniform camera_offset {
    vec2 camera;
    float delta_time;
};



void main() {
    // Retrieve the current particle instance
    Particle particle = particles[gl_InstanceIndex];

    // If the particle is not active, discard the vertex
    if (!particle.is_active) {
        gl_Position = vec4(0.0); // Degenerate vertex
        return;
    }

    // Apply particle transformation (position, rotation, scale)
    vec2 pos = inPosition * particle.size; // Scale the quad
    float cosRot = cos(particle.rotation);
    float sinRot = sin(particle.rotation);

    // Apply rotation matrix
    pos = vec2(
        pos.x * cosRot - pos.y * sinRot,
        pos.x * sinRot + pos.y * cosRot
    );
    
    // Translate to the particle's world position
    pos += particle.position;
    pos -= camera;
    
    
    // Convert to clip space
    gl_Position = transform * vec4(pos, 0.0, 1.0);
    
    
    // Calculate texture coordinates based on the particle's tex_coords
    // tex_coords.xy -> bottom-left corner of the texture in the atlas
    // tex_coords.zw -> top-right corner of the texture in the atlas
    vec2 texCoordMin = animations[particle.type].regions[particle.current_frame].region.xy; // Bottom-left UV
    vec2 texCoordMax = animations[particle.type].regions[particle.current_frame].region.zw; // Top-right UV

    // Interpolate between the bottom-left and top-right based on vertex position
    // Map inPosition (-0.5, -0.5) to (0.5, 0.5) to texCoordMin and texCoordMax
    fragTexCoord = mix(texCoordMin, texCoordMax, inPosition * 0.5 + 0.5);
}

