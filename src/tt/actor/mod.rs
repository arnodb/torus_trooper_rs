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
    actors: Box<[PoolActor<T>]>,
    idx: usize,
    generation: usize,
    num: usize,
}

impl<T> Pool<T> {
    pub fn new(n: usize) -> Self
    where
        T: Default,
    {
        let mut actors = Vec::with_capacity(n);
        for _ in 0..n {
            actors.push(PoolActor {
                actor: T::default(),
                state: ActorState::NotActing,
            });
        }
        Self {
            actors: actors.into_boxed_slice(),
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
        for pa in &mut self.actors[..] {
            pa.state = ActorState::NotActing;
        }
        self.idx = 0;
        self.num = 0;
    }

    pub fn get_num(&self) -> usize {
        self.num
    }

    // This is inspired by split_at_mut, enjoy ;-).
    fn split(&mut self) -> (PoolReleaseArea<T>, PoolGetInstanceArea<T>) {
        let generation = self.generation;
        let self_2 = unsafe { &mut *(self as *mut Self) };
        (
            PoolReleaseArea {
                pool: self,
                generation,
            },
            PoolGetInstanceArea { pool: self_2 },
        )
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

struct PoolReleaseArea<'a, T> {
    pool: &'a mut Pool<T>,
    generation: usize,
}

impl<'a, T> PoolReleaseArea<'a, T> {
    pub fn release(&mut self, index: PoolActorRef) {
        // Early catch (Pool::release is safe anyway)
        if index.generation > self.generation {
            panic!("Trying to release an element which is not part of this area");
        }
        // This, by design, gives the slot back to the "get_instance" area.
        self.pool.release(index)
    }

    fn into_iter(&'a mut self) -> PoolReleaseAreaIter<'a, T> {
        let generation = self.generation;
        PoolReleaseAreaIter {
            pool: self,
            generation,
            index: 0,
            current_ref: None,
        }
    }
}

struct PoolReleaseAreaIter<'a, T> {
    pool: &'a mut PoolReleaseArea<'a, T>,
    generation: usize,
    index: usize,
    current_ref: Option<PoolActorRef>,
}

impl<'a, T> PoolReleaseAreaIter<'a, T> {
    fn next(&mut self) -> Option<(&mut T, PoolActorRef)> {
        let mut pa_ref = PoolActorRef {
            idx: self.index,
            generation: 0,
        };
        let max_index = {
            let actors = &self.pool.pool.actors;
            let max_index = actors.len();
            let max_gen = self.generation;
            while pa_ref.idx < max_index {
                let actor = &actors[pa_ref.idx];
                match actor.state {
                    ActorState::Acting { generation } if generation <= max_gen => {
                        pa_ref.generation = generation;
                        break;
                    }
                    ActorState::Acting { .. } | ActorState::NotActing => {}
                }
                pa_ref.idx += 1;
            }
            max_index
        };
        if pa_ref.idx < max_index {
            self.current_ref = Some(pa_ref);
            self.index = pa_ref.idx + 1;
            Some((&mut self.pool.pool.actors[pa_ref.idx].actor, pa_ref))
        } else {
            self.current_ref = None;
            self.index = pa_ref.idx;
            None
        }
    }

    pub fn release(&mut self) {
        if let Some(current_ref) = self.current_ref {
            self.pool.release(current_ref)
        } else {
            panic!("No actor to release in this iterator");
        }
    }
}

struct PoolGetInstanceArea<'a, T> {
    pool: &'a mut Pool<T>,
}

impl<'a, T> PoolGetInstanceArea<'a, T> {
    pub fn get_instance(&mut self) -> Option<(&mut T, PoolActorRef)> {
        self.pool.get_instance()
    }
}

impl<'a, T> Index<PoolActorRef> for PoolGetInstanceArea<'a, T> {
    type Output = T;
    fn index(&self, index: PoolActorRef) -> &Self::Output {
        &self.pool[index]
    }
}
