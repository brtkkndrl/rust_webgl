use shaders::{FSHADER_FLAT, FSHADER_SMOOTH, VSHADER_FLAT, VSHADER_SMOOTH};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, WebGlBuffer, WebGlProgram, WebGl2RenderingContext, WebGlShader};
use web_sys::{window, console, Response};
use wasm_bindgen_futures::JsFuture;
use nalgebra::{Matrix3, Matrix4, Point3, UnitQuaternion, Vector3, Vector2};

mod mesh;
use mesh::Mesh;
mod shaders;

#[derive(PartialEq, Eq)]
enum ShadingType{
    Smooth,
    Flat
}

struct RenderedMesh{
    mesh: Mesh,
    shading: ShadingType,
    vbo: WebGlBuffer,
    ebo: WebGlBuffer,
    ebo_size : i32,
}

impl RenderedMesh{
    pub fn delete_gl_buffers(&self, gl: &WebGl2RenderingContext){
        gl.delete_buffer(Some(&(self.vbo)));
        gl.delete_buffer(Some(&(self.ebo)));
    }

    pub fn change_gl_buffers(&mut self, gl: &WebGl2RenderingContext)-> Result<(), String>{
        self.delete_gl_buffers(gl);

        let (vertices, indices) = match self.shading {
            ShadingType::Flat => self.mesh.create_primitive_buffers_flatshaded().unwrap(),
            ShadingType::Smooth => self.mesh.create_primitive_buffers().unwrap()
        };

        let (vbo, ebo) = RenderedMesh::create_gl_buffers(&vertices, &indices, gl).unwrap();

        self.vbo = vbo;
        self.ebo = ebo;
        self.ebo_size = indices.len() as i32;

        Ok(())
    }

    pub fn create_gl_buffers(vertices: &[f32], indices: &[u16], gl: &WebGl2RenderingContext) -> Result<(WebGlBuffer, WebGlBuffer), String>{
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

        Ok((vbo, ebo))
    }
}

struct ShaderPrograms{
    program_flat: WebGlProgram,
    program_smooth: WebGlProgram
}

impl ShaderPrograms{
    pub fn load_shaders(gl: &WebGl2RenderingContext) -> Result<ShaderPrograms, String>{
        let vshader_flat = compile_shader(&gl, GL::VERTEX_SHADER, VSHADER_FLAT,)?;
        let fshader_flat = compile_shader(&gl, GL::FRAGMENT_SHADER, FSHADER_FLAT)?;

        let vshader_smooth = compile_shader(&gl, GL::VERTEX_SHADER, VSHADER_SMOOTH,)?;
        let fshader_smooth = compile_shader(&gl, GL::FRAGMENT_SHADER, FSHADER_SMOOTH)?;
    
        let program_flat = link_program(&gl, &vshader_flat, &fshader_flat)?;
        let program_smooth = link_program(&gl, &vshader_smooth, &fshader_smooth)?;

        Ok(ShaderPrograms { program_flat: program_flat, program_smooth: program_smooth })
    }
}

#[wasm_bindgen]
pub struct Renderer {
    gl: GL,
    programs: ShaderPrograms,
    angle_x: f32,
    angle_y: f32,
    mouse_anchor: (i32, i32),
    is_mouse_down: bool,
    angle_anchor_x: f32,
    angle_anchor_y: f32,
    zoom_level: f32,
    rendered_mesh: Option<RenderedMesh>
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
            .dyn_into::<WebGl2RenderingContext>()?;

        gl.enable(GL::CULL_FACE);
        gl.cull_face(GL::BACK);
        gl.enable(WebGl2RenderingContext::DEPTH_TEST);

        let programs = ShaderPrograms::load_shaders(&gl)?;

        Ok(Renderer{
            gl,
            programs: programs,
            rendered_mesh: None,
            angle_x: 0.0,
            angle_y: 0.0,
            mouse_anchor: (0,0),
            is_mouse_down: false,
            angle_anchor_x: 0.0,
            angle_anchor_y: 0.0,
            zoom_level: 10.0
        })
    }

    #[wasm_bindgen]
    pub fn load_model(&mut self, mesh_str: String) -> Result<(), JsValue>{
        let gl = &(self.gl);
        
        if let Some(current_mesh) = self.rendered_mesh.take(){ // delete old gl buffers
            current_mesh.delete_gl_buffers(gl);
        }

        let mesh = Mesh::load_obj(&mesh_str).unwrap();

        // let (vertices, indices) = match self.shading {
        //     ShadingType::flat => self.mesh.create_primitive_buffers().unwrap(),
        //     ShadingType::smooth => self.mesh.create_primitive_buffers().unwrap()
        // };
        let (vertices, indices) = mesh.create_primitive_buffers().unwrap();

        let (vbo, ebo) = RenderedMesh::create_gl_buffers(&vertices, &indices, gl).unwrap();

        // let (vbo, ebo) = Renderer::create_gl_buffers(&vertices, &indices, gl).unwrap();

        self.rendered_mesh = Some(
            RenderedMesh { mesh: mesh, shading: ShadingType::Smooth, vbo: vbo, ebo: ebo, ebo_size: indices.len() as i32 }
        );

        console::log_1(&format!("displaying mesh {:?}v {:?}f", vertices.len()/3, indices.len()/3).into());

        Ok(())
    }

    #[wasm_bindgen]
    pub fn change_shading(&mut self, shading: String) -> Result<(), String>{
        if self.rendered_mesh.is_none() {
            return Err(format!("No mesh loaded!").into());
        }

        // console::log_1(&format!("{:?}", shading).into());
        if let Some(ref mut rendered_mesh) = self.rendered_mesh{
            match shading.as_str() {
                "smooth" => {
                    if rendered_mesh.shading == ShadingType::Smooth {
                        return Ok(());
                    }
                    rendered_mesh.shading = ShadingType::Smooth;
                },
                "flat" => {
                    if rendered_mesh.shading == ShadingType::Flat {
                        return Ok(());
                    }
                    rendered_mesh.shading = ShadingType::Flat;
                },
                _ => {
                    return Err(format!("Unrecognized shading: {}", shading).into());
                }
            }
            rendered_mesh.change_gl_buffers(&self.gl)?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn update(&mut self, mouse_down: bool, mouse_x: i32, mouse_y: i32, mouse_wheel: i32) ->Result<(), JsValue>{
        if mouse_down{
            if !self.is_mouse_down{ // set anchor
                self.mouse_anchor = (mouse_x, mouse_y);
                self.angle_anchor_y = self.angle_y;
                //self.angle_anchor_x = self.angle_x;
            } else { // move
                let mouse_vec = Vector2::new(
                    (self.mouse_anchor.0 - mouse_x) as f32, 
                    (self.mouse_anchor.1 - mouse_y) as f32);

                //self.angle_x = self.angle_anchor_x + (-mouse_vec.y as f32) * 0.0069;
                self.angle_y = self.angle_anchor_y + (-mouse_vec.x as f32) * 0.0069;
            }
        }
        self.is_mouse_down = mouse_down;

        self.zoom_level += (mouse_wheel as f32) * 0.5;
        self.zoom_level = self.zoom_level.clamp(0.0, 50.0);

        // console::log_1(&format!("displaying mesh {:?}", mouse_wheel).into());

        Ok(())
    }

    #[wasm_bindgen]
    pub fn render(&self) -> Result<(), JsValue> {
        if self.rendered_mesh.is_none() {
            return Err(format!("No mesh loaded!").into());
        }
        
        let gl = &(self.gl);

        if let Some(ref rendered_mesh) = self.rendered_mesh{
            let vbo = &rendered_mesh.vbo;
            let ebo = &rendered_mesh.ebo;

            let ebo_size = rendered_mesh.ebo_size;

            let program = match rendered_mesh.shading{
                ShadingType::Flat => {&self.programs.program_flat},
                ShadingType::Smooth => {&self.programs.program_smooth}
            };

            gl.use_program(Some(program));

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
                &Point3::new(0.0, 0.0, self.zoom_level), // Camera Position
                &Point3::new(0.0, 0.0, 0.0), // Look At
                &Vector3::new(0.0, 1.0, 0.0), // Up Vector
            );
            
            //TODO calculate movement size from anchor to new mouse pos distance
            //TODO calculate rotation axis (perpendicular to mouse movement, swap coords)

            let rotation_x = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), self.angle_x);
            let rotation_y = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.angle_y);
            
            let model: Matrix4<f32> = rotation_y.to_homogeneous() * rotation_x.to_homogeneous();

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
        }

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
