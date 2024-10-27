#ifndef PARTICLE_GLSL
#define PARTICLE_GLSL

const uint LINEAR = 0;
const uint GRAVITY = 0;

struct ParticleSpriteRegion {
  vec4 region;
};

const uint MAX_PARTICLE_ANIMATIONS = 128;
struct ParticleAnimation {
  float time_per_frame_ms;
  int animation_count;
  ParticleSpriteRegion regions[MAX_PARTICLE_ANIMATIONS];
};


struct Particle {
  vec2 position;
  vec2 size;
  vec2 velocity;
  int type;
  float rotation;
  uint current_frame;
  float max_lifetime;
  float curr_lifetime;
  uint behaviour;
  bool is_active;
  vec3 padding;
};

#endif
