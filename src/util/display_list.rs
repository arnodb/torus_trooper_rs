use crate::gl;

#[derive(Debug)]
pub struct DisplayList {
    num: u32,
    idx: u32,
    enum_idx: u32,
}

impl DisplayList {
    pub fn new(num: u32) -> Self {
        let idx;
        unsafe {
            idx = gl::GenLists(num as i32);
        }
        DisplayList {
            num,
            idx,
            enum_idx: idx,
        }
    }

    pub fn begin_new_list(&mut self) {
        self.reset_list();
        self.new_list();
    }

    pub fn next_new_list(&mut self) {
        unsafe {
            gl::EndList();
        }
        self.enum_idx += 1;
        if self.enum_idx >= self.idx + self.num || self.enum_idx < self.idx {
            panic!("Can't create new list. Index out of bound.");
        }
        unsafe {
            gl::NewList(self.enum_idx, gl::GL_COMPILE);
        }
    }

    pub fn end_new_list(&self) {
        unsafe {
            gl::EndList();
        }
    }

    fn reset_list(&mut self) {
        self.enum_idx = self.idx;
    }

    pub fn new_list(&self) {
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
        unsafe {
            gl::CallList(self.idx + i);
        }
    }
}
