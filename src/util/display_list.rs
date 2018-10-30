use crate::gl;

#[derive(Debug)]
pub struct DisplayList {
    idx: u32,
    num: u32,
    enum_idx: u32,
}

impl DisplayList {
    pub fn new(num: u32) -> Self {
        let idx;
        unsafe {
            idx = gl::GenLists(num as i32);
        }
        DisplayList {
            idx,
            num,
            enum_idx: idx,
        }
    }

    pub fn new_list(&self) {
        if self.enum_idx >= self.idx + self.num {
            panic!("Display list overflow (compile)!");
        }
        unsafe {
            gl::NewList(self.enum_idx, gl::GL_COMPILE);
        }
    }

    pub fn end_list(&mut self) {
        unsafe {
            gl::EndList();
        }
        self.enum_idx += 1;
    }

    pub fn call(&self, i: u32) {
        if i >= self.num {
            panic!("Display list overflow (call)!");
        }
        unsafe {
            gl::CallList(self.idx + i);
        }
    }
}
