pub const VSHADER_FLAT: &str = 
"#version 300 es
layout(location = 0) in vec3 aPosition;
layout(location = 1) in vec3 aNormal;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;
uniform mat3 normalMatrix;

uniform float animTime;

flat out vec3 Normal;
out vec3 FragPos;

void main() {
    FragPos = vec3(model * vec4(aPosition, 1.0));
    Normal = normalMatrix * aNormal;
    gl_Position = projection * view * vec4(FragPos, 1.0);
}";

pub const FSHADER_FLAT: &str = 
"#version 300 es
precision mediump float;

flat in vec3 Normal;
in vec3 FragPos;
out vec4 outColor;

uniform vec3 lightPos;
uniform vec3 lightColor;
uniform vec3 objectColor;

uniform float animTime;

void main() {
    float ambientStrength = 0.1;
    vec3 ambient = objectColor * ambientStrength;

    vec3 lightDir = normalize(lightPos - FragPos);
    float diff = max(dot(normalize(Normal), lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    outColor = vec4((ambient + diffuse) * objectColor, 1.0);
}";

pub const VSHADER_SMOOTH: &str = 
"#version 300 es
layout(location = 0) in vec3 aPosition;
layout(location = 1) in vec3 aNormal;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;
uniform mat3 normalMatrix;

uniform float animTime;

out vec3 Normal;
out vec3 FragPos;

void main() {
    FragPos = vec3(model * vec4(aPosition, 1.0));
    Normal = normalMatrix * aNormal;
    gl_Position = projection * view * vec4(FragPos, 1.0);
}";

pub const FSHADER_SMOOTH: &str = 
"#version 300 es
precision mediump float;

in vec3 Normal;
in vec3 FragPos;
out vec4 outColor;

uniform vec3 lightPos;
uniform vec3 lightColor;
uniform vec3 objectColor;

uniform float animTime;

void main() {
    float ambientStrength = 0.1;
    vec3 ambient = objectColor * ambientStrength;

    vec3 lightDir = normalize(lightPos - FragPos);
    float diff = max(dot(normalize(Normal), lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    outColor = vec4((ambient + diffuse) * objectColor, 1.0);
}";

pub const VSHADER_LINE: &str = 
"#version 300 es
layout(location = 0) in vec3 aPosition;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

out vec3 FragPos;

void main() {
    FragPos = vec3(model * vec4(aPosition, 1.0));
    gl_Position = projection * view * vec4(FragPos, 1.0);
}";

pub const FSHADER_LINE: &str = 
"#version 300 es
precision mediump float;

in vec3 FragPos;
out vec4 outColor;

uniform vec3 objectColor;

void main() {
    outColor = vec4(objectColor, 1.0);
}";