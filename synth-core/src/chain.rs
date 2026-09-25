use alloc::boxed::Box;
use heapless::Vec;

use crate::{
    effects::{
        EFFECT_COUNT, Effect,
        filters::{FILTER_COUNT, band_pass, high_pass, low_pass},
        gain::Gain,
    },
    engines::Engine,
};

pub struct Chain {
    engine: Box<dyn Engine>,
    // TODO: if this feels weird or causes a memory issue, use Vec<EnumOfAllFilters, Count> instead (same for effects)
    filters: Vec<Box<dyn Effect>, FILTER_COUNT>,
    effects: Vec<Box<dyn Effect>, EFFECT_COUNT>,
}

impl Chain {
    pub fn new(engine: Box<dyn Engine>) -> Self {
        let filters: Vec<Box<dyn Effect>, FILTER_COUNT> = Vec::from_array([
            Box::new(low_pass::new(200.0, 1.0)),
            Box::new(high_pass::new(1000.0, 1.0)),
            Box::new(band_pass::new(800.0, 1.0)),
        ]);
        let effects: Vec<Box<dyn Effect>, EFFECT_COUNT> =
            Vec::from_array([Box::new(Gain::new(0.0))]);

        Chain {
            engine,
            filters,
            effects,
        }
    }

    pub fn set_engine(&mut self, engine: Box<dyn Engine>) {
        self.engine = engine;
    }

    pub fn get_engine(&mut self) -> &mut dyn Engine {
        self.engine.as_mut()
    }

    pub fn get_filters(&mut self) -> &mut Vec<Box<dyn Effect>, FILTER_COUNT> {
        &mut self.filters
    }

    pub fn get_effects(&mut self) -> &mut Vec<Box<dyn Effect>, EFFECT_COUNT> {
        &mut self.effects
    }
}

impl Iterator for Chain {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.engine.next().unwrap_or(0.0);
        let filtered = self
            .filters
            .iter_mut()
            .fold(raw, |sample, filter| filter.process(sample));

        Some(
            self.effects
                .iter_mut()
                .fold(filtered, |sample, effect| effect.process(sample))
                .clamp(-1.0, 1.0),
        )
    }
}
