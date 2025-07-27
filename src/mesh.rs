use nalgebra::{Vector3};
use std::{f32::{INFINITY, NEG_INFINITY}, str::FromStr};
use web_sys::{console};
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

impl Mesh{
    pub fn load_obj(obj_str: &String) -> Result<Mesh, String>{
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
                            f32::from_str(words[1].trim()).map_err(|e| e.to_string())?,
                            f32::from_str(words[2].trim()).map_err(|e| e.to_string())?,
                            f32::from_str(words[3].trim()).map_err(|e| e.to_string())?
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
                    return Err("Can't load obj".to_string())
                }
            }
        }
        
        console::log_1(&format!("loaded {:?}v {:?}f", verts.len(), faces.len()).into());
        let mut mesh = Mesh{verts: verts, faces: faces, is_triangulated: is_triangulated, bb_min: Vector3::new(0.0,0.0,0.0), bb_max: Vector3::new(0.0,0.0,0.0)};
        mesh.derrive_normals_from_faces().unwrap();
        mesh.triangulate_faces().unwrap();
        mesh.move_pivot_to_center();
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
