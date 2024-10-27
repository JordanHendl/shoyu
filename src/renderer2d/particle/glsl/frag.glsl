
#version 450 core
layout(location = 0) in vec2 frag_coords;
layout(location = 0) out vec4 out_color;
layout(binding = 4) uniform sampler2D in_image;

void main() { 
    out_color = texture(in_image, frag_coords); 
      if(out_color.a < 0.9) 
          discard;
}
