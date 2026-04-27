//! LUFS taken from: https://www.itu.int/dms_pubrec/itu-r/rec/bs/R-REC-BS.1770-5-202311-I!!PDF-E.pdf

#[derive(Clone, Copy)]
struct IirFilter<const ORDER: usize> {
    pub b_0: f32,

    pub a: [f32; ORDER],
    pub b: [f32; ORDER],

    delay_x: [f32; ORDER],
    delay_y: [f32; ORDER],
}
impl<const ORDER: usize> IirFilter<ORDER> {
    pub fn process(&mut self, sample: f32) -> f32 {
        let res =
            self.delay_x.iter().copied().zip(self.b).fold(0., |acc, (a, b)| acc + a*b) -
            self.delay_y.iter().copied().zip(self.a).fold(0., |acc, (a, b)| acc + a*b) +
            sample * self.b_0;
        // Shift in the sample to the delay banks
        self.delay_x.rotate_right(1);
        self.delay_y.rotate_right(1);
        self.delay_x[0] = sample;
        self.delay_y[0] = res;
        res
    }

    pub const fn new(b_0: f32, a: [f32; ORDER], b: [f32; ORDER]) -> Self {
        Self {
            b_0, a, b,
            delay_x: [0.; _],
            delay_y: [0.; _]
        }
    }
}


#[derive(Clone, Copy)]
pub struct KWeighting {
    stage_1: IirFilter<2>,
    stage_2: IirFilter<2>,
}
impl KWeighting {
    #[allow(clippy::excessive_precision)]
    pub const fn new() -> Self {
        Self {
            stage_1: IirFilter::new(1.53512485958697, [-1.69065929318241,0.73248077421585], [-2.69169618940638, 1.19839281085285]),
            stage_2: IirFilter::new(1.0, [-1.99004745483398, 0.99007225036621], [-2.0, 1.0])
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.stage_2.process(self.stage_1.process(sample))
    }
}
impl Default for KWeighting {
    fn default() -> Self {
        Self::new()
    }
}

// pub struct UngatedLUFS {
//     filter: KWeighting,
//     filtered: Vec<f32>
// }
// impl UngatedLUFS {
//     pub fn process(&mut self, sample: f32) {
//         self.filtered.rotate_right(1);
//         self.filtered[0] = self.filter.process(sample);
//     }
//     pub fn compute(&self, n_samples: usize) -> f32 {
//         10. * (self.filtered[..n_samples].iter().map(|x| x*x).sum::<f32>() / n_samples as f32).log10() - 0.691
//     }
// }
