#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32
#define BLOCK_SIZE_Y 1
#define BLOCK_SIZE_Z 1

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in;

#include <particle.glsl>

layout(binding = 0) buffer ParticleSystem {
  Particle particles[];
};

layout(binding = 1) uniform ParticleSystemConfig {
  vec2 camera;
  float dt;
};

void main() {

}

