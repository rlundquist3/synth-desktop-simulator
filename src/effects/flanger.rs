use crate::{
    effects::{Effect, EffectComponent, effect_components::lfo_delay_line::LFODelay},
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

#[derive(Debug)]
pub struct Flanger {
    lfo_delay: LFODelay,
    parameters: Vec<Parameter>,
}

impl Flanger {
    pub fn new(delay_ms: f32, amp: f32, freq: f32) -> Self {
        let parameters = vec![
            Parameter::new("Toggle", 0.0, 1.0, (0.0, 1.0), |v| match v {
                1.0 => format!("on"),
                _ => format!("off"),
            }),
            Parameter::new("Delay", delay_ms, 1.0, (0.0, 15.0), |v| {
                format!("{:.0}ms", v)
            }),
            Parameter::new("Amp", amp, 0.05, (0.0, 5.0), |v| format!("{:.2}", v)),
            Parameter::new("Freq", freq, 0.1, (0.0, 10.0), |v| format!("{:.1}Hz", v)),
        ];

        Flanger {
            lfo_delay: LFODelay::new(
                parameters[1].clone(),
                parameters[2].clone(),
                parameters[3].clone(),
            ),
            parameters,
        }
    }
}

impl Effect for Flanger {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(Flanger {
            lfo_delay: self.lfo_delay.clone(),
            parameters: self.parameters.clone(),
        })
    }

    fn process(&mut self, sample: f32) -> f32 {
        let on = self.parameters[0].get_value();

        if on != 1.0 {
            return sample;
        }

        let delayed = self.lfo_delay.process(sample);
        let result = 0.5 * sample + 0.5 * delayed;

        result
    }

    fn get_name(&self) -> String {
        String::from("Flanger")
    }
}

impl UserParameters for Flanger {
    fn get_parameters(&self) -> Vec<Parameter> {
        self.parameters.clone()
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
        let param = self.parameters.get(index)?;
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };

        let updated_value = (param.get_value() + delta).clamp(param.range.0, param.range.1);
        param.set_value(updated_value);

        Some(param.clone())
    }
}
