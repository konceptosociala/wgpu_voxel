use std::{
    any::{Any, TypeId}, 
    collections::HashMap, 
    sync::Arc,
};
use parking_lot::Mutex;
use hecs::World;

#[derive(Default)]
pub struct StateManager {
    world_handle: Arc<Mutex<World>>,

    states: HashMap<TypeId, Arc<Mutex<dyn State>>>,
}

impl StateManager {
    pub fn new(world: Arc<Mutex<World>>) -> StateManager {
        StateManager {
            world_handle: world,
            states: HashMap::new(),
        }
    }

    pub fn attach(&mut self, state: Arc<Mutex<dyn State>>) {
        let id = state.lock().id();
        state.lock().init(&mut self.world_handle.lock());
        self.states.insert(id, state);
    }

    pub fn detach(&mut self, state: Arc<Mutex<dyn State>>) {
        let id = state.lock().id();
        state.lock().cleanup(&mut self.world_handle.lock());
        self.states.remove(&id);
    }

    pub fn set_enabled<S: State>(&mut self, enabled: bool) {
        if let Some(state) = self.states.get(&TypeId::of::<S>()) {
            if enabled {
                state.lock().on_enable(&mut self.world_handle.lock());
            } else {
                state.lock().on_disable(&mut self.world_handle.lock());
            }
        }
    }

    pub fn update(&mut self) {
        for state in self.states.values() {
            state.lock().update(&mut self.world_handle.lock());
        }
    }
}

pub trait State: Any {
    fn init(&mut self, world: &mut World);

    fn cleanup(&mut self, world: &mut World);

    fn on_enable(&mut self, world: &mut World);

    fn on_disable(&mut self, world: &mut World);

    fn update(&mut self, world: &mut World);

    fn id(&self) -> TypeId {
        self.type_id()
    }
}