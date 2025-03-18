use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, WebGlBuffer, WebGlProgram, WebGlShader};
use web_sys::{window, console, Response};
use wasm_bindgen_futures::JsFuture;
use nalgebra::{Matrix4, Matrix3, Vector3, UnitQuaternion, Point3};

mod mesh;
use mesh::Mesh;

#[wasm_bindgen]
pub struct Renderer {
    gl: GL,
    program: Option<WebGlProgram>,
    vbo: Option<WebGlBuffer>,
    ebo: Option<WebGlBuffer>,
    ebo_size : i32,
    angle: f32
}

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Renderer, JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into()?;
        let gl = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>()?;

        gl.enable(GL::CULL_FACE);
        gl.cull_face(GL::BACK);
        gl.enable(web_sys::WebGl2RenderingContext::DEPTH_TEST);

        Ok(Renderer{
            gl,
            program : None,
            vbo : None, ebo : None,
            ebo_size: 0,
            angle: 0.0
        })
    }

    #[wasm_bindgen]
    pub fn load_shaders(&mut self) -> Result<(), JsValue>{
        let gl = &(self.gl);

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
    
            flat out vec3 Normal;
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
    
            flat in vec3 Normal;
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

        self.program = Some(program);

        Ok(())
    }

    #[wasm_bindgen]
    pub async fn load_mesh(&mut self) -> Result<(), JsValue>{
        //let mesh = Mesh::simple_triangle_mesh().unwrap();
        let mesh_str = fetch_resource_as_str("assets/gear.obj").await.unwrap();

        let mesh = Mesh::load_obj(&mesh_str).await.unwrap();

        let (vertices, indices) = mesh.create_primitive_buffers_flatshaded().unwrap();

        let gl = &(self.gl);

        let vbo = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
        unsafe {
            let vertex_array = js_sys::Float32Array::view(&vertices);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vertex_array, GL::STATIC_DRAW);
        }
        
        let ebo = gl.create_buffer().ok_or("Failed to create element buffer")?;
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ebo));
        unsafe {
            let index_array = js_sys::Uint16Array::view(&indices);
            gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &index_array, GL::STATIC_DRAW);
        }

        self.vbo = Some(vbo);
        self.ebo = Some(ebo);

        self.ebo_size = indices.len() as i32;

        console::log_1(&format!("displaying mesh {:?}v {:?}f", vertices.len()/3, indices.len()/3).into());

        Ok(())
    }

    #[wasm_bindgen]
    pub fn update(&mut self) ->Result<(), JsValue>{
        self.angle += 0.01;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn render(&self) -> Result<(), JsValue> {
        let gl = &(self.gl);
        let program = self.program.as_ref().unwrap();
        
        let vbo = self.vbo.as_ref().unwrap();
        let ebo = self.ebo.as_ref().unwrap();

        let ebo_size = self.ebo_size;

        gl.use_program(Some(&program));

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ebo));
    
        // Enable Position Attribute
        let pos_attrib = gl.get_attrib_location(&program, "aPosition") as u32;
        gl.vertex_attrib_pointer_with_i32(pos_attrib, 3, GL::FLOAT, false, 6 * 4, 0);
        gl.enable_vertex_attrib_array(pos_attrib);
    
        // Enable Normal Attribute
        let normal_attrib = gl.get_attrib_location(&program, "aNormal") as u32;
        gl.vertex_attrib_pointer_with_i32(normal_attrib, 3, GL::FLOAT, false, 6 * 4, 3 * 4);
        gl.enable_vertex_attrib_array(normal_attrib);
    
        let projection = Matrix4::new_perspective(45.0f32.to_radians(), 1.0, 0.1, 100.0);

        let view = Matrix4::look_at_rh(
            &Point3::new(0.0, 0.0, 10.0), // Camera Position
            &Point3::new(0.0, 0.0, 0.0), // Look At
            &Vector3::new(0.0, 1.0, 0.0), // Up Vector
        );
        
        // Model transformation with rotation
        let model: Matrix4<f32> = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.angle)
            .to_homogeneous();
        
        // Extract the 3x3 normal matrix
        let normal_matrix = Matrix3::new(
            model[(0, 0)], model[(0, 1)], model[(0, 2)], // First row
            model[(1, 0)], model[(1, 1)], model[(1, 2)], // Second row
            model[(2, 0)], model[(2, 1)], model[(2, 2)], // Third row
        );
        
        let normal_matrix = normal_matrix.try_inverse().unwrap().transpose();
    
        // Pass Uniforms
        let proj_loc = gl.get_uniform_location(&program, "projection").unwrap();
        gl.uniform_matrix4fv_with_f32_array(Some(&proj_loc), false, projection.as_slice());
    
        let view_loc = gl.get_uniform_location(&program, "view").unwrap();
        gl.uniform_matrix4fv_with_f32_array(Some(&view_loc), false, view.as_slice());
    
        let light_pos_loc = gl.get_uniform_location(&program, "lightPos").unwrap();
        gl.uniform3f(Some(&light_pos_loc), 0.0, 0.0, 50.0);
    
        let light_color_loc = gl.get_uniform_location(&program, "lightColor").unwrap();
        gl.uniform3f(Some(&light_color_loc), 1.0, 1.0, 1.0);
    
        let object_color_loc = gl.get_uniform_location(&program, "objectColor").unwrap();
        gl.uniform3f(Some(&object_color_loc), 1.0, 1.0, 1.0);
    
        let model_loc = gl.get_uniform_location(&program, "model").unwrap();
        gl.uniform_matrix4fv_with_f32_array(Some(&model_loc), false, model.as_slice());
    
        let normal_loc = gl.get_uniform_location(&program, "normalMatrix").unwrap();
        gl.uniform_matrix3fv_with_f32_array(Some(&normal_loc), false, normal_matrix.as_slice());
    
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT | web_sys::WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        gl.draw_elements_with_i32(GL::TRIANGLES, ebo_size, GL::UNSIGNED_SHORT, 0);

        Ok(())
    }
}

pub async fn fetch_resource_as_str(path : &str) -> Result<String, JsValue>{
    let resp = JsFuture::from(window().unwrap().fetch_with_str(path)).await?;
    let resp: Response = resp.dyn_into().unwrap();
    let text = JsFuture::from(resp.text()?).await?;
    let obj_str = text.as_string().unwrap();
    Ok(obj_str)
}


#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
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
