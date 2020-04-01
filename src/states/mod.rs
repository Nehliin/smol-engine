use crate::engine::InputEvent;

use legion::prelude::*;

pub mod basic_state;
pub use basic_state::BasicState;
pub trait State {
    // resources??
    fn start(&mut self, world: &mut World, resources: &mut Resources);
    fn update(&mut self, world: &mut World, resources: &mut Resources); // -> transition
    fn stop(&mut self, world: &mut World, resources: &mut Resources);
    fn handle_event(
        &mut self,
        event: InputEvent,
        world: &mut World,
        resources: &mut Resources,
    ) -> bool;
}

/*
Målbild:
Ett system för att hantera intput per state kan va fler men det är en resource själva inputhanteringen
Den resourcen kan man query knapptryck och mouse input
eg input.get_key()

1. SKapa input handler resource som har en crossbeam receiver som fylls med input events
2. registrera callbacks som tar sender och skickar dessa events till input handler

*/
