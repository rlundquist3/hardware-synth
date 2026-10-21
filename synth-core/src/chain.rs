use core::array;

use alloc::boxed::Box;
use heapless::Vec;

use crate::{
    buffer_pool::BufferPool,
    effects::{
        EFFECT_COUNT, Effect, EffectSnapshot,
        chorus::Chorus,
        echo::Echo,
        filters::{FILTER_COUNT, band_pass, high_pass, low_pass},
        flanger::Flanger,
        gain::Gain,
        reverb::Reverb,
        soft_clipper::SoftClipper,
        vibrato::Vibrato,
    },
    engines::{ENGINE_COUNT, Engine, dual::DualOsc, fm::FMSynth},
};

pub struct Chain {
    pub current_engine_index: usize,
    // TODO: if this feels weird or causes a memory issue, use Vec<EnumOfAllFilters, Count> instead (same for effects)
    engines: Vec<Box<dyn Engine>, ENGINE_COUNT>,
    filters: Vec<Box<dyn Effect>, FILTER_COUNT>,
    effects: Vec<Box<dyn Effect>, EFFECT_COUNT>,
}

impl Chain {
    pub fn new(buffer_pool: &mut BufferPool) -> Self {
        let engines: Vec<Box<dyn Engine>, ENGINE_COUNT> =
            Vec::from_array([Box::new(FMSynth::new()), Box::new(DualOsc::new())]);
        let filters: Vec<Box<dyn Effect>, FILTER_COUNT> = Vec::from_array([
            Box::new(low_pass::new(200.0, 1.0)),
            Box::new(high_pass::new(1000.0, 1.0)),
            Box::new(band_pass::new(800.0, 1.0)),
        ]);
        let effects: Vec<Box<dyn Effect>, EFFECT_COUNT> = Vec::from_array([
            Box::new(SoftClipper::new(3.0)),
            Box::new(Echo::new(0.2, 0.4, buffer_pool)),
            Box::new(Vibrato::new(5.0, 0.1, 5.0, buffer_pool)),
            Box::new(Flanger::new(3.0, 0.1, 0.2, buffer_pool)),
            Box::new(Chorus::new(buffer_pool)),
            Box::new(Reverb::new(buffer_pool)),
            Box::new(Gain::new(0.0)),
        ]);

        Chain {
            current_engine_index: 1,
            engines,
            filters,
            effects,
        }
    }

    pub fn set_engine(&mut self, index: usize) {
        self.current_engine_index = index;
    }

    pub fn get_engine(&mut self) -> &mut dyn Engine {
        self.engines[self.current_engine_index].as_mut()
    }

    pub fn get_engine_list(&self) -> [&'static str; ENGINE_COUNT] {
        array::from_fn(|index| self.engines[index].get_name())
    }

    pub fn get_filters(&mut self) -> &mut Vec<Box<dyn Effect>, FILTER_COUNT> {
        &mut self.filters
    }

    pub fn get_filters_snapshot(&self) -> Vec<EffectSnapshot, FILTER_COUNT> {
        self.filters
            .iter()
            .map(|f| EffectSnapshot::from_effect(f.as_ref()))
            .collect()
    }

    pub fn get_effects(&mut self) -> &mut Vec<Box<dyn Effect>, EFFECT_COUNT> {
        &mut self.effects
    }

    pub fn get_effects_snapshot(&self) -> Vec<EffectSnapshot, EFFECT_COUNT> {
        self.effects
            .iter()
            .map(|e| EffectSnapshot::from_effect(e.as_ref()))
            .collect()
    }
}

impl Iterator for Chain {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.engines[self.current_engine_index]
            .next()
            .unwrap_or(0.0);
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
