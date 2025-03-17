use nalgebra::{Matrix4, Matrix3, Vector3, UnitQuaternion, Point3};
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, WebGlBuffer, WebGlProgram, WebGlShader};
use web_sys::{window, console, Response};
use wasm_bindgen_futures::JsFuture;

pub struct Vertex{
    pos: Vector3<f32>,
    normal: Vector3<f32>
}

pub struct Face{
    verts: Vec<u16>
}

pub struct Mesh{
    verts: Vec<Vertex>,
    faces: Vec<Face>
}

impl Mesh{
    pub fn simple_triangle_mesh() -> Result<Mesh, String>{
        
        let verts = vec![
            Vertex{pos: Vector3::new(-0.5, -0.5, 0.0), normal: Vector3::new(0.0, 0.0, 1.0)},
            Vertex{pos: Vector3::new(0.5, -0.5, 0.0), normal: Vector3::new(0.0, 0.0, 1.0)},
            Vertex{pos: Vector3::new(0.0,  0.5, 0.0), normal: Vector3::new(0.0, 0.0, 1.0)}
        ];
    
        let faces = vec![
            Face { verts: vec![0, 1, 2] }
        ];
    
        Ok(Mesh{verts: verts, faces: faces})
    }

    pub async fn load_obj(path : &str) -> Result<Mesh, JsValue>{
        let resp = JsFuture::from(window().unwrap().fetch_with_str(path)).await?;
        let resp: Response = resp.dyn_into().unwrap();
        let text = JsFuture::from(resp.text()?).await?;
        let obj_str = text.as_string().unwrap();

        let mut verts : Vec<Vertex> = vec![];
        let mut faces : Vec<Face> = vec![];

        for line in obj_str.lines() {
            if line.trim() == "" || line.trim().starts_with("#"){
                continue;
            }

            let words: Vec<&str> = line.split(' ').collect();

            match words[0].trim() {
                "v" => 
                {
                    // console::log_1(&format!("vec {:?} {:?} {:?}", f32::from_str(words[1].trim()).unwrap(),
                    // f32::from_str(words[2].trim()).unwrap(),
                    // f32::from_str(words[3].trim()).unwrap()).into());

                    verts.push(Vertex{
                        pos: Vector3::new(
                            f32::from_str(words[1].trim()).unwrap(),
                            f32::from_str(words[2].trim()).unwrap(),
                            f32::from_str(words[3].trim()).unwrap()
                        ),
                        normal: Vector3::new(0.0,0.0,0.0)})
                },
                "f" => 
                {
                    //console::log_1(&format!("face").into());
                    let mut temp_verts : Vec<u16> = vec![];

                    for word in &words[1..] {
                        temp_verts.push(u16::from_str(&word.trim()).unwrap()-1);
                    }
                    faces.push(Face{verts: temp_verts});
                },
                "" => {},
                _ => {
                    console::log_1(&("Can't load obj").into());
                    return Err(("Can't load obj").into())
                }
            }
        }
        
        console::log_1(&format!("loaded {:?}f {:?}v", faces.len(), verts.len()).into());
        let mut mesh = Mesh{verts: verts, faces: faces};
        mesh.compute_flatshaded().unwrap();
        Ok(mesh)
    }

    pub fn create_primitive_buffers(&self) -> Result<(Vec<f32>, Vec<u16>), String>{
        let mut verts = vec![];
        
        let mut indices = vec![];

        for vert in &(self.verts){
            verts.push(vert.pos.x);
            verts.push(vert.pos.y);
            verts.push(vert.pos.z);

            verts.push(vert.normal.x);
            verts.push(vert.normal.y);
            verts.push(vert.normal.z);
        }

        for face in &(self.faces){
            if face.verts.len() > 3{
                for i in 0..face.verts.len()-1{
                    indices.push(face.verts[0]);
                    indices.push(face.verts[i]);
                    indices.push(face.verts[i+1]);
                }
            }else{
                for vert in &(face.verts){
                    indices.push(*vert);
                }
            }
        }

        Ok((verts, indices))
    }

    pub fn create_primitive_buffers_flatshaded(&self) -> Result<(Vec<f32>, Vec<u16>), String>{
        let mut verts = vec![];
        
        let mut indices = vec![];

        //TODO complete this

        // for vert in &(self.verts){
        //     verts.push(vert.pos.x);
        //     verts.push(vert.pos.y);
        //     verts.push(vert.pos.z);

        //     verts.push(vert.normal.x);
        //     verts.push(vert.normal.y);
        //     verts.push(vert.normal.z);
        // }

        // for face in &(self.faces){
        //     if face.verts.len() > 3{
        //         for i in 0..face.verts.len()-1{
        //             indices.push(face.verts[0]);
        //             indices.push(face.verts[i]);
        //             indices.push(face.verts[i+1]);
        //         }
        //     }else{
        //         for vert in &(face.verts){
        //             indices.push(*vert);
        //         }
        //     }
        // }

        Ok((verts, indices))
    }

    pub fn compute_flatshaded(&mut self) -> Result<(), String>{
        for face in &(self.faces){
            let v1 = self.verts[face.verts[0] as usize].pos;
            let v2 = self.verts[face.verts[1] as usize].pos;
            let v3 = self.verts[face.verts[2] as usize].pos;

            let f_normal = (v2 - v1).cross(&(v3-v1)).normalize();

            //console::log_1(&format!("face {:?}", f_normal).into());

            for vert in &(face.verts){
                self.verts[*vert as usize].normal = f_normal;
            }
        }
        
        Ok(())
    }
}
