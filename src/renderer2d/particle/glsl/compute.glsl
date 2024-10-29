#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32
#define BLOCK_SIZE_Y 1
#define BLOCK_SIZE_Z 1

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in;

#include <particle.glsl>

layout(binding = 0) buffer particle_animation {
    ParticleAnimation animations[];
};

layout(binding = 1) coherent buffer particle_buffer {
    Particle particles[];
};

layout(binding = 2) uniform position_offset {
    mat4 transform;
};

layout(binding = 3) uniform camera_offset {
    vec2 camera;
    float delta_time;
};

// Function to update particle based on behaviour
void update_particle(inout Particle particle) {
    if(!particle.is_active) return;

    if (particle.behaviour == LINEAR) {
    } else if (particle.behaviour == GRAVITY) {
        particle.velocity.y = -0.1; // Apply gravity
    }
    
    particle.position += particle.velocity * delta_time;
    particle.curr_lifetime = particle.curr_lifetime + (delta_time) * 1000.0;
    
  
    if(particle.curr_lifetime - particle.animation_timer > animations[particle.type].time_per_frame_ms) {
      particle.animation_timer = particle.curr_lifetime;
      particle.current_frame += 1;
    }

    if(particle.current_frame >= animations[particle.type].animation_count) {
      particle.current_frame = 0;
    }

    if (particle.curr_lifetime > particle.max_lifetime) {
        particle.animation_timer = 0.0;
        particle.is_active = false; // Deactivate particle if its lifetime is over
    }
}

void main() {
  Particle p = particles[gl_GlobalInvocationID.x];
  update_particle(p);
  particles[gl_GlobalInvocationID.x] = p;
}

