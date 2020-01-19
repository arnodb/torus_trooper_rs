use std::ops::{Index, IndexMut};

pub mod bullet;
pub mod enemy;
pub mod float_letter;
pub mod particle;
pub mod shot;

pub struct PoolActor<T> {
    actor: T,
    state: ActorState,
}

enum ActorState {
    NotActing,
    Acting { generation: usize },
}

#[derive(Debug, Clone, Copy)]
pub struct PoolActorRef {
    idx: usize,
    generation: usize,
}

pub struct Pool<T> {
    actors: Vec<PoolActor<T>>,
    idx: usize,
    generation: usize,
    num: usize,
}

impl<T: Default> Pool<T> {
    pub fn new(n: usize) -> Self {
        let mut actors = Vec::with_capacity(n);
        for _ in 0..n {
            actors.push(PoolActor {
                actor: T::default(),
                state: ActorState::NotActing,
            });
        }
        Self {
            actors,
            idx: 0,
            generation: 0,
            num: 0,
        }
    }

    pub fn get_instance(&mut self) -> Option<(&mut T, PoolActorRef)> {
        let mut found = false;
        let mut idx = self.idx;
        {
            let len = self.actors.len();
            for _ in 0..len {
                idx = (idx + 1) % len;
                let pa = &self.actors[idx];
                if let ActorState::NotActing = pa.state {
                    found = true;
                    break;
                }
            }
        }
        self.idx = idx;
        if found {
            let generation = self.generation + 1;
            self.generation = generation;
            let pa = &mut self.actors[idx];
            pa.state = ActorState::Acting { generation };
            self.num += 1;
            Some((&mut pa.actor, PoolActorRef { idx, generation }))
        } else {
            None
        }
    }

    pub fn get_instance_forced(&mut self) -> (&mut T, PoolActorRef) {
        let idx = (self.idx + 1) % self.actors.len();
        self.idx = idx;
        let generation = self.generation + 1;
        self.generation = generation;
        let pa = &mut self.actors[idx];
        if let ActorState::NotActing = pa.state {
            self.num += 1;
        }
        pa.state = ActorState::Acting { generation };
        (&mut pa.actor, PoolActorRef { idx, generation })
    }

    pub fn release(&mut self, index: PoolActorRef) {
        let pa = &mut self.actors[index.idx];
        match &pa.state {
            ActorState::Acting { generation, .. } => {
                if *generation != index.generation {
                    panic!("Actor doesn't exist any more");
                }
                self.num -= 1;
            }
            ActorState::NotActing => {
                panic!("Actor not found");
            }
        };
        pa.state = ActorState::NotActing;
    }

    pub fn clear(&mut self) {
        for pa in &mut self.actors {
            pa.state = ActorState::NotActing;
        }
        self.idx = 0;
        self.num = 0;
    }

    pub fn get_num(&self) -> usize {
        self.num
    }

    fn as_refs(&self) -> Vec<PoolActorRef> {
        self.actors
            .iter()
            .enumerate()
            .filter_map(|(idx, pa)| match pa.state {
                ActorState::Acting { generation, .. } => Some(PoolActorRef { idx, generation }),
                ActorState::NotActing => None,
            })
            .collect()
    }

    fn maybe_index_mut(&mut self, index: PoolActorRef) -> Option<&mut T> {
        let pa = &mut self.actors[index.idx];
        match &pa.state {
            ActorState::Acting { generation, .. } => {
                if *generation != index.generation {
                    return None;
                }
            }
            ActorState::NotActing => {
                return None;
            }
        }
        Some(&mut pa.actor)
    }
}

impl<T> Index<PoolActorRef> for Pool<T> {
    type Output = T;
    fn index(&self, index: PoolActorRef) -> &Self::Output {
        let pa = &self.actors[index.idx];
        match &pa.state {
            ActorState::Acting { generation, .. } => {
                if *generation != index.generation {
                    panic!("Actor not found");
                }
            }
            ActorState::NotActing => {
                panic!("Actor not found");
            }
        };
        &pa.actor
    }
}

impl<T> IndexMut<PoolActorRef> for Pool<T> {
    fn index_mut(&mut self, index: PoolActorRef) -> &mut Self::Output {
        let pa = &mut self.actors[index.idx];
        match &pa.state {
            ActorState::Acting { generation, .. } => {
                if *generation != index.generation {
                    panic!("Actor not found");
                }
            }
            ActorState::NotActing => {
                panic!("Actor not found");
            }
        };
        &mut pa.actor
    }
}

impl<'a, T> IntoIterator for &'a Pool<T> {
    type Item = &'a T;
    type IntoIter = std::iter::FilterMap<
        std::slice::Iter<'a, PoolActor<T>>,
        fn(&'a PoolActor<T>) -> Option<&'a T>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.actors.iter().filter_map(|pa| match pa.state {
            ActorState::Acting { .. } => Some(&pa.actor),
            ActorState::NotActing => None,
        })
    }
}

impl<'a, T> IntoIterator for &'a mut Pool<T> {
    type Item = &'a mut T;
    type IntoIter = std::iter::FilterMap<
        std::slice::IterMut<'a, PoolActor<T>>,
        fn(&'a mut PoolActor<T>) -> Option<&'a mut T>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.actors.iter_mut().filter_map(|pa| match pa.state {
            ActorState::Acting { .. } => Some(&mut pa.actor),
            ActorState::NotActing => None,
        })
    }
}
