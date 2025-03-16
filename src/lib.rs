use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, WebGlProgram, WebGlShader};
use web_sys::{window, console, Response};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;
extern crate nalgebra_glm as glm;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use lazy_static::lazy_static;
use glm::Mat4;

#[wasm_bindgen]
pub struct Renderer {
    context: GL,
    program: WebGlProgram,
}


fn get_gl_ctx() -> Result<GL, JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into()?;
    let gl = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<web_sys::WebGl2RenderingContext>()?;

    Ok(gl)
}
// #[wasm_bindgen]
// pub fn render(){
//     let gl = get_gl_ctx().unwrap_or_else(|e| {
//         console::log_1(&format!("Error compiling shader: {:?}", e).into());
//         std::process::exit(1);
//     });

//     let model : Mat4 = glm::identity();

//     let normal_matrix = glm::mat3(
//         model[(0, 0)], model[(0, 1)], model[(0, 2)], // First row
//         model[(1, 0)], model[(1, 1)], model[(1, 2)], // Second row
//         model[(2, 0)], model[(2, 1)], model[(2, 2)], // Third row
//     );

//     // Compute normal matrix (3x3)
//     let normal_matrix = normal_matrix.try_inverse().unwrap().transpose();

//     // unsafe {
//     //     let program = GL_PROGRAM.unwrap();
//     // }

//     // let model_loc = gl.get_uniform_location(&program, "model").unwrap();
//     // gl.uniform_matrix4fv_with_f32_array(Some(&model_loc), false, model.as_slice());

//     // let normal_loc = gl.get_uniform_location(&program, "normalMatrix").unwrap();
//     // gl.uniform_matrix3fv_with_f32_array(Some(&normal_loc), false, normal_matrix.as_slice());

//     gl.clear_color(0.0, 0.0, 0.0, 1.0);
//     gl.clear(GL::COLOR_BUFFER_BIT);
//     gl.draw_elements_with_i32(GL::TRIANGLES, 3, GL::UNSIGNED_SHORT, 0);
// }

#[wasm_bindgen]
pub async fn load_obj(path : &str) -> Result<(), JsValue> {
    let resp = JsFuture::from(window().unwrap().fetch_with_str(path)).await?;
    let resp: Response = resp.dyn_into().unwrap();
    let text = JsFuture::from(resp.text()?).await?;
    let obj_str = text.as_string().unwrap();

    // Print each line to the JS console
    for _line in obj_str.lines() {
        //console::log_1(&line.into());
    }

    Ok(())
}

#[wasm_bindgen]
pub fn anim_frame() {
    console::log_1(&"anim frame".into());
}

#[wasm_bindgen]
pub fn init_buffers(gl: &GL) -> Result<(), JsValue> {
    let vert_shader = compile_shader(
        &gl,
        GL::VERTEX_SHADER,
        "#version 300 es
        layout(location = 0) in vec3 aPosition;
        layout(location = 1) in vec3 aNormal;

        uniform mat4 projection;
        uniform mat4 view;
        uniform mat4 model;
        uniform mat3 normalMatrix;

        out vec3 Normal;
        out vec3 FragPos;

        void main() {
            FragPos = vec3(model * vec4(aPosition, 1.0));
            Normal = normalMatrix * aNormal;
            gl_Position = projection * view * vec4(FragPos, 1.0);
        }
        ",
    ).unwrap_or_else(|e| {
        console::log_1(&format!("Error compiling shader: {:?}", e).into());
        std::process::exit(1);
    });

    let frag_shader = compile_shader(
        &gl,
        GL::FRAGMENT_SHADER,
        "#version 300 es
        precision mediump float;

        in vec3 Normal;
        in vec3 FragPos;
        out vec4 outColor;

        uniform vec3 lightPos;
        uniform vec3 lightColor;
        uniform vec3 objectColor;

        void main() {
            float ambientStrength = 0.1;
            vec3 ambient = objectColor * ambientStrength;

            vec3 lightDir = normalize(lightPos - FragPos);
            float diff = max(dot(normalize(Normal), lightDir), 0.0);
            vec3 diffuse = diff * lightColor;

            outColor = vec4((ambient + diffuse) * objectColor, 1.0);
        }
        ",
    ).unwrap_or_else(|e| {
        console::log_1(&format!("Error compiling shader: {:?}", e).into());
        std::process::exit(1);
    });

    let program = link_program(&gl, &vert_shader, &frag_shader).unwrap_or_else(|e| {
        console::log_1(&format!("Error compiling shader: {:?}", e).into());
        std::process::exit(1);
    });

    gl.use_program(Some(&program));


    // Define vertex data (x, y, z, nx, ny, nz)
    let vertices: [f32; 18] = [
        -0.5, -0.5, 0.0,  0.0, 0.0, 1.0,  // Vertex 1
         0.5, -0.5, 0.0,  0.0, 0.0, 1.0,  // Vertex 2
         0.0,  0.5, 0.0,  0.0, 0.0, 1.0,  // Vertex 3
    ];

    let indices: [u16; 3] = [0, 1, 2];

    // Create & Bind VBO
    let vbo = gl.create_buffer().ok_or("Failed to create buffer")?;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
    unsafe {
        let vertex_array = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vertex_array, GL::STATIC_DRAW);
    }

    // Create & Bind EBO
    let ebo = gl.create_buffer().ok_or("Failed to create element buffer")?;
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ebo));
    unsafe {
        let index_array = js_sys::Uint16Array::view(&indices);
        gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &index_array, GL::STATIC_DRAW);
    }

    // Enable Position Attribute
    let pos_attrib = gl.get_attrib_location(&program, "aPosition") as u32;
    gl.vertex_attrib_pointer_with_i32(pos_attrib, 3, GL::FLOAT, false, 6 * 4, 0);
    gl.enable_vertex_attrib_array(pos_attrib);

    // Enable Normal Attribute
    let normal_attrib = gl.get_attrib_location(&program, "aNormal") as u32;
    gl.vertex_attrib_pointer_with_i32(normal_attrib, 3, GL::FLOAT, false, 6 * 4, 3 * 4);
    gl.enable_vertex_attrib_array(normal_attrib);

    // Camera Setup
    let projection = glm::perspective(1.0, 45.0_f32.to_radians(), 0.1, 100.0);
    let view = glm::look_at(
        &glm::vec3(0.0, 0.0, 3.0),  // Camera Position
        &glm::vec3(0.0, 0.0, 0.0),  // Look At
        &glm::vec3(0.0, 1.0, 0.0),  // Up Vector
    );
    let model : Mat4 = glm::identity();

    let normal_matrix = glm::mat3(
        model[(0, 0)], model[(0, 1)], model[(0, 2)], // First row
        model[(1, 0)], model[(1, 1)], model[(1, 2)], // Second row
        model[(2, 0)], model[(2, 1)], model[(2, 2)], // Third row
    );

    // Compute normal matrix (3x3)
    let normal_matrix = normal_matrix.try_inverse().unwrap().transpose();

    // Pass Uniforms
    let proj_loc = gl.get_uniform_location(&program, "projection").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&proj_loc), false, projection.as_slice());

    let view_loc = gl.get_uniform_location(&program, "view").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&view_loc), false, view.as_slice());

    let light_pos_loc = gl.get_uniform_location(&program, "lightPos").unwrap();
    gl.uniform3f(Some(&light_pos_loc), 1.2, 1.0, 2.0);

    let light_color_loc = gl.get_uniform_location(&program, "lightColor").unwrap();
    gl.uniform3f(Some(&light_color_loc), 1.0, 1.0, 1.0);

    let object_color_loc = gl.get_uniform_location(&program, "objectColor").unwrap();
    gl.uniform3f(Some(&object_color_loc), 1.0, 0.5, 0.31);

    let model_loc = gl.get_uniform_location(&program, "model").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&model_loc), false, model.as_slice());

    let normal_loc = gl.get_uniform_location(&program, "normalMatrix").unwrap();
    gl.uniform_matrix3fv_with_f32_array(Some(&normal_loc), false, normal_matrix.as_slice());


    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL::COLOR_BUFFER_BIT);
    gl.draw_elements_with_i32(GL::TRIANGLES, 3, GL::UNSIGNED_SHORT, 0);

    Ok(())

}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let gl = get_gl_ctx().unwrap();

    init_buffers(&gl).unwrap();

    spawn_local(async {
        match load_obj("assets/teapot.obj").await {
            Ok(_) => {}
            Err(e) => {
                console::error_1(&e);
            }
        }
    });

    Ok(())
}

fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = gl.create_shader(shader_type).ok_or("Failed to create shader")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if gl.get_shader_parameter(&shader, GL::COMPILE_STATUS).as_bool().unwrap_or(false) {
        Ok(shader)
    } else {
        Err(gl.get_shader_info_log(&shader).unwrap_or("Unknown error".into()))
    }
}

fn link_program(gl: &GL, vert_shader: &WebGlShader, frag_shader: &WebGlShader) -> Result<WebGlProgram, String> {
    let program = gl.create_program().ok_or("Failed to create program")?;
    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);
    if gl.get_program_parameter(&program, GL::LINK_STATUS).as_bool().unwrap_or(false) {
        Ok(program)
    } else {
        Err(gl.get_program_info_log(&program).unwrap_or("Unknown error".into()))
    }
}
