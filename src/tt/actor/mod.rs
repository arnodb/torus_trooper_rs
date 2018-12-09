use std::ops::{Index, IndexMut};

pub mod enemy;
pub mod shot;

pub struct PoolActor<T, S> {
    pub actor: T,
    state: ActorState<S>,
}

impl<T, S> PoolActor<T, S> {
    fn prepare(&mut self, pa_ref: PoolActorRef, spec: S) {
        if let ActorState::NotActing = self.state {
            self.state = ActorState::Acting {
                spec,
                generation: pa_ref.generation,
            }
        } else {
            panic!("called `PoolActor::prepare()` on an `Acting` value");
        }
    }

    pub fn release(&mut self) {
        self.state = ActorState::NotActing;
    }
}

enum ActorState<S> {
    NotActing,
    Acting { spec: S, generation: usize },
}

impl<S> ActorState<S> {
    #[inline]
    pub fn spec(&self) -> &S {
        match self {
            ActorState::Acting { spec, .. } => spec,
            ActorState::NotActing => panic!("called `ActorState::spec()` on a `NotActing` value"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PoolActorRef {
    idx: usize,
    generation: usize,
}

pub struct Pool<T, S> {
    actors: Vec<PoolActor<T, S>>,
    idx: usize,
    generation: usize,
}

impl<T: Default, S> Pool<T, S> {
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
        }
    }

    pub fn get_instance(&mut self) -> Option<(&mut PoolActor<T, S>, PoolActorRef)> {
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
            self.generation += 1;
            Some((
                &mut self.actors[idx],
                PoolActorRef {
                    idx,
                    generation: self.generation,
                },
            ))
        } else {
            println!("cannot get instance");
            None
        }
    }

    pub fn get_instance_forced(&mut self) -> (&mut PoolActor<T, S>, PoolActorRef) {
        let idx = (self.idx + 1) % self.actors.len();
        self.idx = idx;
        self.generation += 1;
        let pa = &mut self.actors[idx];
        pa.state = ActorState::NotActing;
        (
            pa,
            PoolActorRef {
                idx,
                generation: self.generation,
            },
        )
    }

    pub fn clear(&mut self) {
        for pa in &mut self.actors {
            pa.state = ActorState::NotActing;
        }
        self.idx = 0;
    }

    pub fn get_num(&self) -> usize {
        // TODO improve performance
        self.actors
            .iter()
            .map(|pool_actor| {
                if let ActorState::Acting { .. } = &pool_actor.state {
                    1
                } else {
                    0
                }
            })
            .sum()
    }
}

impl<T, S> Index<PoolActorRef> for Pool<T, S> {
    type Output = PoolActor<T, S>;
    fn index(&self, index: PoolActorRef) -> &Self::Output {
        let pa = &self.actors[index.idx];
        match &pa.state {
            ActorState::Acting { generation, .. } => {
                if *generation != index.generation {
                    panic!("Actor doesn't exist any more");
                }
            }
            ActorState::NotActing => {
                panic!("Actor not found");
            }
        };
        pa
    }
}

impl<T, S> IndexMut<PoolActorRef> for Pool<T, S> {
    fn index_mut(&mut self, index: PoolActorRef) -> &mut Self::Output {
        let pa = &mut self.actors[index.idx];
        match &pa.state {
            ActorState::Acting { generation, .. } => {
                if *generation != index.generation {
                    panic!("Actor doesn't exist any more");
                }
            }
            ActorState::NotActing => {
                panic!("Actor not found");
            }
        };
        pa
    }
}

impl<'a, T, S> IntoIterator for &'a Pool<T, S> {
    type Item = &'a PoolActor<T, S>;
    type IntoIter =
        std::iter::Filter<std::slice::Iter<'a, PoolActor<T, S>>, fn(&&PoolActor<T, S>) -> bool>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.actors).into_iter().filter(|pa| match pa.state {
            ActorState::Acting { .. } => true,
            ActorState::NotActing => false,
        })
    }
}

impl<'a, T, S> IntoIterator for &'a mut Pool<T, S> {
    type Item = &'a mut PoolActor<T, S>;
    type IntoIter = std::iter::Filter<
        std::slice::IterMut<'a, PoolActor<T, S>>,
        fn(&&mut PoolActor<T, S>) -> bool,
    >;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.actors).into_iter().filter(|pa| match pa.state {
            ActorState::Acting { .. } => true,
            ActorState::NotActing => false,
        })
    }
}
