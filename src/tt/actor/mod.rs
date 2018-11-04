pub mod shot;

pub struct Pool<T, S> {
    actors: Vec<PoolActor<T, S>>,
    idx: usize,
    special_instance_idx: Option<usize>,
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
            actors,
            idx: 0,
            special_instance_idx: None,
        }
    }

    pub fn get_instance(&mut self, spec: S) -> Option<&mut T> {
        let len = self.actors.len();
        let mut idx = self.idx;
        let mut found = false;
        for _ in 0..len {
            idx = (idx + 1) % len;
            let pa = &mut self.actors[idx];
            if let ActorState::NotActing = pa.state {
                self.idx = idx;
                found = true;
                break;
            }
        }
        self.idx = idx;
        if found {
            let pa = &mut self.actors[idx];
            pa.state = ActorState::Acting(spec);
            Some(&mut pa.actor)
        } else {
            None
        }
    }

    pub fn get_special_instance(&mut self, spec: S) -> &mut T {
        if let None = self.special_instance_idx {
            let idx = (self.idx + 1) % self.actors.len();
            self.idx = idx;
            self.special_instance_idx = Some(idx);
            self.actors[idx].state = ActorState::Acting(spec);
        }
        &mut self.actors[self.special_instance_idx.unwrap()].actor
    }

    pub fn release_special_instance(&mut self) {
        if let Some(idx) = self.special_instance_idx {
            self.actors[idx].state = ActorState::NotActing;
            self.special_instance_idx = None;
        }
    }

    pub fn clear(&mut self) {
        self.special_instance_idx = None;
        for pa in &mut self.actors {
            pa.state = ActorState::NotActing;
        }
        self.idx = 0;
    }

    pub fn mov<O>(&mut self, op: O)
    where
        O: Fn(&S, &mut T) -> bool,
    {
        for (idx, pool_actor) in &mut self.actors.iter_mut().enumerate() {
            if let ActorState::Acting(spec) = &pool_actor.state {
                let remove = op(spec, &mut pool_actor.actor);
                if remove {
                    pool_actor.state = ActorState::NotActing;
                    if Some(idx) == self.special_instance_idx {
                        self.special_instance_idx = None;
                    }
                }
            }
        }
    }

    pub fn draw<O>(&self, op: O)
    where
        O: Fn(&S, &T),
    {
        for pool_actor in &self.actors {
            if let ActorState::Acting(spec) = &pool_actor.state {
                op(spec, &pool_actor.actor);
            }
        }
    }
}
