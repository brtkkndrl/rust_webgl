use nalgebra::{Point3, Vector3};
use std::{f32::{INFINITY, NEG_INFINITY}, str::FromStr};
use web_sys::{console};
use std::collections::HashMap;

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
    is_triangulated: bool,
    bb_min: Vector3<f32>,
    bb_max: Vector3<f32>
}

#[derive(PartialEq, Eq)]
enum ParsingState{
    ParsingVerts,
    ParsingNormals,
    ParsingFacesSimple,
    ParsingFacesComplex
}

impl Mesh{
    pub fn load_obj(obj_str: &String) -> Result<Mesh, String>{
        let mut obj_vertices: Vec<Vector3<f32>> = vec![];
        let mut obj_normals: Vec<Vector3<f32>> = vec![];
        let mut obj_faces: Vec<Vec<(i32, i32, i32)>> = vec![];

        let mut verts : Vec<Vertex> = vec![];
        let mut faces : Vec<Face> = vec![];

        let mut is_triangulated = true;
        let mut has_normals_defined = false;
        let mut found_simple_face_def = false;
        let mut found_complex_face_def = false;

        // let mut parsing_state: ParsingState = ParsingState::ParsingVerts;

        for line in obj_str.lines() {
            if line.trim() == "" || line.trim().starts_with("#"){
                continue;
            }

            let words: Vec<&str> = line.split(' ').collect();

            let first_word = words[0].trim();

        //     match parsing_state{
        //         ParsingState::ParsingVerts => {
        //             if first_word == "vn"{parsing_state = ParsingState::ParsingNormals};

        //             if first_word != "v"{return Err(format!("Expected vertex definition, found: {first_word}").to_string());}

        //             obj_vertices.push(Vector3::new(
        //                     f32::from_str(words[1].trim()).map_err(|e| e.to_string())?,
        //                     f32::from_str(words[2].trim()).map_err(|e| e.to_string())?,
        //                     f32::from_str(words[3].trim()).map_err(|e| e.to_string())?
        //             ));
        //         },
        //         ParsingState::ParsingNormals => {
        //             if first_word != "vn"{return Err(format!("Expected normal definition, found: {first_word}").to_string());}
        //         },
        //         ParsingState::ParsingFacesSimple => {
        //             if first_word != "f"{return Err(format!("Expected simple face definition, found: {first_word}").to_string());}
        //         },
        //         ParsingState::ParsingFacesComplex => {
        //             if first_word != "f"{return Err(format!("Expected complex face definition, found: {first_word}").to_string());}
        //         },
        //     }

            match first_word {
                "v" => 
                {
                    obj_vertices.push(Vector3::new(
                            f32::from_str(words[1].trim()).map_err(|e| e.to_string())?,
                            f32::from_str(words[2].trim()).map_err(|e| e.to_string())?,
                            f32::from_str(words[3].trim()).map_err(|e| e.to_string())?
                    ));
                },
                "vn" =>{
                    obj_normals.push(Vector3::new(
                            f32::from_str(words[1].trim()).map_err(|e| e.to_string())?,
                            f32::from_str(words[2].trim()).map_err(|e| e.to_string())?,
                            f32::from_str(words[3].trim()).map_err(|e| e.to_string())?
                    ));
                }
                "f" => 
                {
                    let mut obj_face: Vec<(i32, i32, i32)> = vec![];
                    if line.contains("/"){// vert/texture/normal
                        if found_simple_face_def{return Err("Invalid face definition, expected simple".to_string())}

                        found_complex_face_def = true;

                        for word in &words[1..] {
                            let parts: Vec<&str> = word.trim().split('/').collect();

                            if parts.len() != 3{return Err("Invalid face definition".to_string())}

                            let vert_index = i32::from_str(&parts[0]).unwrap()-1;
                            let normal_index = i32::from_str(&parts[2]).unwrap()-1;

                            obj_face.push((vert_index, -1, normal_index));
                        }
                    }else{//simple definition
                        if found_complex_face_def{return Err("Invalid face definition, expected complex".to_string())}

                        found_simple_face_def = true;

                        for word in &words[1..] {
                            let vert_index = i32::from_str(&word.trim()).unwrap()-1;
                            obj_face.push((vert_index, -1, -1));
                        }
                    }
                    obj_faces.push(obj_face);


                    // let mut temp_verts : Vec<u16> = vec![];
                    // for word in &words[1..] {
                    //     temp_verts.push(u16::from_str(&word.trim()).unwrap()-1);
                    // }
                    // faces.push(Face{verts: temp_verts.clone()});

                    // if temp_verts.len() > 3 {
                    //     is_triangulated = false;
                    // }

                    //console::log_1(&format!("face").into());
                },
                "" => {},
                _ => {
                    // console::log_1(&("Can't load obj").into());
                    return Err(format!("Unexpected character: {first_word}").to_string())
                }
            }
        }

        assert!(!(found_simple_face_def && found_complex_face_def));

        // Transform obj_verts, obj_normals, and obj_faces into Vertex, and Face vectors

        if found_simple_face_def{
            for obj_vert in obj_vertices{
                verts.push(Vertex { pos: obj_vert, normal: Vector3::new(0.0,0.0,0.0) });
            }
            for obj_face in obj_faces{
                let mut temp_vert_ids : Vec<u16> = vec![];
                for vert_uv_normal_def in obj_face{
                    temp_vert_ids.push(vert_uv_normal_def.0 as u16);
                }
                if temp_vert_ids.len() > 3{is_triangulated = false;}
                faces.push(Face{verts: temp_vert_ids.clone()});
            }
        }else if found_complex_face_def{
            has_normals_defined = true;

            let mut indexes_to_vert_ids: HashMap<(i32, i32, i32), u16> = HashMap::new();

            for obj_face in obj_faces{
                let mut temp_vert_ids : Vec<u16> = vec![];

                for vert_uv_normal_def in obj_face{
                    if let Some(vert_id) = indexes_to_vert_ids.get(&vert_uv_normal_def){//already exists
                        temp_vert_ids.push(vert_id.clone());
                    }else{
                        verts.push(Vertex { pos: obj_vertices[vert_uv_normal_def.0 as usize],
                             normal: obj_normals[vert_uv_normal_def.2 as usize] });
                        let new_vert_index =  u16::try_from(verts.len() - 1).expect("Exceeded vert limit");
                        indexes_to_vert_ids.insert(vert_uv_normal_def, new_vert_index);
                        temp_vert_ids.push(new_vert_index);
                    }
                }

                if temp_vert_ids.len() > 3{is_triangulated = false;}
                faces.push(Face{verts: temp_vert_ids.clone()});
            }
        }
        
        let mut mesh = Mesh{verts: verts, faces: faces, is_triangulated: is_triangulated,
            bb_min: Vector3::new(0.0,0.0,0.0), bb_max: Vector3::new(0.0,0.0,0.0)};
        if found_simple_face_def{
            mesh.derrive_normals_from_faces()?;
        }
        mesh.triangulate_faces()?;
        mesh.move_pivot_to_center();
        
        console::log_1(&format!("loaded {:?}v {:?}f", mesh.verts.len(), mesh.faces.len()).into());
        console::log_1(&format!("was triangulated: {is_triangulated}").into());
        console::log_1(&format!("had normals: {found_complex_face_def}").into());
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
            if face.verts.len() > 3{ // TODO it should not happen as there is triangulation test
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

    pub fn create_primitive_buffers_wireframe(&self) -> Result<(Vec<f32>, Vec<u16>), &str>{
        if !self.is_triangulated{
            return Err("Mesh is not triangulated");
        }

        let mut verts = vec![];
        
        let mut indices = vec![];

        for vert in &(self.verts){
            verts.push(vert.pos.x);
            verts.push(vert.pos.y);
            verts.push(vert.pos.z);
        }

        for face in &(self.faces){
            indices.push(face.verts[0]); indices.push(face.verts[1]);
            indices.push(face.verts[1]); indices.push(face.verts[2]);
            indices.push(face.verts[2]); indices.push(face.verts[0]);
        }

        Ok((verts, indices))
    }

    fn compute_bounds(&self) -> (Vector3<f32>, Vector3<f32>){
        let (mut min_x, mut min_y, mut min_z) =  (INFINITY, INFINITY, INFINITY);
        let (mut max_x, mut max_y, mut max_z) =  (NEG_INFINITY, NEG_INFINITY, NEG_INFINITY);

        for vert in &(self.verts){
            min_x = min_x.min(vert.pos.x);
            min_y = min_y.min(vert.pos.y);
            min_z = min_z.min(vert.pos.z);
            //
            max_x = max_x.max(vert.pos.x);
            max_y = max_y.max(vert.pos.y);
            max_z = max_z.max(vert.pos.z);
        }

        return (Vector3::new(min_x, min_y, min_z), Vector3::new(max_x, max_y, max_z));
    }

    pub fn create_bb_primitive_buffers(&self) -> Result<(Vec<f32>, Vec<u16>), &str>{
        let mut verts = vec![];
        let mut indices = vec![];

        let (min_x, min_y, min_z) = (self.bb_min.x, self.bb_min.y, self.bb_min.z);
        let (max_x, max_y, max_z) = (self.bb_max.x, self.bb_max.y, self.bb_max.z);

        //

        verts.push(min_x); verts.push(min_y); verts.push(min_z);//000
        verts.push(min_x); verts.push(min_y); verts.push(max_z);//001
        verts.push(min_x); verts.push(max_y); verts.push(min_z);//010
        verts.push(min_x); verts.push(max_y); verts.push(max_z);//011
        verts.push(max_x); verts.push(min_y); verts.push(min_z);//100
        verts.push(max_x); verts.push(min_y); verts.push(max_z);//101
        verts.push(max_x); verts.push(max_y); verts.push(min_z);//110
        verts.push(max_x); verts.push(max_y); verts.push(max_z);//111

        //

        indices.extend_from_slice(&[
            0,1, 1,5, 5,4, 4,0,
            2,3, 3,7, 7,6, 6,2,
            0,2, 1,3, 5,7, 4,6
        ]);

        Ok((verts, indices))
    }

    pub fn move_pivot_to_center(&mut self) {
        let (bb_min, bb_max) = self.compute_bounds();
        let bb_center = (bb_max + bb_min) / 2.0;

         for vert in &mut (self.verts){
            // pos
            vert.pos -= bb_center;
        }

        self.bb_min = bb_min-bb_center;
        self.bb_max = bb_max-bb_center;
    }

    pub fn triangulate_faces(&mut self) ->Result<(), &str>{
        if self.is_triangulated{
            return Ok(());
        }

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
                // if !is_used[face.verts[0] as usize] {
                //     final_tri = (face.verts[1], face.verts[2], face.verts[0]);
                //     is_used[face.verts[0] as usize] = true;
                // } else if !is_used[face.verts[1] as usize] {
                //     final_tri = (face.verts[2], face.verts[0], face.verts[1]);
                //     is_used[face.verts[1] as usize] = true;
                // } else{
                    final_tri = (face.verts[0], face.verts[1], (verts.len() / vert_attr_count) as u16);// set to the last element, before pushing the vert!

                    verts.push(self.verts[face.verts[2] as usize].pos.x);
                    verts.push(self.verts[face.verts[2] as usize].pos.y);
                    verts.push(self.verts[face.verts[2] as usize].pos.z);
                    verts.push(f_normal.x);
                    verts.push(f_normal.y);
                    verts.push(f_normal.z);

                    // console::log_1(&("duplicating").into());
                // }
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

        console::log_1(&format!("flatshaded: {}v", verts.len()).into());

        Ok((verts, indices))
    }

    /// fills in vert normals from face normals
    pub fn derrive_normals_from_faces(&mut self) -> Result<(), &str>{
        for vert in &mut self.verts{
            vert.normal = Vector3::zeros();
        }

        for face in &(self.faces){
            let v1 = self.verts[face.verts[0] as usize].pos;
            let v2 = self.verts[face.verts[1] as usize].pos;
            let v3 = self.verts[face.verts[2] as usize].pos;

            let f_normal = (v2 - v1).cross(&(v3-v1)).normalize();

            for vert in &(face.verts){
                self.verts[*vert as usize].normal += f_normal;
            }
        }

        for vert in &mut self.verts{
            vert.normal.normalize();
        }

        Ok(())
    }
}
