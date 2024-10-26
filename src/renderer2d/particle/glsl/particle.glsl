#ifndef PARTICLE_GLSL
#define PARTICLE_GLSL

const uint LINEAR = 0;
const uint GRAVITY = 0;

struct Particle {
  vec2 position;
  vec2 size;
  float rotation;
  vec2 initial_velocity;
  vec4 tex_coords;
  uint current_frame;
  float max_lifetime;
  float curr_lifetime;
  uint behaviour;
  bool is_active;
};

#endif
