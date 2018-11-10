use std::cell::{Cell, RefCell};

pub mod enemy;
pub mod shot;

pub struct Pool<T, S> {
    actors: RefCell<Vec<PoolActor<T, S>>>,
    idx: usize,
    special_instance_idx: Cell<Option<usize>>,
}

pub struct PoolActor<T, S> {
    actor: T,
    state: ActorState<S>,
}

enum ActorState<S> {
    NotActing,
    Acting(S),
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
            actors: RefCell::new(actors),
            idx: 0,
            special_instance_idx: Cell::new(None),
        }
    }

    pub fn get_instance_and<O>(&mut self, spec: S, mut op: O)
    where
        O: FnMut(&mut T),
    {
        let mut found = false;
        let mut idx = self.idx;
        {
            let actors = self.actors.borrow();
            let len = actors.len();
            for _ in 0..len {
                idx = (idx + 1) % len;
                let pa = &actors[idx];
                if let ActorState::NotActing = pa.state {
                    self.idx = idx;
                    found = true;
                    break;
                }
            }
        }
        self.idx = idx;
        if found {
            let pa = &mut self.actors.borrow_mut()[idx];
            pa.state = ActorState::Acting(spec);
            op(&mut pa.actor);
        }
    }

    pub fn get_special_instance_and<O>(&mut self, spec: S, op: O)
    where
        O: Fn(&mut T) -> bool,
    {
        let idx = match self.special_instance_idx.get() {
            Some(idx) => idx,
            None => {
                let idx = (self.idx + 1) % self.actors.borrow().len();
                self.idx = idx;
                self.special_instance_idx.set(Some(idx));
                idx
            }
        };
        let pa = &mut self.actors.borrow_mut()[idx];
        pa.state = ActorState::Acting(spec);
        let remove = op(&mut pa.actor);
        if remove {
            pa.state = ActorState::NotActing;
            self.special_instance_idx.set(None);
        }
    }

    pub fn clear(&mut self) {
        self.special_instance_idx.set(None);
        for pa in self.actors.borrow_mut().iter_mut() {
            pa.state = ActorState::NotActing;
        }
        self.idx = 0;
    }

    pub fn foreach_mut<O>(&self, mut op: O)
    where
        O: FnMut(&S, &mut T) -> bool,
    {
        for (idx, pool_actor) in self.actors.borrow_mut().iter_mut().enumerate() {
            let remove = if let ActorState::Acting(spec) = &pool_actor.state {
                op(spec, &mut pool_actor.actor)
            } else {
                false
            };
            if remove {
                pool_actor.state = ActorState::NotActing;
                if Some(idx) == self.special_instance_idx.get() {
                    self.special_instance_idx.set(None);
                }
            }
        }
    }

    pub fn foreach<O>(&self, op: O)
    where
        O: Fn(&S, &T),
    {
        for pool_actor in self.actors.borrow().iter() {
            if let ActorState::Acting(spec) = &pool_actor.state {
                op(spec, &pool_actor.actor);
            }
        }
    }

    pub fn get_num(&self) -> usize {
        // TODO improve performance
        self.actors.borrow().iter()
            .map(|pool_actor| {
                if let ActorState::Acting(_) = &pool_actor.state {
                    1
                } else {
                    0
                }
            })
            .sum()
    }
}
