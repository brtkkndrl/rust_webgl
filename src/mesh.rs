use nalgebra::{Matrix4, Matrix3, Vector3, UnitQuaternion, Point3};
use std::array;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, WebGlBuffer, WebGlProgram, WebGlShader};
use web_sys::{window, console, Response};
use wasm_bindgen_futures::JsFuture;

pub struct Vertex{
    pos: Vector3<f32>,
    normal: Vector3<f32>
}

#[derive(Clone)]
pub struct Face{
    verts: Vec<u16>
}

pub struct Mesh{
    verts: Vec<Vertex>,
    faces: Vec<Face>,
    is_triangulated: bool
}

impl Mesh{
    pub fn basic_triangle() -> Result<Mesh, String>{
        
        let verts = vec![
            Vertex{pos: Vector3::new(-0.5, -0.5, 0.0), normal: Vector3::new(0.0, 0.0, 1.0)},
            Vertex{pos: Vector3::new(0.5, -0.5, 0.0), normal: Vector3::new(0.0, 0.0, 1.0)},
            Vertex{pos: Vector3::new(0.0,  0.5, 0.0), normal: Vector3::new(0.0, 0.0, 1.0)}
        ];
    
        let faces = vec![
            Face { verts: vec![0, 1, 2] }
        ];
    
        Ok(Mesh{verts: verts, faces: faces, is_triangulated: true})
    }

    pub async fn load_obj(obj_str: &String) -> Result<Mesh, &str>{
        let mut verts : Vec<Vertex> = vec![];
        let mut faces : Vec<Face> = vec![];

        let mut is_triangulated = true;

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
                    faces.push(Face{verts: temp_verts.clone()});

                    if temp_verts.len() > 3 {
                        is_triangulated = false;
                    }
                },
                "" => {},
                _ => {
                    console::log_1(&("Can't load obj").into());
                    return Err("Can't load obj")
                }
            }
        }
        
        console::log_1(&format!("loaded {:?}v {:?}f", verts.len(), faces.len()).into());
        let mut mesh = Mesh{verts: verts, faces: faces, is_triangulated: is_triangulated};
        mesh.triangulate_faces().unwrap();
        //mesh.compute_flatshaded().unwrap();
        Ok(mesh)
    }

    pub fn create_primitive_buffers(&self) -> Result<(Vec<f32>, Vec<u16>), &str>{
        if !self.is_triangulated{
            return Err("Mesh is not triangulated");
        }

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

    pub fn triangulate_faces(&mut self) ->Result<(), &str>{
        let mut new_faces: Vec<Face> = vec![];
        
        for face in &(self.faces){
            if face.verts.len() == 3{
                new_faces.push(face.clone());
            }else{
                let mut indices: Vec<u16> = vec![0, 0, 0];

                for i in 0..face.verts.len()-1{
                    indices[0] = face.verts[0];
                    indices[1] = face.verts[i];
                    indices[2] = face.verts[i+1];
                    new_faces.push(Face{verts: indices.clone()});
                }
            }
        }

        self.faces = new_faces;
        self.is_triangulated = true;
        Ok(())
    }

    pub fn create_primitive_buffers_flatshaded_redundant(&self) -> Result<(Vec<f32>, Vec<u16>), &str>{
        if !self.is_triangulated{
            return Err("Mesh is not triangulated");
        }

        let mut verts: Vec<f32> = vec![];
        
        let mut indices: Vec<u16> = vec![];

        for face in &(self.faces){
            let v1 = self.verts[face.verts[0] as usize].pos;
            let v2 = self.verts[face.verts[1] as usize].pos;
            let v3 = self.verts[face.verts[2] as usize].pos;

            let f_normal = (v2 - v1).cross(&(v3-v1)).normalize();

            for vert_id in &(face.verts){
                let vert_pos = self.verts[*vert_id as usize].pos;
                verts.push(vert_pos.x);
                verts.push(vert_pos.y);
                verts.push(vert_pos.z);

                if *vert_id == face.verts[2]{
                    verts.push(f_normal.x);
                    verts.push(f_normal.y);
                    verts.push(f_normal.z);
                }else{
                    verts.push(0.0);
                    verts.push(0.0);
                    verts.push(0.0);
                }

                indices.push(indices.len() as u16);
            }
        }

        Ok((verts, indices))
    }

    pub fn create_primitive_buffers_flatshaded(&self) -> Result<(Vec<f32>, Vec<u16>), &str>{
        if !self.is_triangulated{
            return Err("Mesh is not triangulated");
        }
        
        let vertex_count: usize = self.verts.len();
        let mut is_used:  Vec<bool> = vec![false; vertex_count]; // is vertex at that index used by some face
        
        let mut verts = vec![];
        let mut indices = vec![];

        for vert in &(self.verts){
            // pos
            verts.push(vert.pos.x);
            verts.push(vert.pos.y);
            verts.push(vert.pos.z);
            // normal
            verts.push(0.0);
            verts.push(0.0);
            verts.push(0.0);
        }

        let vert_attr_count = 6;

        for face in &(self.faces){ // assumes all faces are triangles
            let v1 = self.verts[face.verts[0] as usize].pos;
            let v2 = self.verts[face.verts[1] as usize].pos;
            let v3 = self.verts[face.verts[2] as usize].pos;

            let f_normal = (v2 - v1).cross(&(v3-v1)).normalize();

            let final_tri: (u16, u16, u16);

            if is_used[face.verts[2] as usize]{ // duplicate vertex
                // TODO try rearanging
                if !is_used[face.verts[0] as usize] {
                    final_tri = (face.verts[1], face.verts[2], face.verts[0]);
                    // is_used[face.verts[0] as usize] = true;
                } else if !is_used[face.verts[1] as usize] {
                    final_tri = (face.verts[2], face.verts[0], face.verts[1]);
                    // is_used[face.verts[1] as usize] = true;
                } else{
                    final_tri = (face.verts[2], face.verts[0], (verts.len() / vert_attr_count) as u16);// set to the last element, before pushing the vert!

                    verts.push(self.verts[face.verts[2] as usize].pos.x);
                    verts.push(self.verts[face.verts[2] as usize].pos.y);
                    verts.push(self.verts[face.verts[2] as usize].pos.z);
                    verts.push(f_normal.x);
                    verts.push(f_normal.y);
                    verts.push(f_normal.z);

                    console::log_1(&("duplicating").into());
                }
            }else{
                is_used[face.verts[2] as usize] = true;
                final_tri = (face.verts[0], face.verts[1], face.verts[2]);
                // update desired normal
            }
            
            let arr_index = (final_tri.2 as usize)*vert_attr_count;
            verts[arr_index+3] = f_normal.x;
            verts[arr_index+4] = f_normal.y;
            verts[arr_index+5] = f_normal.z;

            indices.push(final_tri.0);
            indices.push(final_tri.1);
            indices.push(final_tri.2);
        }

        Ok((verts, indices))
    }

    pub fn compute_flatshaded(&mut self) -> Result<(), &str>{
        if !self.is_triangulated{
            return Err("Mesh is not triangulated");
        }
        
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
