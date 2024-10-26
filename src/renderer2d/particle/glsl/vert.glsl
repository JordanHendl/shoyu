#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

layout(location = 0) in vec2 inPosition; // vertex position for a quad (-0.5, -0.5) to (0.5, 0.5)

#include <particle.glsl>

layout(std430, binding = 0) buffer ParticleBuffer {
    Particle particles[];
};


layout(location = 0) out vec2 fragTexCoord; // Output texture coordinates for the fragment shader

layout(binding = 0) uniform position_offset {
    mat4 transform;
};

layout(binding = 1) uniform camera_offset {
    vec2 camera;
    float delta_time;
};


// Function to update particle based on behaviour
void updateParticle(inout Particle particle) {
    if (particle.behaviour == LINEAR) {
        particle.position += particle.initial_velocity * delta_time;
    } else if (particle.behaviour == GRAVITY) {
        particle.position += particle.initial_velocity * delta_time;
        particle.initial_velocity.y -= 9.81 * delta_time; // Apply gravity
    }

    particle.curr_lifetime += delta_time;
    if (particle.curr_lifetime > particle.max_lifetime) {
        particle.is_active = false; // Deactivate particle if its lifetime is over
    }
}

void main() {
    // Retrieve the current particle instance
    Particle particle = particles[gl_InstanceIndex];

    // If the particle is not active, discard the vertex
    if (!particle.is_active) {
        gl_Position = vec4(0.0); // Degenerate vertex
        return;
    }

    // Update particle state
    updateParticle(particle);

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
    vec2 texCoordMin = particle.tex_coords.xy; // Bottom-left UV
    vec2 texCoordMax = particle.tex_coords.zw; // Top-right UV

    // Interpolate between the bottom-left and top-right based on vertex position
    // Map inPosition (-0.5, -0.5) to (0.5, 0.5) to texCoordMin and texCoordMax
    fragTexCoord = mix(texCoordMin, texCoordMax, inPosition * 0.5 + 0.5);
}

