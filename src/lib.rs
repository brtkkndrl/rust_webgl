use shaders::{FSHADER_FLAT, FSHADER_SMOOTH, VSHADER_FLAT, VSHADER_SMOOTH};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as GL, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader};
use web_sys::{window, console, Response};
use wasm_bindgen_futures::JsFuture;
use nalgebra::{Matrix3, Matrix4, Point2, Point3, Translation3, UnitQuaternion, Vector2, Vector3};

mod mesh;
use mesh::Mesh;

use crate::shaders::{FSHADER_LINE, VSHADER_LINE};
mod shaders;

#[derive(PartialEq, Eq)]
enum ShadingType{
    Smooth,
    Flat,
    Wireframe
}

struct RenderedMesh{
    mesh: Mesh,
    shading: ShadingType,
    gl_buffers: GLBuffers,
    bb_gl_buffers: GLBuffers // bounding box gl buffers
}

struct GLBuffers{
    vbo: WebGlBuffer,
    ebo: WebGlBuffer,
    ebo_size : i32
}

impl GLBuffers{
    pub fn delete(&self, gl: &WebGl2RenderingContext){
        gl.delete_buffer(Some(&(self.vbo)));
        gl.delete_buffer(Some(&(self.ebo)));
    }

    pub fn create(vertices: &[f32], indices: &[u16], gl: &WebGl2RenderingContext) -> Result<GLBuffers, String>{
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

        Ok(GLBuffers{vbo: vbo, ebo: ebo, ebo_size: indices.len() as i32})
    }
}

impl RenderedMesh{
    pub fn create(gl: &WebGl2RenderingContext, mesh: Mesh, shading: ShadingType) -> Result<RenderedMesh, String>{
        let (vertices, indices) = match shading {
            ShadingType::Flat => mesh.create_primitive_buffers_flatshaded()?,
            ShadingType::Smooth => mesh.create_primitive_buffers()?,
            ShadingType::Wireframe => mesh.create_primitive_buffers_wireframe()?
        };

        let gl_buffers = GLBuffers::create(&vertices, &indices, &gl)?;

        let (bb_vertices, bb_indices) = mesh.create_bb_primitive_buffers()?;
    
        let bb_gl_buffers = GLBuffers::create(&bb_vertices, &bb_indices, &gl)?;

        return Ok(RenderedMesh { mesh: mesh, shading: shading, gl_buffers: gl_buffers, bb_gl_buffers: bb_gl_buffers });
    }

    pub fn reload_gl_buffers(&mut self, gl: &WebGl2RenderingContext)-> Result<(), String> {
        self.gl_buffers.delete(gl);

        let (vertices, indices) = match self.shading {
            ShadingType::Flat => self.mesh.create_primitive_buffers_flatshaded()?,
            ShadingType::Smooth => self.mesh.create_primitive_buffers()?,
            ShadingType::Wireframe => self.mesh.create_primitive_buffers_wireframe()?
        };

        self.gl_buffers = GLBuffers::create(&vertices, &indices, gl).unwrap();

        Ok(())
    }
}

struct ShaderPrograms{
    program_flat: WebGlProgram,
    program_smooth: WebGlProgram,
    program_lines: WebGlProgram
}

impl ShaderPrograms{
    pub fn load_shaders(gl: &WebGl2RenderingContext) -> Result<ShaderPrograms, String>{
        let vshader_flat = compile_shader(&gl, GL::VERTEX_SHADER, VSHADER_FLAT)?;
        let fshader_flat = compile_shader(&gl, GL::FRAGMENT_SHADER, FSHADER_FLAT)?;
        let program_flat = link_program(&gl, &vshader_flat, &fshader_flat)?;

        let vshader_smooth = compile_shader(&gl, GL::VERTEX_SHADER, VSHADER_SMOOTH)?;
        let fshader_smooth = compile_shader(&gl, GL::FRAGMENT_SHADER, FSHADER_SMOOTH)?;
        let program_smooth = link_program(&gl, &vshader_smooth, &fshader_smooth)?;

        let vshader_lines = compile_shader(&gl, GL::VERTEX_SHADER, VSHADER_LINE)?;
        let fshader_lines = compile_shader(&gl, GL::FRAGMENT_SHADER, FSHADER_LINE)?;
        let program_lines = link_program(&gl, &vshader_lines, &fshader_lines)?;

        Ok(ShaderPrograms { program_flat: program_flat, program_smooth: program_smooth, program_lines: program_lines })
    }
}

#[wasm_bindgen]
pub struct Renderer {
    gl: GL,
    canvas : HtmlCanvasElement,
    programs: ShaderPrograms,
    mouse_anchor: Point2<i32>,
    is_mouse_down: bool,
    is_bb_visible: bool,
    rendered_mesh: Option<RenderedMesh>,
    camera: Camera,
    screen_dimensions: Vector2<i32>,
    last_normal_attrib_pos: i32
}

pub struct Camera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    angle_x_deg: f32,
    angle_y_deg: f32,
    zoom_level: f32,
    from_target_direction: Vector3<f32>
}

impl Camera {
    const MOUSE_SENSITIVITY: f32 = 0.33;
    const FOV: f32 = 45.0f32.to_radians();

    pub fn new(position: Point3<f32>, target: Point3<f32>, up: Vector3<f32>) -> Camera{
        return Camera { position: position, target: target, up: up, angle_x_deg: 0.0, angle_y_deg: 0.0, zoom_level: 10.0,
        from_target_direction: (position-target).normalize() }
    }

    pub fn projection_matrix(screen_dimensions: &Vector2<i32>) -> Matrix4<f32>{
        let aspect_ratio = (screen_dimensions.x as f32) / (screen_dimensions.y as f32);
        return Matrix4::new_perspective(aspect_ratio, Camera::FOV, 0.1, 100.0);
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        return Matrix4::look_at_rh(&self.position, &self.target, &self.up,);
    }

    fn update_position(&mut self){
        self.position = self.target + self.from_target_direction * self.zoom_level;
    }

    pub fn mouse_scroll(&mut self, mouse_wheel : f32) {
        self.zoom_level += mouse_wheel * 0.5;
        self.zoom_level = self.zoom_level.clamp(0.0, 50.0);
        self.update_position();
    }

    pub fn mouse_move(&mut self, mouse_move_vec: Vector2<f32>){
        self.angle_x_deg += mouse_move_vec.y * Camera::MOUSE_SENSITIVITY;
		self.angle_y_deg += mouse_move_vec.x * Camera::MOUSE_SENSITIVITY;

		self.angle_x_deg = self.angle_x_deg.clamp( -90.0 + 0.1, 90.0 - 0.1); // TODO should be self.angle_y_deg

        let target_to_self_quaternion =  
        UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.angle_y_deg.to_radians())
        * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), self.angle_x_deg.to_radians());

        self.from_target_direction = target_to_self_quaternion * Vector3::z_axis().into_inner();

        self.update_position();
    }
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
        
        let canvas_dom_width = canvas.client_width();
        let canvas_dom_height = canvas.client_height();

        canvas.set_width(canvas_dom_width as u32);
        canvas.set_height(canvas_dom_height as u32);

        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

        let programs = ShaderPrograms::load_shaders(&gl)?;

        Ok(Renderer{
            gl,
            canvas,
            programs: programs,
            rendered_mesh: None,
            mouse_anchor: Point2::new(0,0),
            is_mouse_down: false,
            is_bb_visible: false,
            camera : Camera::new(Point3::new(0.0, 0.0, 10.0), Point3::new(0.0,0.0,0.0), Vector3::new(0.0,1.0,0.0)),
            screen_dimensions: Vector2::new(canvas_dom_width, canvas_dom_height),
            last_normal_attrib_pos: -1
        })
    }

    #[wasm_bindgen]
    pub fn load_model(&mut self, mesh_str: String) -> Result<(), JsValue>{
        let gl = &(self.gl);
        
        let mut shading = ShadingType::Flat;

        if let Some(current_mesh) = self.rendered_mesh.take(){ // delete old gl buffers
            current_mesh.gl_buffers.delete(gl);
            current_mesh.bb_gl_buffers.delete(gl);
            shading = current_mesh.shading;
        }

        let mesh = Mesh::load_obj(&mesh_str)?;

        self.rendered_mesh = Some(RenderedMesh::create(gl, mesh, shading)?);

        //console::log_1(&format!("displaying mesh {:?}v {:?}f", vertices.len()/3, indices.len()/3).into());

        Ok(())
    }

    #[wasm_bindgen]
    pub fn resize_context(&mut self) -> Result<(), String>{

        let canvas_dom_width = self.canvas.client_width();
        let canvas_dom_height = self.canvas.client_height();

        self.canvas.set_width(canvas_dom_width as u32);
        self.canvas.set_height(canvas_dom_height as u32);

        self.gl.viewport(0, 0, self.canvas.width() as i32, self.canvas.height() as i32);

        self.screen_dimensions = Vector2::new(canvas_dom_width, canvas_dom_height);

        return Ok(());
    }

    #[wasm_bindgen]
    pub fn set_bb_visible(&mut self, visible: bool) -> Result<(), JsValue>{
        self.is_bb_visible = visible;
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
                "wireframe" => {
                    if rendered_mesh.shading == ShadingType::Wireframe {
                        return Ok(());
                    }
                    rendered_mesh.shading = ShadingType::Wireframe;
                },
                _ => {
                    return Err(format!("Unrecognized shading: {}", shading).into());
                }
            }
            rendered_mesh.reload_gl_buffers(&self.gl)?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn update(&mut self, mouse_down: bool, mouse_x: i32, mouse_y: i32, mouse_wheel: i32) ->Result<(), JsValue>{
        if mouse_down{
            if !self.is_mouse_down{ // set anchor
                self.mouse_anchor = Point2::new(mouse_x, mouse_y);
            } else { // move
                self.camera.mouse_move(Vector2::new(
                    (self.mouse_anchor.x - mouse_x) as f32, 
                    (self.mouse_anchor.y - mouse_y) as f32));
                self.mouse_anchor = Point2::new(mouse_x, mouse_y);
            }
        }
        self.is_mouse_down = mouse_down;

        self.camera.mouse_scroll(mouse_wheel as f32);

        // console::log_1(&format!("displaying mesh {:?}", mouse_wheel).into());

        Ok(())
    }

    fn pass_mvp_uniforms(&self, gl: &WebGl2RenderingContext, program: &WebGlProgram, model: &Matrix4<f32>, view: &Matrix4<f32>, projection: &Matrix4<f32>) -> Result<(), String>{    
        // Pass Uniforms
        let proj_loc = gl.get_uniform_location(program, "projection").unwrap();
        gl.uniform_matrix4fv_with_f32_array(Some(&proj_loc), false, projection.as_slice());
    
        let view_loc = gl.get_uniform_location(program, "view").unwrap();
        gl.uniform_matrix4fv_with_f32_array(Some(&view_loc), false, view.as_slice());
    
        let model_loc = gl.get_uniform_location(program, "model").unwrap();
        gl.uniform_matrix4fv_with_f32_array(Some(&model_loc), false, model.as_slice());

        Ok(())
    }

    #[wasm_bindgen]
    pub fn render(&mut self) -> Result<(), JsValue> {
        if self.rendered_mesh.is_none() {
            return Err(format!("No mesh loaded!").into());
        }
        
        let gl = &(self.gl);

        if let Some(ref rendered_mesh) = self.rendered_mesh{
            let vbo = &rendered_mesh.gl_buffers.vbo;
            let ebo = &rendered_mesh.gl_buffers.ebo;

            let ebo_size = rendered_mesh.gl_buffers.ebo_size;

            let program = match rendered_mesh.shading{
                ShadingType::Flat => {&self.programs.program_flat},
                ShadingType::Smooth => {&self.programs.program_smooth},
                ShadingType::Wireframe => {&self.programs.program_lines}
            };

            gl.use_program(Some(program));

            gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
            gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ebo));

            //console::log_1(&JsValue::from_str(&format!("ebo_size: {}", ebo_size)));
        
            // Vertex attributes
            if rendered_mesh.shading == ShadingType::Wireframe{ // just position attribute for wireframe                
                let pos_attrib = gl.get_attrib_location(&program, "aPosition") as u32;
                gl.vertex_attrib_pointer_with_i32(pos_attrib, 3, GL::FLOAT, false, 3 * 4, 0);
                gl.enable_vertex_attrib_array(pos_attrib);

                if self.last_normal_attrib_pos >= 0{
                    gl.disable_vertex_attrib_array(self.last_normal_attrib_pos as u32);
                    self.last_normal_attrib_pos = -1;
                }
            }else{ // position and normal for flat and smooth shading
                let pos_attrib = gl.get_attrib_location(&program, "aPosition") as u32;
                gl.vertex_attrib_pointer_with_i32(pos_attrib, 3, GL::FLOAT, false, 6 * 4, 0);
                gl.enable_vertex_attrib_array(pos_attrib);
        
                self.last_normal_attrib_pos = gl.get_attrib_location(&program, "aNormal");
                gl.vertex_attrib_pointer_with_i32(self.last_normal_attrib_pos as u32, 3, GL::FLOAT, false, 6 * 4, 3 * 4);
                gl.enable_vertex_attrib_array(self.last_normal_attrib_pos as u32);
            }
        
            let projection = Camera::projection_matrix(&(self.screen_dimensions));
            let view = self.camera.view_matrix();
            let model = Translation3::new(0.0, 0.0, 0.0).to_homogeneous();

            let object_color_loc = gl.get_uniform_location(&program, "objectColor").unwrap();
            gl.uniform3f(Some(&object_color_loc), 1.0, 1.0, 1.0);

            // Pass uniforms
            self.pass_mvp_uniforms(&gl, &program, &model, &view, &projection)?;

            if rendered_mesh.shading != ShadingType::Wireframe{
                // Extract the 3x3 normal matrix
                let normal_matrix = Matrix3::new(
                    model[(0, 0)], model[(0, 1)], model[(0, 2)], // First row
                    model[(1, 0)], model[(1, 1)], model[(1, 2)], // Second row
                    model[(2, 0)], model[(2, 1)], model[(2, 2)], // Third row
                );
            
                let normal_matrix = normal_matrix.try_inverse().unwrap().transpose();

                let normal_loc = gl.get_uniform_location(program, "normalMatrix").unwrap();
                gl.uniform_matrix3fv_with_f32_array(Some(&normal_loc), false, normal_matrix.as_slice());

                let light_pos_loc = gl.get_uniform_location(&program, "lightPos").unwrap();
                gl.uniform3f(Some(&light_pos_loc), self.camera.position.x, self.camera.position.y, self.camera.position.z);
        
                let light_color_loc = gl.get_uniform_location(&program, "lightColor").unwrap();
                gl.uniform3f(Some(&light_color_loc), 1.0, 1.0, 1.0);
            }

            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT | web_sys::WebGl2RenderingContext::DEPTH_BUFFER_BIT);

            if rendered_mesh.shading == ShadingType::Wireframe{
                gl.draw_elements_with_i32(GL::LINES, ebo_size, GL::UNSIGNED_SHORT, 0);
            }else{
                gl.draw_elements_with_i32(GL::TRIANGLES, ebo_size, GL::UNSIGNED_SHORT, 0);
            }

            if self.is_bb_visible{            //render bounding box
                let bb_vbo = &rendered_mesh.bb_gl_buffers.vbo;
                let bb_ebo = &rendered_mesh.bb_gl_buffers.ebo;

                let bb_ebo_size = rendered_mesh.bb_gl_buffers.ebo_size;

                let bb_program = &self.programs.program_lines;

                gl.use_program(Some(bb_program));

                gl.bind_buffer(GL::ARRAY_BUFFER, Some(&bb_vbo));
                gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&bb_ebo));

                let bb_pos_attrib = gl.get_attrib_location(&bb_program, "aPosition") as u32;
                gl.vertex_attrib_pointer_with_i32(bb_pos_attrib, 3, GL::FLOAT, false, 3 * 4, 0);
                gl.enable_vertex_attrib_array(bb_pos_attrib);

                let bb_object_color_loc = gl.get_uniform_location(&bb_program, "objectColor").unwrap();
                gl.uniform3f(Some(&bb_object_color_loc), 1.0, 0.0, 0.0);

                self.pass_mvp_uniforms(&gl, &bb_program, &model, &view, &projection)?;

                gl.draw_elements_with_i32(GL::LINES, bb_ebo_size, GL::UNSIGNED_SHORT, 0);
            }
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
