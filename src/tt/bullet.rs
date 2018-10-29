use crate::util::vector::Vector;

pub trait BulletTarget {
    fn get_target_pos(&self) -> Vector;
}
